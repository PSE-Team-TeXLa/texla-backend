pub type Uuid = u64;

pub trait UuidProvider {
    // TODO use Option<Uuid> as return type as in spec?
    fn new_uuid(&self) -> Uuid;
}

pub struct TexlaUuidProvider {}

impl UuidProvider for TexlaUuidProvider {
    fn new_uuid(&self) -> Uuid {
        todo!()
    }
}
