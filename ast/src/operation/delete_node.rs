use serde::Deserialize;

use crate::errors::OperationError;
use crate::operation::Operation;
use crate::texla_ast::TexlaAst;
use crate::uuid_provider::Uuid;
use crate::Ast;

#[derive(Deserialize, Debug)]
pub struct DeleteNode {
    pub target: Uuid,
}

impl Operation<TexlaAst> for DeleteNode {
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), OperationError> {
        let node_ref = &ast.get_node(self.target);
        ast.remove_node(node_ref);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::node::{NodeRef, NodeType};
    use crate::operation::delete_node::DeleteNode;
    use crate::parser::parse_latex;
    use crate::texla_ast::TexlaAst;
    use crate::uuid_provider::Uuid;
    use crate::Ast;
    use std::fs;

    #[test]
    fn test_delete_node() {
        let subsection_name_to_be_deleted_raw_latex = "\\subsection{Subtitle}";
        let subsection_first_child_raw_latex = "another Block of text\naaaaa";
        let subsection_second_child_raw_latex = "jhhgghjg";
        let section_that_contains_to_be_deleted_subsection = "\\section{Title1}";
        let section_that_is_no_child_of_subsection_raw_latex = "\\section{Title2}";

        let original_latex_single_string =
            fs::read_to_string("../test_resources/latex/simple.tex").unwrap();
        let mut ast = parse_latex(original_latex_single_string.clone()).expect("Valid Latex");

        let target_uuid = find_uuid_by_content(&ast, subsection_name_to_be_deleted_raw_latex)
            .expect("Failed to find");

        let title1_children_count_before =
            get_node_and_count_children(&ast, section_that_contains_to_be_deleted_subsection);

        let operation = Box::new(DeleteNode {
            target: target_uuid,
        });

        ast.execute(operation).expect("should succeed");

        // reparse
        let new_latex_single_string = ast.to_latex(Default::default()).unwrap();
        ast = parse_latex(new_latex_single_string.clone()).expect("Valid Latex");

        let title1_children_count_after =
            get_node_and_count_children(&ast, section_that_contains_to_be_deleted_subsection);

        assert!(original_latex_single_string.contains(subsection_name_to_be_deleted_raw_latex));
        assert!(original_latex_single_string.contains(subsection_first_child_raw_latex));
        assert!(original_latex_single_string.contains(subsection_second_child_raw_latex));
        assert!(
            original_latex_single_string.contains(section_that_is_no_child_of_subsection_raw_latex)
        );

        assert!(!new_latex_single_string.contains(subsection_name_to_be_deleted_raw_latex));
        assert!(!new_latex_single_string.contains(subsection_name_to_be_deleted_raw_latex));
        assert!(!new_latex_single_string.contains(subsection_name_to_be_deleted_raw_latex));
        assert!(new_latex_single_string.contains(section_that_is_no_child_of_subsection_raw_latex));

        assert_eq!(
            title1_children_count_before - 1,
            title1_children_count_after,
            "Section Title should have one less child"
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

    fn get_node_and_count_children(ast: &TexlaAst, content: &str) -> usize {
        let node_uuid = find_uuid_by_content(ast, content).expect("Failed to find");
        let node_ref = ast.get_node(node_uuid);
        count_children_of_node(&node_ref)
    }

    fn count_children_of_node(node_ref: &NodeRef) -> usize {
        match &node_ref.lock().unwrap().node_type {
            NodeType::Expandable { children, .. } => children.len(),
            _ => 0, // Return 0 for non-Expandable nodes
        }
    }
}
