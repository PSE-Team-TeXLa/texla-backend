use serde::Serialize;

pub(crate) type Uuid = u64;

static JS_MAX_SAFE_INTEGER: Uuid = 2u64.pow(53);
static MAX_UUID: Uuid = JS_MAX_SAFE_INTEGER;

// TODO: subject to change (unsafe!)
static mut HIGHEST_UUID: Uuid = 0;

pub(crate) trait UuidProvider {
    fn new_uuid(&mut self) -> Uuid;
    // TODO use Option<Uuid> as return type as in spec?
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TexlaUuidProvider {
    highest_uuid: Uuid,
}

impl UuidProvider for TexlaUuidProvider {
    fn new_uuid(&mut self) -> Uuid {
        unsafe {
            HIGHEST_UUID += 1;
            HIGHEST_UUID %= MAX_UUID;
            HIGHEST_UUID
        }
    }
}

impl TexlaUuidProvider {
    pub(crate) fn new() -> Self {
        TexlaUuidProvider { highest_uuid: 0 }
    }
}
