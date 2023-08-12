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
    use super::*;
    use crate::operation::test::find_uuid_by_content;
    use crate::operation::test::get_node_and_count_children;
    use crate::parser::parse_latex;
    use std::fs;

    // Move "another Block of text aaa" leaf to \subsection{Subtitle} behind Something Leaf

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
        let parent_uuid =
            find_uuid_by_content(&ast, subsection_to_be_moved_to_content).expect("Failed to find");
        let after_sibling_uuid =
            find_uuid_by_content(&ast, subsection_first_child_content).expect("Failed to find");

        let title1_children_count_before =
            get_node_and_count_children(&ast, section_to_be_moved_from_content);
        let subtitle_children_count_before =
            get_node_and_count_children(&ast, subsection_to_be_moved_to_content);

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

        let title1_children_count_after =
            get_node_and_count_children(&ast, section_to_be_moved_from_content);
        let subtitle_children_count_after =
            get_node_and_count_children(&ast, subsection_to_be_moved_to_content);

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

    // Move subsection subtitle from title 1 to title 2
    #[test]
    fn test_move_expandable() {
        let subsection_to_be_moved = "\\subsection{Subtitle}";
        let section_to_be_moved_from_content = "\\section{Title1}";
        let section_to_be_moved_to_content = "\\section{Title2}";

        let original_latex_single_string =
            fs::read_to_string("../test_resources/latex/simple_for_operation_testing.tex").unwrap();
        let mut ast = parse_latex(original_latex_single_string.clone()).expect("Valid Latex");

        let target_uuid =
            find_uuid_by_content(&ast, subsection_to_be_moved).expect("Failed to find");
        let parent_uuid =
            find_uuid_by_content(&ast, section_to_be_moved_to_content).expect("Failed to find");

        let title1_children_count_before =
            get_node_and_count_children(&ast, section_to_be_moved_from_content);
        let title2_children_count_before =
            get_node_and_count_children(&ast, section_to_be_moved_to_content);
        let subtitle_children_count_before =
            get_node_and_count_children(&ast, subsection_to_be_moved);

        let position = Position {
            parent: parent_uuid,
            after_sibling: None,
        };

        let operation = Box::new(MoveNode {
            target: target_uuid,
            destination: position,
        });

        ast.execute(operation).expect("Should succeed");
        // reparse
        let new_latex_single_string = ast.to_latex(Default::default()).unwrap();
        ast = parse_latex(new_latex_single_string.clone()).expect("Valid Latex");

        let title1_children_count_after =
            get_node_and_count_children(&ast, section_to_be_moved_from_content);
        let title2_children_count_after =
            get_node_and_count_children(&ast, section_to_be_moved_to_content);
        let subtitle_children_count_after =
            get_node_and_count_children(&ast, subsection_to_be_moved);

        assert_eq!(
            title1_children_count_before - 1,
            title1_children_count_after,
            "The parent node should have one more child after the operation"
        );

        assert_eq!(
            title2_children_count_before + 1,
            title2_children_count_after,
            "The parent node should have one less child after the operation"
        );

        assert_eq!(
            subtitle_children_count_before, subtitle_children_count_after,
            "The subtitle node should have the same number of children after the operation"
        );
    }
}
