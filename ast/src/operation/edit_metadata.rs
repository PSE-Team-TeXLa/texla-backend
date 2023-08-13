use std::collections::HashMap;

use serde::Deserialize;

use crate::errors::OperationError;
use crate::operation::Operation;
use crate::texla_ast::TexlaAst;
use crate::uuid_provider::Uuid;
use crate::Ast;

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
    use std::fs;

    use crate::operation::test::find_uuid_by_content;
    use crate::parser::parse_latex;

    use super::*;

    fn lf(s: String) -> String {
        s.replace("\r\n", "\n")
    }

    #[test]
    fn test_edit_metadata() {
        let section_containing_meta_data_raw_latex = "\\section{Title1}";

        let original_latex_single_string = lf(fs::read_to_string(
            "../test_resources/latex/latex_with_metadata/simple_with_metadata.tex",
        )
        .unwrap());
        let mut ast = parse_latex(original_latex_single_string).expect("Valid Latex");

        let mut target_uuid = find_uuid_by_content(&ast, section_containing_meta_data_raw_latex)
            .expect("Failed to find");

        let original_meta_data = ast
            .get_node(target_uuid)
            .lock()
            .unwrap()
            .meta_data
            .data
            .clone();

        let mut to_add_meta_data: HashMap<String, String> = HashMap::new();

        to_add_meta_data.insert(String::from("key3"), String::from("value3"));
        to_add_meta_data.insert(String::from("key4"), String::from("value4"));

        let operation = Box::new(EditMetadata {
            target: target_uuid,
            new: to_add_meta_data.clone(),
        });

        ast.execute(operation).expect("Should succeed");
        // reparse, default sets true for both comments and metadata
        let new_latex_single_string = ast.to_latex(Default::default()).unwrap();
        ast = parse_latex(new_latex_single_string).expect("Valid Latex");

        target_uuid = find_uuid_by_content(&ast, section_containing_meta_data_raw_latex)
            .expect("Failed to find");

        // Combine keys from original_meta_data and new_meta_data
        let mut expected_meta_data = original_meta_data;
        expected_meta_data.extend(to_add_meta_data.clone());

        let new_meta_data = ast
            .get_node(target_uuid)
            .lock()
            .unwrap()
            .meta_data
            .data
            .clone();

        assert_eq!(
            expected_meta_data, new_meta_data,
            "Metadata does not match expected metadata"
        );
    }
}
