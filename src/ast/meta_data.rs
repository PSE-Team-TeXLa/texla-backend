use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MetaData {
    #[serde(rename = "meta_data")]
    pub(crate) data: HashMap<String, String>,
}

// TODO: maybe &str is sufficient here
impl MetaData {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn edit_meta_data(&mut self, new_meta_data: HashMap<String, String>) {
        for (key, value) in new_meta_data {
            self.data.insert(key, value);
        }
    }

    pub fn delete_meta_data(&mut self, key: String) {
        self.data.remove(&key);
    }
}
