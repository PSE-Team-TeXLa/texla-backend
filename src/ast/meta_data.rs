use std::collections::HashMap;

pub struct MetaData {
    meta_data: HashMap<String, String>,
}

impl MetaData {
    fn new() -> Self {
        MetaData {
            meta_data: HashMap::new(),
        }
    }

    fn edit_meta_data(&mut self, new_meta_data: HashMap<String, String>) {
        todo!()
    }

    fn delete_meta_data(&mut self, key: String) {
        todo!()
    }
}