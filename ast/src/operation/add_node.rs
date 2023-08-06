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

// Position is in operation.rs
//  has: parent:UUid, after_sibling:Option<Uuid>

//socket perform_and_check_operation() -> ast.execute(operation) -> single_string -> from_latex -> stringify

// Need to crate a simple ast, for example from /test_resources/latex/simple.tex
// Then, add a subsubsection after \subsection{Subtitle}
// look for subsubsection in ast, as well as for correct parent reference

mod tests {
    use crate::node::{ExpandableData, NodeType};
    use crate::operation::add_node::AddNode;
    use crate::operation::{Operation, Position};
    use crate::parser::parse_latex;
    use crate::texla_ast::TexlaAst;
    use crate::uuid_provider::Uuid;
    use crate::Ast;
    use std::fs;

    fn lf(s: String) -> String {
        s.replace("\r\n", "\n")
    }

    fn test_add_node() {
        let latex = fs::read_to_string("../test_resources/latex/simple.tex").unwrap();
        let mut ast = parse_latex(latex.clone()).expect("Valid Latex");

        let subsection_uuid =
            find_uuid_by_content(&ast, "\\subsection{Subtitle}").expect("Failed to find");

        let raw_latex = "\\subsubsection{New Subsubsection}";

        let position = Position {
            parent: subsection_uuid,
            after_sibling: None,
        };

        let operation = Box::new(AddNode {
            destination: position,
            raw_latex: raw_latex.to_string(),
        });

        ast.execute(operation).expect("should succeed");

        let new_subsection_uuid = find_uuid_by_content(&ast, raw_latex);

        let parent_node_ref = ast
            .get_node(new_subsection_uuid.expect(""))
            .lock()
            .unwrap()
            .parent
            .as_ref()
            .expect("New subsubsection should have a parent")
            .upgrade()
            .expect("Parent node should be valid");

        assert!(new_subsection_uuid.is_some(), "subsection added");
    }

    fn find_uuid_by_content(ast: &TexlaAst, content: &str) -> Option<Uuid> {
        for (uuid, node_ref_weak) in &ast.portal {
            let node_ref = node_ref_weak.upgrade().expect("Invalid weak reference");
            let node = node_ref.lock().unwrap();
            if let NodeType::Expandable { data, .. } = &node.node_type {
                if let ExpandableData::Dummy {
                    before_children, ..
                } = data
                {
                    if before_children == content {
                        return Some(*uuid);
                    }
                }
            }
        }
        None
    }
}
