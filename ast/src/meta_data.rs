use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::string::String;

use serde::Serialize;

/// A wrapper around a [HashMap<String, String>].
/// Empty string values are the same as not having this key value pair at all.
/// The normal form is not having it, resulting in non-empty values.
#[derive(Debug, Serialize)]
pub(crate) struct MetaData {
    #[serde(rename = "meta_data")]
    pub(crate) data: HashMap<String, String>,
}

impl MetaData {
    pub(crate) fn new() -> Self {
        let mut this = Self {
            data: HashMap::new(),
        };
        this.normalize();
        this
    }

    pub(crate) fn normalize(&mut self) {
        self.data.retain(|_, value| !value.is_empty());
    }

    pub(crate) fn edit(&mut self, new_data: HashMap<String, String>) {
        self.data.extend(new_data);
        self.normalize();
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
