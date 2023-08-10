use serde::Deserialize;

use crate::errors::OperationError;
use crate::node::{LeafData, NodeType};
use crate::operation::Operation;
use crate::texla_ast::TexlaAst;
use crate::uuid_provider::Uuid;
use crate::Ast;

#[derive(Deserialize, Debug)]
pub struct MergeNodes {
    pub second_node: Uuid,
}

impl Operation<TexlaAst> for MergeNodes {
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), OperationError> {
        let second_node_ref = ast.get_node(self.second_node);
        let latex = {
            let second_node = second_node_ref.lock().unwrap();
            match &second_node.node_type {
                NodeType::Leaf {
                    data: LeafData::Text { text },
                } => text.clone(),
                _ => {
                    return Err(OperationError {
                        message: "only Text nodes can be merged".to_string(),
                    });
                }
            }
        };

        let first_uuid = ast
            .remove_node(&second_node_ref)
            .after_sibling
            .ok_or(OperationError {
                message: "no predecessor found to merge into".to_string(),
            })?;
        let first_node_ref = ast.get_node(first_uuid);
        let mut first_node = first_node_ref.lock().unwrap();

        match &mut first_node.node_type {
            NodeType::Leaf {
                data: LeafData::Text { text },
            } => {
                text.push_str(&format!("\n{}", latex.as_str()));
            }
            _ => {
                return Err(OperationError {
                    message: "only Text nodes can be merged".to_string(),
                })
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::node::{NodeRef, NodeType};
    use crate::operation::merge_nodes::MergeNodes;
    use crate::parser::parse_latex;
    use crate::texla_ast::TexlaAst;
    use crate::uuid_provider::Uuid;
    use crate::Ast;
    use std::fs;

    #[test]
    fn test_merge_nodes() {
        let subsection_with_children_to_be_merged_content = "\\subsection{Subtitle}";
        let subsection_first_child_content = "another Block of text\naaaaa";
        let leaf_node_to_be_merged_content = "jhhgghjg";

        let mut expected_merged_content = String::from(subsection_first_child_content);
        expected_merged_content.push_str(leaf_node_to_be_merged_content);

        let original_latex_single_string =
            fs::read_to_string("../test_resources/latex/simple.tex").unwrap();
        let mut ast = parse_latex(original_latex_single_string.clone()).expect("Valid Latex");

        // count children
        let mut subsection_uuid =
            find_uuid_by_content(&ast, subsection_with_children_to_be_merged_content)
                .expect("Failed to find");
        let parent_node_before = ast.get_node(subsection_uuid);
        let children_count_before = match &parent_node_before.lock().unwrap().node_type {
            NodeType::Expandable { children, .. } => children.len(),
            _ => panic!("Parent node should be of type Expandable"),
        };

        let target_uuid =
            find_uuid_by_content(&ast, leaf_node_to_be_merged_content).expect("Failed to find");

        let operation = Box::new(MergeNodes {
            second_node: target_uuid,
        });

        ast.execute(operation).expect("should succeed");
        // reparse
        let new_latex_single_string = ast.to_latex(Default::default()).unwrap();
        ast = parse_latex(new_latex_single_string.clone()).expect("Valid Latex");

        subsection_uuid = find_uuid_by_content(&ast, subsection_with_children_to_be_merged_content)
            .expect("Failed to find");

        let parent_node_after = ast.get_node(subsection_uuid);
        let children_count_after = match &parent_node_after.lock().unwrap().node_type {
            NodeType::Expandable { children, .. } => children.len(),
            _ => panic!("Parent node should be of type Expandable"),
        };

        assert!(
            original_latex_single_string.contains(subsection_with_children_to_be_merged_content)
        );
        assert!(original_latex_single_string.contains(subsection_first_child_content));
        assert!(original_latex_single_string.contains(leaf_node_to_be_merged_content));

        assert!(new_latex_single_string.contains(subsection_with_children_to_be_merged_content));
        assert!(new_latex_single_string.contains(subsection_first_child_content));
        assert!(new_latex_single_string.contains(leaf_node_to_be_merged_content));

        assert_eq!(
            children_count_before - 1,
            children_count_after,
            "The parent node should have one more child after the operation"
        );
    }

    fn find_uuid_by_content(ast: &TexlaAst, content: &str) -> Option<Uuid> {
        find_uuid_by_content_recursive(&ast.root, content)
    }

    fn find_uuid_by_content_recursive(node_ref: &NodeRef, content: &str) -> Option<Uuid> {
        let node = node_ref.lock().unwrap();
        let current_raw_latex = &node.raw_latex.to_string();

        // Check if the raw_latex of the current node matches the content
        if current_raw_latex.contains(content) {
            return Some(node.uuid);
        }

        // If not, continue the traversal based on the node type
        match &node.node_type {
            NodeType::Expandable { children, .. } => {
                for child_ref in children {
                    if let Some(uuid) = find_uuid_by_content_recursive(child_ref, content) {
                        return Some(uuid);
                    }
                }
            }
            NodeType::Leaf { .. } => {
                // For Leaf nodes, we've already checked the raw_latex above.
                // So, there's no need for additional checks here.
            }
        }
        None
    }
}
