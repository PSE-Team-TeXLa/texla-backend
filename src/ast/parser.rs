use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::os::unix::prelude::OsStringExt;

use chumsky::prelude::*;
use chumsky::text::newline;
use chumsky::Parser;

use crate::ast;
use crate::ast::node::{ExpandableData, LeafData, Node, NodeRef, NodeRefWeak};
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
    fn build_segment(&self, heading: String, children: Vec<NodeRef>, raw: String) -> NodeRef {
        Node::new_expandable(
            ExpandableData::Segment { heading },
            children,
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
            raw,
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
        )
    }
    fn parser(&self) -> impl Parser<char, NodeRef, Error = Simple<char>> + '_ {
        // let word = filter(|char: &char| char.is_ascii_alphanumeric())
        //     .repeated()
        //     .at_least(1)
        //     .collect::<String>()
        //     .boxed();

        // TODO find way to ignore \sectioning (use keyword?)
        let terminator = choice((
            just("\\section").rewind(),
            just("\\subsection").rewind(),
            just("\\begin").rewind(),
            just("\\end{document}").rewind(),
            just("\\includegraphics").rewind(),
            newline().then(newline()).to("\n\n"),
            // TODO recognize and consume also more than 2 newlines
        ));

        // TODO write parsers
        let environment = just(" ").map(|_| self.build_text(" ".to_string()));
        let input = just(" ").map(|_| self.build_text(" ".to_string()));

        let text_node = take_until(terminator)
            .try_map(|(v, _), span| {
                if !v.is_empty() {
                    return Ok(v);
                } else {
                    return Err(Simple::custom(span, format!("Found empty text")));
                }
            })
            .collect::<String>()
            .then_ignore(newline().or_not())
            .map(|x: String| self.build_text(x.trim_end().to_string()))
            .boxed();

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

        let leaf = choice((image.clone(), text_node.clone())).boxed();
        // TODO or image...

        let prelude = choice((environment.clone(), input.clone(), leaf.clone()));

        let heading = none_of("}").repeated().at_least(1).collect().boxed();

        // TODO extract method
        let subsection = just("\\subsection")
            .ignore_then(heading.clone().delimited_by(just('{'), just('}')))
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
            .ignore_then(heading.clone().delimited_by(just('{'), just('}')))
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

        let root_children = prelude.clone().repeated().then(choice((
            section.clone().repeated().at_least(1), //at_least used so this doesnt match with 0 occurrences and quit
            subsection.clone().repeated(), // Last Item should not have at_least to allow for empty document
                                           // TODO others
        )));

        // TODO implement preamble
        let document = just::<_, _, Simple<char>>("\\begin{document}")
            .then(newline())
            .or_not()
            .ignore_then(root_children.clone())
            .then_ignore(just("\\end{document}"))
            .map(|(mut leaves, mut segments)| {
                self.build_document(String::new(), String::new(), {
                    leaves.append(&mut segments);
                    leaves
                })
            })
            .then_ignore(end().padded())
            .boxed();
        document
    }
    fn find_highest_level(&self) -> impl Parser<char, i8, Error = Simple<char>> + '_ {
        take_until(just("\\section").or(just("\\subsection"))).map(
            |(_trash, keyword)| match keyword {
                "\\section" => 2,
                "\\subsection" => 3,
                _ => 8,
            },
        )
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
