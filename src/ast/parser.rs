use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;

use chumsky::Parser;
use chumsky::prelude::*;
use chumsky::text::newline;

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
    //TODO Indentation Support
    fn new() -> Self {
        LatexParser {
            uuid_provider: RefCell::new(TexlaUuidProvider::new()),
            portal: RefCell::new(HashMap::new()),
        }
    }
    fn build_text(&self, text: String) -> NodeRef {
        Node::new_leaf(
            LeafData::Text { text },
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
        )
    }
    fn build_segment(&self, heading: String, children: Vec<NodeRef>) -> NodeRef {
        Node::new_expandable(
            ExpandableData::Segment { heading },
            children,
            self.uuid_provider.borrow_mut().deref_mut(),
            self.portal.borrow_mut().deref_mut(),
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
        )
    }
    fn parser(&self) -> impl Parser<char, NodeRef, Error = Simple<char>> + '_ {
        let word = filter(|char: &char| char.is_ascii_alphanumeric())
            .repeated()
            .at_least(1)
            .collect::<String>()
            .boxed();

        // TODO: why \\ and this should somehow use newline()
        // Und warum ist der Parser überhaupt zeilenweise? Die sind bis auf doppelte doch irrelevant
        let line = none_of("\r\n\\")
            .repeated()
            .at_least(1)
            .then_ignore(newline())
            .collect::<String>()
            .map(|mut line: String| {
                line.push('\n');
                line
            })
            .boxed();

        let block = line
            .clone()
            .repeated()
            .at_least(1)
            .collect::<String>()
            .then_ignore(newline().or_not())
            .map(|x: String| self.build_text(x))
            .boxed();

        // TODO: this should not be repetitive
        let subsection = just("\\subsection")
            .ignore_then(word.clone().delimited_by(just('{'), just('}')))
            .then_ignore(newline())
            .then(block.clone().repeated())
            .map(|(heading, blocks): (String, Vec<NodeRef>)| self.build_segment(heading, blocks))
            .boxed();

        let section = just("\\section")
            .ignore_then(word.clone().delimited_by(just('{'), just('}')))
            .then_ignore(newline())
            .then(block.clone().repeated())
            .then(subsection.clone().repeated())
            .map(
                |((heading, mut blocks), mut subsections): (
                    (String, Vec<NodeRef>),
                    Vec<NodeRef>,
                )| {
                    blocks.append(&mut subsections);
                    self.build_segment(heading, blocks)
                },
            )
            .boxed();

        let node = section
            .clone()
            .or(subsection.clone())
            .or(block.clone())
            .boxed();

        let document = just::<_, _, Simple<char>>("\\begin{document}")
            .then(newline())
            .ignore_then(node.clone().repeated())
            .then_ignore(just("\\end{document}"))
            .map(|children: Vec<NodeRef>| {
                self.build_document(String::new(), String::new(), children)
            })
            .then_ignore(end().padded())
            .boxed();
        document
    }
    fn find_highest_level(&self) -> impl Parser<char, u8, Error = Simple<char>> + '_ {
        take_until(just("\\section").or(just("\\subsection"))).map(
            |(_trash, keyword)| match keyword {
                "\\section" => 2,
                "\\subsection" => 3,
                _ => 7,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use crate::ast::Ast;
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
        let json = ast.to_json(Default::default()).unwrap();
        fs::write(Path::new("./latex_test_files/simple_latex.json"), json).unwrap();
    }
}
