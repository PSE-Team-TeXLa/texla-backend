use std::cell::RefCell;

pub type Uuid = u64;

pub trait UuidProvider {
    fn new_uuid(&mut self) -> Uuid;
}

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
    fn new() -> TexlaUuidProvider {
        TexlaUuidProvider { highest_uuid: 0 }
    }
}
