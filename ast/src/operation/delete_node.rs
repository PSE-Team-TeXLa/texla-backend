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
    use super::*;
    use crate::operation::test::find_uuid_by_content;
    use crate::operation::test::get_node_and_count_children;
    use crate::parser::parse_latex;
    use std::fs;

    #[test]
    fn test_delete_node() {
        let subsection_name_to_be_deleted_raw_latex = "\\subsection{Subtitle}";
        let subsection_first_child_raw_latex = "another Block of text\naaaaa";
        let subsection_second_child_raw_latex = "jhhgghjg";
        let section_that_contains_to_be_deleted_subsection = "\\section{Title1}";
        let section_that_is_no_child_of_subsection_raw_latex = "\\section{Title2}";

        let original_latex_single_string =
            fs::read_to_string("../test_resources/latex/simple_for_operation_testing.tex").unwrap();
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
}
