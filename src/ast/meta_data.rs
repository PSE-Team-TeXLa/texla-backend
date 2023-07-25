use std::collections::HashMap;
use std::fmt::{format, write, Display, Formatter};
use std::string::String;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MetaData {
    #[serde(rename = "meta_data")]
    pub(crate) data: HashMap<String, String>,
}

// TODO: maybe &str is sufficient here
impl MetaData {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    fn edit_meta_data(&mut self, new_meta_data: HashMap<String, String>) {
        for (key, value) in new_meta_data {
            self.data.insert(key, value);
        }
    }

    fn delete_meta_data(&mut self, key: String) {
        self.data.remove(&key);
    }
}
impl Display for MetaData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = self
            .data
            .iter()
            .fold(String::new(), |mut string, (key, value)| {
                string.push_str(&format!("{key}: {value},"));
                string
            });
        write!(f, "({})", string)
    }
}
