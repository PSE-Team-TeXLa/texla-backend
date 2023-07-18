use serde::Serialize;

pub type Uuid = u64;

static JS_MAX_SAFE_INTEGER: Uuid = 2u64.pow(53);
static MAX_UUID: Uuid = JS_MAX_SAFE_INTEGER;

pub trait UuidProvider {
    fn new_uuid(&mut self) -> Uuid;
    // TODO use Option<Uuid> as return type as in spec?
}

#[derive(Debug, Clone, Serialize)]
pub struct TexlaUuidProvider {
    highest_uuid: Uuid,
}

impl UuidProvider for TexlaUuidProvider {
    fn new_uuid(&mut self) -> Uuid {
        self.highest_uuid += 1;
        if self.highest_uuid > MAX_UUID {
            // we do not expect to have this many nodes (2^53 = 9e15)
            panic!("UUID overflow")
        }
        self.highest_uuid
    }
}

impl TexlaUuidProvider {
    pub fn new() -> Self {
        TexlaUuidProvider { highest_uuid: 0 }
    }
}
