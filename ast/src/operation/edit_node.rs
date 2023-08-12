use std::sync::{Arc, Mutex};

use serde::Deserialize;

use crate::errors::OperationError;
use crate::meta_data::MetaData;
use crate::node::{ExpandableData, Node, NodeType};
use crate::operation::Operation;
use crate::texla_ast::TexlaAst;
use crate::uuid_provider::Uuid;
use crate::Ast;

#[derive(Deserialize, Debug)]
pub struct EditNode {
    pub target: Uuid,
    pub raw_latex: String,
}

impl Operation<TexlaAst> for EditNode {
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), OperationError> {
        let node_ref = ast.get_node(self.target);

        // create new node from old node
        let new_node_ref = {
            let node = node_ref.lock().unwrap();
            let node_meta_data_map = &node.meta_data.data;
            let node_parent = &node.parent;

            let mut parts = self.raw_latex.split("...");
            let before_children = parts.next().unwrap_or("").to_string();
            let after_children = parts.next().unwrap_or("").to_string();

            Arc::new(Mutex::new(Node {
                uuid: self.target,
                node_type: NodeType::Expandable {
                    data: ExpandableData::Dummy {
                        before_children,
                        after_children,
                    },
                    children: match &node.node_type {
                        NodeType::Expandable { children, .. } => children.clone(), // copies children from old node
                        NodeType::Leaf { .. } => {
                            vec![]
                        }
                    },
                },
                meta_data: MetaData {
                    data: node_meta_data_map.clone(),
                },
                parent: node_parent.clone(),
                raw_latex: String::new(), // shouldn't matter since it gets re-parsed instantly
            }))
        };

        if node_ref.lock().unwrap().parent.as_ref().is_some() {
            // update node in ast
            let position = ast.remove_node(&node_ref);
            ast.insert_node_at_position(new_node_ref, position);
        } else {
            // if parent is None, then this node is the root node
            ast.root = new_node_ref;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::test::find_uuid_by_content;
    use crate::parser::parse_latex;
    use std::fs;

    #[test]
    fn test_edit_node() {
        let original_section_raw_latex = "\\section{Title1}";
        let changed_section_raw_latex = "\\section{EditedTitle}";

        let original_latex_single_string =
            fs::read_to_string("../test_resources/latex/simple_for_operation_testing.tex").unwrap();
        let mut ast = parse_latex(original_latex_single_string.clone()).expect("Valid Latex");

        let mut target_uuid =
            find_uuid_by_content(&ast, original_section_raw_latex).expect("Failed to find");

        let node_before = ast.get_node(target_uuid).clone();

        let operation = Box::new(EditNode {
            target: target_uuid,
            raw_latex: changed_section_raw_latex.to_string(),
        });

        ast.execute(operation).expect("should succeed");

        // reparse LaTeX
        let new_latex_single_string = ast.to_latex(Default::default());
        let new_latex_single_string_unwrapped = new_latex_single_string.unwrap();
        ast = parse_latex(new_latex_single_string_unwrapped.clone()).expect("should succeed");

        target_uuid =
            find_uuid_by_content(&ast, changed_section_raw_latex).expect("Failed to find");

        let node_after = ast.get_node(target_uuid).clone();

        assert_ne!(
            node_before.lock().unwrap().uuid,
            node_after.lock().unwrap().uuid,
            "UUID Should have changed"
        );

        // Old content should be present in the original_latex_single_string and absent in new_latex_single_string
        assert!(
            original_latex_single_string.contains(original_section_raw_latex),
            "The original LaTeX should contain '\\section{{Title1}}'"
        );
        assert!(
            !original_latex_single_string.contains(changed_section_raw_latex),
            "The edited LaTeX should not contain '\\section{{EditedTitle}}'"
        );

        // New content should be absent in the original_latex_single_string and present in new_latex_single_string
        assert!(
            !new_latex_single_string_unwrapped.contains(original_section_raw_latex),
            "The original LaTeX should not contain '\\section{{EditedTitle}}'"
        );
        assert!(
            new_latex_single_string_unwrapped.contains(changed_section_raw_latex),
            "The edited LaTeX should contain '\\section{{EditedTitle}}'"
        );
    }
}
