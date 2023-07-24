use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;

use axum::body::HttpBody;
use chumsky::prelude::*;
use chumsky::text::newline;
use chumsky::Parser;
use tower::ServiceExt;

use crate::ast;
use crate::ast::node::{ExpandableData, LeafData, MathKind, Node, NodeRef, NodeRefWeak};
use crate::ast::texla_ast::TexlaAst;
use crate::ast::uuid_provider::{TexlaUuidProvider, Uuid};

#[derive(Clone)]
struct LatexParser {
    uuid_provider: RefCell<TexlaUuidProvider>,
    portal: RefCell<HashMap<Uuid, NodeRefWeak>>,
}

pub fn parse_latex(string: String) -> Result<TexlaAst, ast::errors::ParseError> {
    // TODO: for performance, the parser should not be created every time, but reused
    let parser = LatexParser::new();
    let root = parser.parser().parse(string.clone())?;
    let highest_level = parser.find_highest_level().parse(string)?;
    Ok(TexlaAst {
        portal: parser.portal.into_inner(),
        uuid_provider: parser.uuid_provider.into_inner(),
        root,
        highest_level,
    })
}

impl LatexParser {
    // TODO Indentation Support
    fn new() -> Self {
        LatexParser {
            uuid_provider: RefCell::new(TexlaUuidProvider::new()),
            portal: RefCell::new(HashMap::new()),
        }
    }

    fn build_text(&self, text: String) -> NodeRef {
        Node::new_leaf(
            LeafData::Text { text: text.clone() },
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            text,
        )
    }

    fn build_math(&self, text: String, kind: MathKind) -> NodeRef {
        Node::new_leaf(
            LeafData::Math {
                kind: kind.clone(),
                content: text.clone(),
            },
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            match kind {
                MathKind::SquareBrackets => {
                    format!("\\[{}\\]", text.clone())
                }
                MathKind::DoubleDollars => {
                    format!("$${}$$", text.clone())
                }
                MathKind::Displaymath => {
                    format!("\\begin{{displaymath}}{}\\end{{displaymath}}", text.clone())
                }
                MathKind::Equation => {
                    format!("\\begin{{equation}}{}\\end{{equation}}", text.clone())
                }
            },
        )
    }

    fn build_image(&self, options: Option<String>, path: String) -> NodeRef {
        Node::new_leaf(
            LeafData::Image {
                path: path.clone(),
                options: options.clone(),
            },
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            match options {
                None => {
                    format!("\\includegraphics{{{path}}}")
                }
                Some(option) => {
                    format!("\\includegraphics[{option}]{{{path}}}")
                }
            },
        )
    }

    fn build_caption(&self, caption: String) -> NodeRef {
        Node::new_leaf(
            LeafData::Caption {
                caption: caption.clone(),
            },
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            format!("\\caption{{{caption}}}"),
        )
    }

    fn build_label(&self, label: String) -> NodeRef {
        Node::new_leaf(
            LeafData::Label {
                label: label.clone(),
            },
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            format!("\\label{{{label}}}"),
        )
    }

    fn build_file(&self, path: String, children: Vec<NodeRef>) -> NodeRef {
        Node::new_expandable(
            ExpandableData::File { path: path.clone() },
            children,
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            format!("\\input{{{path}}}"),
        )
    }

    fn build_env(&self, name: String, children: Vec<NodeRef>, raw: String) -> NodeRef {
        Node::new_expandable(
            ExpandableData::Environment { name },
            children,
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            raw,
            // TODO include children into raw_latex?
            // TODO construct raw_latex here instead of passing as argument?
        )
    }

    fn build_segment(&self, heading: String, children: Vec<NodeRef>, raw: String) -> NodeRef {
        Node::new_expandable(
            ExpandableData::Segment { heading },
            children,
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            raw,
            // TODO only pass segment level as argument and construct raw_latex here?
        )
    }

    fn build_document(
        &self,
        preamble: String,
        postamble: String,
        children: Vec<NodeRef>,
    ) -> NodeRef {
        Node::new_expandable(
            ExpandableData::Document {
                preamble,
                postamble,
            },
            children,
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            String::new(), // TODO should raw_latex include '\begin{document}' and '\end{document}'?
        )
    }

    fn parser(&self) -> impl Parser<char, NodeRef, Error = Simple<char>> + '_ {
        let heading = none_of("}")
            .repeated()
            .at_least(1)
            .delimited_by(just("{"), just("}"))
            .collect::<String>()
            .boxed();
        // FIXME none_of("}") is not sufficient since a heading may contain pairs of curly braces

        // TODO write parsers
        let environment = just(" ").map(|_| self.build_text(" ".to_string()));
        let input = just(" ").map(|_| self.build_text(" ".to_string()));

        let options = just("[")
            .ignore_then(none_of("]").repeated())
            .then_ignore(just("]"))
            .collect()
            .boxed();

        let image = just("\\includegraphics")
            .ignore_then(options.or_not())
            .then(
                none_of("}")
                    .repeated()
                    .at_least(1)
                    .collect()
                    .delimited_by(just("{"), just("}")),
            )
            .padded()
            .map(|(options, path): (Option<String>, String)| self.build_image(options, path))
            .boxed();

        let double_dollar_math = take_until(just("$$").rewind())
            .delimited_by(just("$$"), just("$$"))
            .map(|(inner, _)| self.build_math(inner.iter().collect(), MathKind::DoubleDollars))
            .boxed();

        let square_br_math = take_until(just("\\]").rewind())
            .delimited_by(just("\\["), just("\\]"))
            .map(|(inner, _)| self.build_math(inner.iter().collect(), MathKind::SquareBrackets))
            .boxed();

        let equation_math = take_until(just("\\end{equation}").rewind())
            .delimited_by(just("\\begin{equation}"), just("\\end{equation}"))
            .map(|(inner, _)| self.build_math(inner.iter().collect(), MathKind::Equation))
            .boxed();

        let displaymath = take_until(just("\\end{displaymath}").rewind())
            .delimited_by(just("\\begin{displaymath}"), just("\\end{displaymath}"))
            .map(|(inner, _)| self.build_math(inner.iter().collect(), MathKind::Displaymath))
            .boxed();

        let math = choice((
            double_dollar_math,
            square_br_math,
            equation_math,
            displaymath,
        ))
        .padded()
        .boxed();

        let caption = just("\\caption")
            .ignore_then(heading.clone())
            .map(|text| self.build_caption(text))
            .padded()
            .boxed();
        let label = just("\\label")
            .ignore_then(heading.clone())
            .map(|text| self.build_label(text))
            .padded()
            .boxed();

        // TODO find way to ignore \sectioning (use keyword?)
        let terminator = choice((
            just("\\section").rewind(),
            just("\\subsection").rewind(),
            // TODO implement all segment levels
            just("\\begin").rewind(),
            just("\\end{document}").rewind(),
            image.clone().to("image").rewind(),
            math.clone().to("math").rewind(),
            caption.clone().to("caption").rewind(),
            label.clone().to("label").rewind(),
            newline().then(newline()).to("\n\n"),
            // TODO recognize and consume also more than 2 newlines
        ))
        .boxed();

        let text_node = take_until(terminator)
            .try_map(|(v, _), span| {
                if !v.is_empty() {
                    Ok(v)
                } else {
                    Err(Simple::custom(span, "Found empty text".to_string()))
                }
            })
            .collect::<String>()
            .then_ignore(newline().or_not())
            .map(|x: String| self.build_text(x.trim_end().to_string()))
            .boxed();

        let leaf = choice((
            image.clone(),
            math.clone(),
            caption.clone(),
            label.clone(),
            text_node.clone(),
        ))
        .boxed();

        let prelude = choice((environment.clone(), input.clone(), leaf.clone())).boxed();

        // TODO extract method
        let subsection = just("\\subsection")
            .ignore_then(heading.clone())
            .then_ignore(newline())
            .then(prelude.clone().repeated())
            .map(|(heading, blocks): (String, Vec<NodeRef>)| {
                self.build_segment(
                    heading.clone(),
                    blocks,
                    format!("\\subsection{{{heading}}}"),
                )
            })
            .boxed();

        let section = just("\\section")
            .ignore_then(heading.clone())
            .then_ignore(newline())
            .then(prelude.clone().repeated())
            .then(subsection.clone().repeated())
            .map(
                |((heading, mut blocks), mut subsections): (
                    (String, Vec<NodeRef>),
                    Vec<NodeRef>,
                )| {
                    blocks.append(&mut subsections);
                    self.build_segment(heading.clone(), blocks, format!("\\section{{{heading}}}"))
                },
            )
            .boxed();

        // TODO implement all segment levels

        let root_children = prelude
            .clone()
            .repeated()
            .then(choice((
                section.clone().repeated().at_least(1), // at_least used so this doesn't match with 0 occurrences and quit
                subsection.clone().repeated(), // last item shouldn't have at_least to allow for empty document
                                               // TODO implement all segment levels
            )))
            .boxed();

        let preamble = take_until(just("\\begin{document}").rewind())
            .map(|(preamble, _)| preamble.iter().collect())
            .boxed();

        let document = preamble
            .clone()
            .or_not()
            .then_ignore(just::<_, _, Simple<char>>("\\begin{document}").padded())
            .then(root_children.clone())
            .then_ignore(just("\\end{document}"))
            .map(|(preamble, (mut leaves, mut segments))| {
                self.build_document(preamble.unwrap_or(String::new()), String::new(), {
                    leaves.append(&mut segments);
                    leaves
                })
            })
            .then_ignore(end().padded())
            .boxed();
        document
    }

    fn find_highest_level(&self) -> impl Parser<char, i8, Error = Simple<char>> + '_ {
        take_until(just("\\section").or(just("\\subsection")))
            .map(|(_trash, keyword)| match keyword {
                "\\section" => 2,
                "\\subsection" => 3,
                _ => 8,
            })
            .boxed()
        // TODO implement all segment levels
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use crate::ast::parser::parse_latex;

    #[test]
    fn simple() {
        let latex = fs::read_to_string(Path::new("./latex_test_files/simple_latex.tex")).unwrap();
        let ast = parse_latex(latex);
        println!("{:#?}", ast);
    }

    #[test]
    fn get_sample_json() {
        let latex = fs::read_to_string(Path::new("./latex_test_files/simple_latex.tex")).unwrap();
        let ast = parse_latex(latex).unwrap();
        let json = serde_json::to_string_pretty(&ast).unwrap();
        fs::write(Path::new("./latex_test_files/simple_latex.json"), json).unwrap();
    }
}
