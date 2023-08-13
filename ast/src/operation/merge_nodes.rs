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
    use std::fs;

    use crate::operation::test::find_uuid_by_content;
    use crate::operation::test::get_node_and_count_children;
    use crate::parser::parse_latex;

    use super::*;

    fn lf(s: String) -> String {
        s.replace("\r\n", "\n")
    }

    #[test]
    fn test_merge_nodes() {
        let subsection_with_children_to_be_merged_content = "\\subsection{Subtitle}";
        let subsection_first_child_content = "another Block of text\naaaaa";
        let leaf_node_to_be_merged_content = "jhhgghjg";

        let mut expected_merged_content = String::from(subsection_first_child_content);
        expected_merged_content.push_str(leaf_node_to_be_merged_content);

        let original_latex_single_string = lf(fs::read_to_string(
            "../test_resources/latex/simple_for_operation_testing.tex",
        )
        .unwrap());
        let mut ast = parse_latex(original_latex_single_string.clone()).expect("Valid Latex");

        let children_count_before =
            get_node_and_count_children(&ast, subsection_with_children_to_be_merged_content);

        let target_uuid =
            find_uuid_by_content(&ast, leaf_node_to_be_merged_content).expect("Failed to find");

        let operation = Box::new(MergeNodes {
            second_node: target_uuid,
        });

        ast.execute(operation).expect("should succeed");
        // reparse
        let new_latex_single_string = ast.to_latex(Default::default()).unwrap();
        ast = parse_latex(new_latex_single_string.clone()).expect("Valid Latex");

        let children_count_after =
            get_node_and_count_children(&ast, subsection_with_children_to_be_merged_content);

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
}
