use std::cell::RefCell;

use serde::Serialize;

pub type Uuid = u64;

pub trait UuidProvider {
    fn new_uuid(&mut self) -> Uuid;
}

#[derive(Debug, Clone, Serialize)]
pub struct TexlaUuidProvider {
    highest_uuid: Uuid,
}

impl UuidProvider for TexlaUuidProvider {
    fn new_uuid(&mut self) -> Uuid {
        self.highest_uuid += 1;
        self.highest_uuid
    }
}

impl TexlaUuidProvider {
    pub fn new() -> Self {
        TexlaUuidProvider { highest_uuid: 0 }
    }
}
