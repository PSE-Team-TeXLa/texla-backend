use std::collections::HashMap;

use serde::Serialize;

// TODO: by now, this is a nitwit wrapper around a HashMap. Consider removing it.
#[derive(Debug, Serialize)]
pub struct MetaData {
    #[serde(rename = "meta_data")]
    pub(crate) data: HashMap<String, String>,
}

impl MetaData {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}
