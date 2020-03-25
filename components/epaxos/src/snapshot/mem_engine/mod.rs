use std::collections::BTreeMap;
use std::sync::Mutex;

mod memdb;
pub use memdb::*;

pub type MemBT = BTreeMap<Vec<u8>, Vec<u8>>;

/// MemEngine is a in-memory storage for testing or non-persistent environment.
///
/// A storage deals with concurrency itself.
/// Replicas keeps a `Arc` reference to storage engine.
///
/// ```text
/// Replica-1 → Engine
///           ↗
/// Replica-1
/// ```
pub struct MemEngine {
    pub _db: Mutex<MemBT>,
}
