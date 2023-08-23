use serde::{Deserialize, Serialize};

pub(crate) type Uuid = u64;

static JS_MAX_SAFE_INTEGER: Uuid = 2u64.pow(53);
static MAX_UUID: Uuid = JS_MAX_SAFE_INTEGER;

/// This uses unsafe code but is safe, because UUIDs do not need to be unique across ASTs that are
/// built at the same time, as they cannot belong to the same client.
/// Cleaner: https://stackoverflow.com/questions/27791532/how-do-i-create-a-global-mutable-singleton
static mut HIGHEST_UUID: Uuid = 0;

pub(crate) trait UuidProvider {
    fn new_uuid(&mut self) -> Uuid;
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TexlaUuidProvider {
    highest_uuid: Uuid,
}

impl UuidProvider for TexlaUuidProvider {
    /// For a justification of unsafe, see [HIGHEST_UUID]
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

/// Represents a Position in an [Ast]. The Position points between to nodes or behind a node in order to allow specifying positions which are not currently occupied.
/// As a result this can not be used to specify the position of a node that is already in the Ast.
#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Position {
    pub parent: Uuid,
    pub after_sibling: Option<Uuid>,
}
