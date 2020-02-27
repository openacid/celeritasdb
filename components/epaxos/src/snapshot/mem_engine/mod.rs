use std::collections::BTreeMap;
use std::sync::Mutex;

mod memdb;
pub use memdb::*;

pub struct MemEngine {
    pub _db: BTreeMap<Vec<u8>, Vec<u8>>,
    pub _mutex: Mutex<i32>,
}
