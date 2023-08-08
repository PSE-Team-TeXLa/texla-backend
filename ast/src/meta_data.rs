use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::string::String;

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
