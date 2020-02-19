use rocksdb::DB;

#[macro_use]
mod enum_str;

mod errors;
pub use errors::*;

mod rocks;
pub use rocks::*;

mod traits;
pub use traits::*;

pub struct RocksDBEngine {
    db: DB,
}

// TODO: define details later
pub struct InstanceIter {}
