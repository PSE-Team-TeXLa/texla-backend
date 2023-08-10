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
        let subsection_name_to_be_deleted = "\\subsection{Subtitle}";
        let subsection_first_child_content = "another Block of text\naaaaa";
        let subsection_second_child_content = "jhhgghjg";
        let section_that_is_no_child_of_subsection_content = "\\section{Title2}";

        let original_latex_single_string =
            fs::read_to_string("../test_resources/latex/simple.tex").unwrap();
        let mut ast = parse_latex(original_latex_single_string.clone()).expect("Valid Latex");

        let target_uuid =
            find_uuid_by_content(&ast, subsection_name_to_be_deleted).expect("Failed to find");

        let operation = Box::new(DeleteNode {
            target: target_uuid,
        });

        ast.execute(operation).expect("should succeed");

        // reparse
        let new_latex_single_string = ast.to_latex(Default::default()).unwrap();

        assert!(original_latex_single_string.contains(subsection_name_to_be_deleted));
        assert!(original_latex_single_string.contains(subsection_first_child_content));
        assert!(original_latex_single_string.contains(subsection_second_child_content));
        assert!(
            original_latex_single_string.contains(section_that_is_no_child_of_subsection_content)
        );

        assert!(!new_latex_single_string.contains(subsection_name_to_be_deleted));
        assert!(!new_latex_single_string.contains(subsection_name_to_be_deleted));
        assert!(!new_latex_single_string.contains(subsection_name_to_be_deleted));
        assert!(new_latex_single_string.contains(section_that_is_no_child_of_subsection_content));
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
