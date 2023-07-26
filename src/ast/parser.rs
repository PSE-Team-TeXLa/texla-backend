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

    fn build_text(&self, text: String, metadata: HashMap<String, String>) -> NodeRef {
        Node::new_leaf(
            LeafData::Text { text: text.clone() },
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            text,
            metadata,
        )
    }

    fn build_comment(&self, comment: String, metadata: HashMap<String, String>) -> NodeRef {
        Node::new_leaf(
            LeafData::Comment {
                comment: format!("% {}", comment.clone()),
            },
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            format!("% {}", comment.clone()),
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

    fn build_image(
        &self,
        options: Option<String>,
        path: String,
        metadata: HashMap<String, String>,
    ) -> NodeRef {
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
            metadata,
        )
    }

    fn build_caption(&self, caption: String, metadata: HashMap<String, String>) -> NodeRef {
        Node::new_leaf(
            LeafData::Caption {
                caption: caption.clone(),
            },
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            format!("\\caption{{{caption}}}"),
            metadata,
        )
    }

    fn build_label(&self, label: String, metadata: HashMap<String, String>) -> NodeRef {
        Node::new_leaf(
            LeafData::Label {
                label: label.clone(),
            },
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            format!("\\label{{{label}}}"),
            metadata,
        )
    }

    fn build_file(
        &self,
        path: String,
        children: Vec<NodeRef>,
        metadata: HashMap<String, String>,
    ) -> NodeRef {
        Node::new_expandable(
            ExpandableData::File { path: path.clone() },
            children,
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            format!("\\input{{{path}}}"),
            metadata,
        )
    }

    fn build_env(
        &self,
        name: String,
        children: Vec<NodeRef>,
        metadata: HashMap<String, String>,
    ) -> NodeRef {
        Node::new_expandable(
            ExpandableData::Environment { name: name.clone() },
            children,
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            format!("\\begin{{{}\n...\n\\end{{{}}}", name.clone(), name),
            metadata,
        )
    }

    fn build_segment(
        &self,
        heading: String,
        children: Vec<NodeRef>,
        raw: String,
        metadata: HashMap<String, String>,
    ) -> NodeRef {
        Node::new_expandable(
            ExpandableData::Segment { heading },
            children,
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            raw,
            metadata,
        )
    }

    fn build_document(
        &self,
        preamble: String,
        postamble: String,
        children: Vec<NodeRef>,
        metadata: HashMap<String, String>,
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
            metadata,
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

        let comment = metadata
            .clone()
            .then_ignore(just("%").padded())
            .then(take_until(choice((
                newline().then(none_of("%").rewind()).to("END"),
                just("% TEXLA").rewind(),
            ))))
            .try_map(|(metadata, (comment, _)), span| {
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

        let image = metadata
            .clone()
            .then_ignore(just("\\includegraphics"))
            .then(options.or_not())
            .then(
                none_of("}")
                    .repeated()
                    .at_least(1)
                    .collect::<String>()
                    .delimited_by(just("{"), just("}")),
            )
            .padded()
            .map(|((metadata, options), path)| {
                self.build_image(options, path.parse().unwrap(), metadata)
            })
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

        let caption = metadata
            .clone()
            .then_ignore(just("\\caption"))
            .then(heading.clone())
            .map(|(metadata, text)| self.build_caption(text, metadata))
            .padded()
            .boxed();

        let label = metadata
            .clone()
            .then_ignore(just("\\label"))
            .then(heading.clone())
            .map(|(metadata, text)| self.build_label(text, metadata))
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

        let text_node = metadata
            .clone()
            .then(take_until(terminator))
            .try_map(|(metadata, (v, _)), span| {
                if !v.is_empty() {
                    Ok((metadata, v))
                } else {
                    Err(Simple::custom(span, "Found empty text".to_string()))
                }
            })
            .then_ignore(newline().or_not())
            .map(|(metadata, x)| {
                self.build_text(
                    x.iter().collect::<String>().trim_end().to_string(),
                    metadata,
                )
            })
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
            metadata
                .clone()
                .then_ignore(just("\\begin"))
                .then(heading.clone())
                .padded()
                .then(leaf.clone().or(environment).repeated())
                .then(just("\\end").ignore_then(heading.clone()).padded())
                .try_map(|(((metadata, name_begin), children), name_end), span| {
                    if name_begin != name_end {
                        Err(Simple::custom(span, "Environment not closed correctly"))
                    } else {
                        Ok(self.build_env(name_end, children, metadata))
                    }
                })
        })
        .boxed();

        let prelude = choice((leaf.clone(), environment.clone())).boxed();

        let prelude_in_inputs = recursive(|prelude_in_inputs| {
            metadata
                .clone()
                .then_ignore(just("% TEXLA FILE BEGIN"))
                .then(heading.clone().padded())
                .then(prelude.clone().or(prelude_in_inputs).repeated())
                .then_ignore(just("% TEXLA FILE END").padded())
                .map(|((metadata, path), children)| self.build_file(path, children, metadata))
        })
        .boxed();

        let prelude_any = prelude_in_inputs
            .clone()
            .or(prelude.clone())
            .repeated()
            .padded()
            .boxed();

        // TODO extract method
        let subsection = metadata
            .clone()
            .then_ignore(just("\\subsection"))
            .then(heading.clone())
            .then_ignore(newline())
            .then(prelude_any.clone())
            .map(|((metadata, heading), blocks)| {
                self.build_segment(
                    heading.clone(),
                    blocks,
                    format!("\\subsection{{{heading}}}"),
                    metadata,
                )
            })
            .boxed();

        let subsections_in_inputs = recursive(|subsections_in_inputs| {
            metadata
                .clone()
                .then_ignore(just("% TEXLA FILE BEGIN"))
                .then(heading.clone().padded())
                .then(prelude_any.clone())
                .then(subsections_in_inputs.or(subsection.clone()).repeated())
                .then_ignore(just("% TEXLA FILE END").padded())
                .map(|(((metadata, path), mut prelude), mut children)| {
                    self.build_file(
                        path,
                        {
                            prelude.append(&mut children);
                            prelude
                        },
                        metadata,
                    )
                })
        })
        .boxed();

        let subsection_any = subsections_in_inputs
            .clone()
            .or(subsection.clone())
            .padded()
            .boxed();

        let section = metadata
            .clone()
            .then_ignore(just("\\section"))
            .then(heading.clone())
            .then_ignore(newline())
            .then(prelude_any.clone())
            .then(subsection_any.clone().repeated())
            .map(|(((metadata, heading), mut blocks), mut subsections)| {
                blocks.append(&mut subsections);
                self.build_segment(
                    heading.clone(),
                    blocks,
                    format!("\\section{{{heading}}}"),
                    metadata,
                )
            })
            .boxed();

        let sections_in_inputs = recursive(|sections_in_inputs| {
            metadata
                .clone()
                .then_ignore(just("% TEXLA FILE BEGIN"))
                .then(heading.clone().padded())
                .then(prelude_any.clone())
                .then(sections_in_inputs.or(section.clone()).repeated())
                .then_ignore(just("% TEXLA FILE END").padded())
                .map(|(((metadata, path), mut prelude), mut children)| {
                    self.build_file(
                        path,
                        {
                            prelude.append(&mut children);
                            prelude
                        },
                        metadata,
                    )
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
            .then(metadata.clone())
            .then_ignore(just::<_, _, Simple<char>>("\\begin{document}").padded())
            .then(root_children.clone())
            .then_ignore(just("\\end{document}"))
            .map(|((preamble, metadata), (mut leaves, mut segments))| {
                self.build_document(
                    preamble.unwrap_or(String::new()),
                    String::new(),
                    {
                        leaves.append(&mut segments);
                        leaves
                    },
                    metadata,
                )
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
