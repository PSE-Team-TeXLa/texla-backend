use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MetaData {
    #[serde(rename = "meta_data")]
    pub(crate) data: HashMap<String, String>,
}

impl MetaData {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    fn edit_meta_data(&mut self, new_meta_data: HashMap<String, String>) {
        todo!()
    }

    fn delete_meta_data(&mut self, key: String) {
        todo!()
    }
}
