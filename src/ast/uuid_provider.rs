pub type Uuid = u64;

pub trait UuidProvider {
    // Linus: i don't know why there is an option in the spec
    fn new_uuid(&self) -> Uuid;
}

pub struct TexlaUuidProvider {}

impl UuidProvider for TexlaUuidProvider {
    fn new_uuid(&self) -> Uuid {
        todo!()
    }
}
