use serde::Deserialize;

use crate::errors::OperationError;
use crate::operation::{Operation, Position};
use crate::texla_ast::TexlaAst;
use crate::uuid_provider::Uuid;
use crate::Ast;

#[derive(Deserialize, Debug)]
pub struct MoveNode {
    pub target: Uuid,
    pub destination: Position,
}

impl Operation<TexlaAst> for MoveNode {
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), OperationError> {
        let node_ref = ast.get_node(self.target);
        ast.remove_node(&node_ref);
        ast.insert_node_at_position(node_ref.clone(), self.destination);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::node::{NodeRef, NodeType};
    use crate::operation::delete_node::DeleteNode;
    use crate::operation::move_node::MoveNode;
    use crate::operation::Position;
    use crate::parser::parse_latex;
    use crate::texla_ast::TexlaAst;
    use crate::uuid_provider::Uuid;
    use crate::Ast;
    use std::fs;

    //move "another Block of text aaa" leaf to \subsection{Subtitle} behind Something Leaf

    #[test]
    fn test_move_leaf() {
        let subsection_to_be_moved_to_content = "\\subsection{Subtitle}";
        let section_to_be_moved_from_content = "\\section{Title1}";
        let subsection_first_child_content = "another Block of text\naaaaa";
        let subsection_second_child_content = "jhhgghjg";
        let leaf_to_be_moved_content = "Something";

        let original_latex_single_string =
            fs::read_to_string("../test_resources/latex/simple.tex").unwrap();
        let mut ast = parse_latex(original_latex_single_string.clone()).expect("Valid Latex");

        let target_uuid =
            find_uuid_by_content(&ast, leaf_to_be_moved_content).expect("Failed to find");
        let mut parent_uuid =
            find_uuid_by_content(&ast, subsection_to_be_moved_to_content).expect("Failed to find");
        let after_sibling_uuid =
            find_uuid_by_content(&ast, subsection_first_child_content).expect("Failed to find");
        let mut title1_node_uuid =
            find_uuid_by_content(&ast, section_to_be_moved_from_content).expect("Failed to find");

        let title1_node_before = ast.get_node(title1_node_uuid);
        let title1_children_count_before = match &title1_node_before.lock().unwrap().node_type {
            NodeType::Expandable { children, .. } => children.len(),
            _ => panic!("Parent node should be of type Expandable"),
        };

        let subtitle_node_before = ast.get_node(parent_uuid);
        let subtitle_children_count_before = match &subtitle_node_before.lock().unwrap().node_type {
            NodeType::Expandable { children, .. } => children.len(),
            _ => panic!("Parent node should be of type Expandable"),
        };

        let position = Position {
            parent: parent_uuid,
            after_sibling: Option::from(after_sibling_uuid),
        };

        let operation = Box::new(MoveNode {
            target: target_uuid,
            destination: position,
        });

        ast.execute(operation).expect("Should succeed");
        // reparse
        let new_latex_single_string = ast.to_latex(Default::default()).unwrap();
        ast = parse_latex(new_latex_single_string.clone()).expect("Valid Latex");

        title1_node_uuid =
            find_uuid_by_content(&ast, section_to_be_moved_from_content).expect("Failed to find");
        parent_uuid =
            find_uuid_by_content(&ast, subsection_to_be_moved_to_content).expect("Failed to find");

        let title1_node_after = ast.get_node(title1_node_uuid);
        let title1_children_count_after = match &title1_node_after.lock().unwrap().node_type {
            NodeType::Expandable { children, .. } => children.len(),
            _ => panic!("Parent node should be of type Expandable"),
        };

        let subtitle_node_after = ast.get_node(parent_uuid);
        let subtitle_children_count_after = match &subtitle_node_after.lock().unwrap().node_type {
            NodeType::Expandable { children, .. } => children.len(),
            _ => panic!("Parent node should be of type Expandable"),
        };

        assert_eq!(
            title1_children_count_before - 1,
            title1_children_count_after,
            "The title1 node should have one less child after the operation"
        );

        assert_eq!(
            subtitle_children_count_before + 1,
            subtitle_children_count_after,
            "The subtitle node should have one more child after the operation"
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
