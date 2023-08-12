use crate::Ast;
use serde::Deserialize;

use crate::errors::OperationError;
use crate::operation::Operation;
use crate::texla_ast::TexlaAst;
use crate::uuid_provider::Uuid;

#[derive(Deserialize, Debug)]
pub struct DeleteMetadata {
    pub target: Uuid,
    pub key: String,
}

impl Operation<TexlaAst> for DeleteMetadata {
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), OperationError> {
        let node_ref = ast.get_node(self.target);
        let mut node = node_ref.lock().unwrap();
        node.meta_data.data.remove(&self.key);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::node::{NodeRef, NodeType};
    use crate::operation::delete_metadata::DeleteMetadata;
    use crate::operation::edit_metadata::EditMetadata;
    use crate::parser::parse_latex;
    use crate::texla_ast::TexlaAst;
    use crate::uuid_provider::Uuid;
    use crate::Ast;
    use std::collections::HashMap;
    use std::fs;

    #[test]
    fn test_delete_metadata() {
        let section_containing_meta_data_raw_latex = "\\section{Title1}";
        let key_to_delete_name = "key1";

        let original_latex_single_string =
            fs::read_to_string("../test_resources/latex/simple_with_metadata.tex").unwrap();
        let mut ast = parse_latex(original_latex_single_string.clone()).expect("Valid Latex");

        let mut target_uuid = find_uuid_by_content(&ast, section_containing_meta_data_raw_latex)
            .expect("Failed to find");

        let original_meta_data = ast
            .get_node(target_uuid)
            .lock()
            .unwrap()
            .meta_data
            .data
            .clone();

        let operation = Box::new(DeleteMetadata {
            target: target_uuid,
            key: key_to_delete_name.to_string(),
        });

        ast.execute(operation).expect("Should succeed");
        // reparse, default sets true for both comments and metadata
        let new_latex_single_string = ast.to_latex(Default::default()).unwrap();
        ast = parse_latex(new_latex_single_string.clone()).expect("Valid Latex");

        target_uuid = find_uuid_by_content(&ast, section_containing_meta_data_raw_latex)
            .expect("Failed to find");

        let new_meta_data = ast
            .get_node(target_uuid)
            .lock()
            .unwrap()
            .meta_data
            .data
            .clone();

        assert_eq!(
            original_meta_data.len() - 1,
            new_meta_data.len(),
            "New metadata should have one less key in comparison to the original"
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
