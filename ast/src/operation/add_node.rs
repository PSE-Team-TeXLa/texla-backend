use std::sync::{Arc, Mutex};

use serde::Deserialize;

use crate::errors::OperationError;
use crate::meta_data::MetaData;
use crate::node::{ExpandableData, Node, NodeType};
use crate::operation::{Operation, Position};
use crate::texla_ast::TexlaAst;
use crate::uuid_provider::UuidProvider;
use crate::Ast;

/// Tries to add a node represented by `raw_latex` into the [Ast] at the given [Position].
#[derive(Deserialize, Debug)]
pub struct AddNode {
    pub destination: Position,
    pub raw_latex: String,
}

impl Operation<TexlaAst> for AddNode {
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), OperationError> {
        // create new node
        // TODO: maybe outsource node creation later
        let uuid = ast.uuid_provider.new_uuid();

        let new_node_ref = Arc::new(Mutex::new(Node {
            uuid,
            node_type: NodeType::Expandable {
                data: ExpandableData::Dummy {
                    before_children: self.raw_latex.clone(),
                    after_children: "".to_string(),
                },
                children: vec![],
            },
            meta_data: MetaData::new(),
            parent: Some(Arc::downgrade(&ast.get_node(self.destination.parent))),
            raw_latex: String::new(), // shouldn't matter since it gets re-parsed instantly
        }));

        // insert into ast
        ast.insert_node_at_position(new_node_ref, self.destination);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::test::find_uuid_by_content;
    use crate::operation::test::get_node_and_count_children;
    use crate::parser::parse_latex;
    use std::fs;

    #[test]
    fn test_add_node() {
        let subsection_to_be_added_to_raw_latex = "\\subsection{Subtitle}";
        let subsubsection_to_be_added_raw_latex = "\\subsubsection{Subsubtitle}";

        let original_latex_single_string =
            fs::read_to_string("../test_resources/latex/simple.tex").unwrap();
        let mut ast = parse_latex(original_latex_single_string.clone()).expect("Valid Latex");

        let subsection_uuid = find_uuid_by_content(&ast, subsection_to_be_added_to_raw_latex)
            .expect("Failed to find");

        let position = Position {
            parent: subsection_uuid,
            after_sibling: None,
        };

        let subsection_children_count_before =
            get_node_and_count_children(&ast, subsection_to_be_added_to_raw_latex);

        let operation = Box::new(AddNode {
            destination: position,
            raw_latex: subsubsection_to_be_added_raw_latex.to_string(),
        });

        ast.execute(operation).expect("should succeed");
        // reparse
        let new_latex_single_string = ast.to_latex(Default::default()).unwrap();
        ast = parse_latex(new_latex_single_string.clone()).expect("Valid Latex");

        let subsection_children_count_after =
            get_node_and_count_children(&ast, subsection_to_be_added_to_raw_latex);
        let subsubsection_children_count_after_adding =
            get_node_and_count_children(&ast, subsubsection_to_be_added_raw_latex);

        // subsection{subtitle} should now only have one child, namely subsubsection
        // all the children from before should now be contained in subsubsection
        assert_eq!(
            subsection_children_count_before - 1,
            subsection_children_count_after,
            "The parent node should have one more child after the operation"
        );

        assert_eq!(
            subsection_children_count_before, subsubsection_children_count_after_adding,
            "subsubsection should now contain the children of subsection"
        );

        assert!(!original_latex_single_string.contains(subsubsection_to_be_added_raw_latex));
        assert!(new_latex_single_string.contains(subsubsection_to_be_added_raw_latex));
    }
}
