use std::sync::{Arc, Mutex};

use serde::Deserialize;

use crate::errors::OperationError;
use crate::meta_data::MetaData;
use crate::node::{ExpandableData, Node, NodeType};
use crate::operation::{Operation, Position};
use crate::texla_ast::TexlaAst;
use crate::uuid_provider::UuidProvider;
use crate::Ast;

/// Tries to add a node represented by `raw_latex` into the [Ast] at the given [Position].
#[derive(Deserialize, Debug)]
pub struct AddNode {
    pub destination: Position,
    pub raw_latex: String,
}

impl Operation<TexlaAst> for AddNode {
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), OperationError> {
        // create new node
        // TODO: maybe outsource node creation later
        let uuid = ast.uuid_provider.new_uuid();

        let new_node_ref = Arc::new(Mutex::new(Node {
            uuid,
            node_type: NodeType::Expandable {
                data: ExpandableData::Dummy {
                    before_children: self.raw_latex.clone(),
                    after_children: "".to_string(),
                },
                children: vec![],
            },
            meta_data: MetaData::new(),
            parent: Some(Arc::downgrade(&ast.get_node(self.destination.parent))),
            raw_latex: String::new(), // shouldn't matter since it gets re-parsed instantly
        }));

        // insert into ast
        ast.insert_node_at_position(new_node_ref, self.destination);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::node::{NodeRef, NodeType};
    use crate::operation::add_node::AddNode;
    use crate::operation::Position;
    use crate::parser::parse_latex;
    use crate::texla_ast::TexlaAst;
    use crate::uuid_provider::Uuid;
    use crate::Ast;
    use std::fs;

    #[test]
    fn test_add_node() {
        let original_latex = fs::read_to_string("../test_resources/latex/simple.tex").unwrap();
        let mut ast = parse_latex(original_latex.clone()).expect("Valid Latex");

        let subsection_uuid =
            find_uuid_by_content(&ast, "\\subsection{Subtitle}").expect("Failed to find");

        let position = Position {
            parent: subsection_uuid,
            after_sibling: None,
        };

        // 1. Get the parent node and count its children before the operation
        let parent_node_before = ast.get_node(subsection_uuid);

        //let raw_latex_parent = parent_node_before.lock().unwrap().raw_latex.to_string();
        //println!("raw latex: {}", raw_latex_parent);

        let children_count_before = match &parent_node_before.lock().unwrap().node_type {
            NodeType::Expandable { children, .. } => children.len(),
            _ => panic!("Parent node should be of type Expandable"),
        };

        let operation = Box::new(AddNode {
            destination: position,
            raw_latex: "".to_string(),
        });

        ast.execute(operation).expect("should succeed");

        // 2. Get the parent node and count its children after the operation
        let parent_node_after = ast.get_node(subsection_uuid);
        let children_count_after = match &parent_node_after.lock().unwrap().node_type {
            NodeType::Expandable { children, .. } => children.len(),
            _ => panic!("Parent node should be of type Expandable"),
        };

        // 3. Verify
        assert_eq!(
            children_count_before + 1,
            children_count_after,
            "The parent node should have one more child after the operation"
        );
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

//    #[test]
//    fn test_add_node() {
//        let original_latex = fs::read_to_string("../test_resources/latex/simple.tex").unwrap();
//        let mut ast = parse_latex(original_latex.clone()).expect("Valid Latex");
//
//        let subsection_uuid =
//            find_uuid_by_content(&ast, "\\subsection{Subtitle}").expect("Failed to find");
//
//        let raw_latex = "\\subsubsection{New Subsubsection}";
//
//        let position = Position {
//            parent: subsection_uuid,
//            after_sibling: None,
//        };
//
//        let operation = Box::new(AddNode {
//            destination: position,
//            raw_latex: raw_latex.to_string(),
//        });
//
//        //println!("raw_latex_in_operation: {}", operation.raw_latex);
//
//        ast.execute(operation).expect("should succeed");
//
//        let new_ast_to_latex = ast.to_latex(Default::default());
//
//        println!("new ast: {}", new_ast_to_latex.unwrap().to_string());
//
//        let new_sub_sub_section_uuid = find_uuid_by_content(&ast, raw_latex);
//
//        //let parent_node_uuid_from_subsubsection = ast
//        //    .get_node(new_sub_sub_section_uuid.expect(""))
//        //    .lock()
//        //    .unwrap()
//        //    .parent
//        //    .as_ref()
//        //    .expect("New subsubsection should have a parent")
//        //    .upgrade()
//        //    .expect("Parent node should be valid");
//
//        assert!(new_sub_sub_section_uuid.is_some(), "subsection added");
//    }
