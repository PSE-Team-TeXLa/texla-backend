use serde::Deserialize;

use crate::errors::OperationError;
use crate::operation::Operation;
use crate::texla_ast::TexlaAst;
use crate::uuid_provider::Uuid;

/// Tries to delete a key-value pair from the Metadata Hashmap of some Node.
/// The Node is specified by its `target` Uuid, the key value pair is specified by its `key`.
/// This Struct is a Strategy. It can be created explicitly and should be used on an Ast via the `execute_on()` method.
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
    use std::fs;

    use crate::operation::test::find_uuid_by_content;
    use crate::parser::parse_latex;
    use crate::Ast;

    use super::*;

    fn lf(s: String) -> String {
        s.replace("\r\n", "\n")
    }

    #[test]
    fn test_delete_metadata() {
        let section_containing_meta_data_raw_latex = "\\section{Title1}";
        let key_to_delete_name = "key1";

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

        let operation = Box::new(DeleteMetadata {
            target: target_uuid,
            key: key_to_delete_name.to_string(),
        });

        ast.execute(operation).expect("Should succeed");
        // reparse, default sets true for both comments and metadata
        let new_latex_single_string = ast.to_latex(Default::default()).unwrap();
        ast = parse_latex(new_latex_single_string).expect("Valid Latex");

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
}
