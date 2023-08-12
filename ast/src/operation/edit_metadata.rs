use std::collections::HashMap;

use crate::Ast;
use serde::Deserialize;

use crate::errors::OperationError;
use crate::operation::Operation;
use crate::texla_ast::TexlaAst;
use crate::uuid_provider::Uuid;

#[derive(Deserialize, Debug)]
pub struct EditMetadata {
    pub target: Uuid,
    pub new: HashMap<String, String>,
}

impl Operation<TexlaAst> for EditMetadata {
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), OperationError> {
        let node_ref = ast.get_node(self.target);
        let mut node = node_ref.lock().unwrap();
        node.meta_data.data.extend(self.new.clone());

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
    use std::collections::HashMap;
    use std::fs;
    use std::hash::Hash;

    #[test]
    fn test_edit_metadata() {
        let original_latex_single_string =
            fs::read_to_string("../test_resources/latex/simple.tex").unwrap();
        let mut ast = parse_latex(original_latex_single_string.clone()).expect("Valid Latex");

        let mut meta_data: HashMap<String, String> = HashMap::new();
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
