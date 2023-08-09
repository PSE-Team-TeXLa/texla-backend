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
    use crate::node::{NodeRef, NodeType};
    use crate::operation::add_node::AddNode;
    use crate::operation::edit_node::EditNode;
    use crate::operation::Position;
    use crate::parser::parse_latex;
    use crate::texla_ast::TexlaAst;
    use crate::uuid_provider::Uuid;
    use crate::Ast;
    use std::fs;

    #[test]
    fn test_edit_node() {
        let source_latex = fs::read_to_string("../test_resources/latex/simple.tex").unwrap();
        let mut ast = parse_latex(source_latex.clone()).expect("Valid Latex");

        //let original_to_latex = ast.to_latex(Default::default());
        //println!(
        //    "original_to_latex: {}",
        //    original_to_latex.unwrap().to_string()
        //);

        let target_uuid = find_uuid_by_content(&ast, "\\section{Title1}").expect("Failed to find");

        let node_before = ast.get_node(target_uuid).clone();

        let raw_latex = "\\section{Title1New}";

        let operation = Box::new(EditNode {
            target: target_uuid,
            raw_latex: raw_latex.to_string(),
        });

        ast.execute(operation).expect("should succeed");

        let new_target_uuid =
            find_uuid_by_content(&ast, "\\section{Title1New}").expect("Failed to find");

        let node_after = ast.get_node(new_target_uuid).clone();
    }

    fn find_uuid_by_content(ast: &TexlaAst, content: &str) -> Option<Uuid> {
        find_uuid_by_content_recursive(&ast.root, content)
    }

    fn find_uuid_by_content_recursive(node_ref: &NodeRef, content: &str) -> Option<Uuid> {
        let node = node_ref.lock().unwrap();
        let current_raw_latex = &node.raw_latex.to_string();
        //println!("current_raw_latex: {}", current_raw_latex.to_string());

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
