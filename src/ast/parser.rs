use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::rc::Rc;

use chumsky::prelude::*;
use chumsky::Parser;

use crate::ast::meta_data::MetaData;
use crate::ast::node::{ExpandableData, LeafData, Node, NodeRef, NodeRefWeak, NodeType};
use crate::ast::texla_ast::TexlaAst;
use crate::ast::uuid_provider::{TexlaUuidProvider, Uuid};

#[derive(Clone)]
struct LatexParser {
    uuid_provider: RefCell<TexlaUuidProvider>,
    portal: RefCell<HashMap<Uuid, NodeRefWeak>>,
}
pub fn parse_latex(string: String) -> TexlaAst {
    let mut parser = LatexParser::new();
    let root = parser.parser().parse(string).unwrap();
    TexlaAst {
        portal: parser.portal.into_inner(),
        uuid_provider: parser.uuid_provider.into_inner(),
        root,
    }
}

impl LatexParser {
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
        let line = none_of("\r\n\\")
            .repeated()
            .at_least(1)
            .then_ignore(just("\n"))
            .collect::<String>()
            .map(|mut line| {
                line.push_str("\n");
                line
            });

        let block = line
            .clone()
            .repeated()
            .at_least(1)
            .collect::<String>()
            .then_ignore(just("\n").or_not())
            .map(|x| self.build_text(x));

        let section = just("\\section")
            .ignore_then(line.clone())
            .then(block.clone())
            .map(|(heading, child)| self.build_segment(heading, vec![child]));

        let subsection = just("\\subsection")
            .ignore_then(line.clone())
            .then(block.clone())
            .map(|(heading, child)| self.build_segment(heading, vec![child]));

        let node = section.clone().or(subsection.clone()).or(block.clone());

        let document = just::<_, _, Simple<char>>("\\begin{document}\n")
            .ignore_then(node.clone().repeated())
            .then_ignore(just("\\end{document}"))
            .map(|children| self.build_document(String::new(), String::new(), children))
            .then_ignore(end());
        document
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::ast::parser::parse_latex;

    #[test]
    fn simple() {
        let latex = fs::read_to_string("simple_latex").unwrap();
        let ast = parse_latex(latex);
        println!("{:#?}", ast);
        assert!(1 + 1 == 2);
    }
}
