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
            Default::default(),
        )
    }

    fn build_comment(&self, comment: String, metadata: HashMap<String, String>) -> NodeRef {
        Node::new_leaf(
            LeafData::Comment {
                comment: comment.clone(),
            },
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            comment.lines().fold(String::new(), |mut acc, line| {
                acc.push_str(format!("% {line}\n").as_str());
                acc
            }),
            metadata,
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
            Default::default(),
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
            Default::default(),
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
            Default::default(),
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
            Default::default(),
        )
    }

    fn build_file(&self, path: String, children: Vec<NodeRef>) -> NodeRef {
        Node::new_expandable(
            ExpandableData::File { path: path.clone() },
            children,
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            format!("\\input{{{path}}}"),
            Default::default(),
        )
    }

    fn build_env(&self, name: String, children: Vec<NodeRef>) -> NodeRef {
        Node::new_expandable(
            ExpandableData::Environment { name: name.clone() },
            children,
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            format!("\\begin{{{}\n...\n\\end{{{}}}", name.clone(), name),
            Default::default(),
        )
    }

    fn build_segment(&self, heading: String, children: Vec<NodeRef>, raw: String) -> NodeRef {
        Node::new_expandable(
            ExpandableData::Segment { heading },
            children,
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            raw,
            Default::default(),
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
            String::new(),
            Default::default(),
        )
    }

    fn parser(&self) -> impl Parser<char, NodeRef, Error = Simple<char>> + '_ {
        let key_value_pair = text::ident()
            .then_ignore(just(":"))
            .then(text::ident().padded())
            .map(|(key, value)| (key, value))
            .boxed();

        let metadata = just("% TEXLA METADATA")
            .padded()
            .ignore_then(
                key_value_pair
                    .separated_by(just(","))
                    .allow_trailing()
                    .delimited_by(just("("), just(")")),
            )
            .padded()
            .or_not()
            .map(|option| match option {
                Some(pairs) => pairs.into_iter().collect::<HashMap<String, String>>(),
                None => HashMap::new(),
            })
            .boxed();

        let comment = just("%")
            .padded()
            .ignore_then(take_until(choice((
                newline().then(none_of("%").rewind()).to("END"),
                just("% TEXLA").rewind(),
            ))))
            .then(metadata.clone())
            .try_map(|((comment, _), metadata), span| {
                let string: String = comment.iter().collect();
                if string.starts_with("TEXLA") {
                    Err(Simple::custom(
                        span,
                        "found TEXLA Metadata instead of regular Comment",
                    ))
                } else {
                    Ok(self.build_comment(comment.iter().collect(), metadata))
                }
            })
            .padded()
            .boxed();

        let heading = none_of("}")
            .repeated()
            .at_least(1)
            .delimited_by(just("{"), just("}"))
            .collect::<String>()
            .boxed();
        // FIXME none_of("}") is not sufficient since a heading may contain pairs of curly braces

        // TODO write parsers

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
            just("\\end").rewind(),
            just("\\end{document}").rewind(),
            just("%").rewind(),
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
            comment.clone(),
            text_node.clone(),
        ))
        .boxed();

        let environment = recursive(|environment| {
            just("\\begin")
                .ignore_then(heading.clone())
                .padded()
                .then(leaf.clone().or(environment).repeated())
                .then(just("\\end").ignore_then(heading.clone()).padded())
                .try_map(|((name_begin, children), name_end), span| {
                    if name_begin != name_end {
                        Err(Simple::custom(span, "Environment not closed correctly"))
                    } else {
                        Ok(self.build_env(name_end, children))
                    }
                })
        })
        .boxed();

        let prelude = choice((leaf.clone(), environment.clone())).boxed();

        let prelude_in_inputs = recursive(|prelude_in_inputs| {
            just("% TEXLA FILE BEGIN")
                .ignore_then(heading.clone().padded())
                .then(prelude.clone().or(prelude_in_inputs).repeated())
                .then_ignore(just("% TEXLA FILE END").padded())
                .map(|(path, children)| self.build_file(path, children))
        })
        .boxed();

        let prelude_any = prelude_in_inputs
            .clone()
            .or(prelude.clone())
            .repeated()
            .padded()
            .boxed();
        // TODO extract method
        let subsection = just("\\subsection")
            .ignore_then(heading.clone())
            .then_ignore(newline())
            .then(prelude_any.clone())
            .map(|(heading, blocks): (String, Vec<NodeRef>)| {
                self.build_segment(
                    heading.clone(),
                    blocks,
                    format!("\\subsection{{{heading}}}"),
                )
            })
            .boxed();

        let subsections_in_inputs = recursive(|subsections_in_inputs| {
            just("% TEXLA FILE BEGIN")
                .ignore_then(heading.clone().padded())
                .then(prelude_any.clone())
                .then(subsections_in_inputs.or(subsection.clone()).repeated())
                .then_ignore(just("% TEXLA FILE END").padded())
                .map(|((path, mut prelude), mut children)| {
                    self.build_file(path, {
                        prelude.append(&mut children);
                        prelude
                    })
                })
        })
        .boxed();

        let subsection_any = subsections_in_inputs
            .clone()
            .or(subsection.clone())
            .padded()
            .boxed();

        let section = just("\\section")
            .ignore_then(heading.clone())
            .then_ignore(newline())
            .then(prelude_any.clone())
            .then(subsection_any.clone().repeated())
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

        let sections_in_inputs = recursive(|sections_in_inputs| {
            just("% TEXLA FILE BEGIN")
                .ignore_then(heading.clone().padded())
                .then(prelude_any.clone())
                .then(sections_in_inputs.or(section.clone()).repeated())
                .then_ignore(just("% TEXLA FILE END").padded())
                .map(|((path, mut prelude), mut children)| {
                    self.build_file(path, {
                        prelude.append(&mut children);
                        prelude
                    })
                })
        })
        .boxed();
        // TODO implement all segment levels

        let section_any = sections_in_inputs
            .clone()
            .or(section.clone())
            .padded()
            .boxed();

        let root_children = prelude_any
            .clone()
            .then(choice((
                section_any.clone().repeated().at_least(1), // at_least used so this doesn't match with 0 occurrences and quit
                subsection_any.clone().repeated(), // last item shouldn't have at_least to allow for empty document
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
