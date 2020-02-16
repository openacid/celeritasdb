use rocksdb::DB;

#[macro_use]
mod enum_str;

mod errors;
pub use errors::*;

mod rocks;
pub use rocks::*;

mod traits;
pub use traits::*;

mod memEngine;
pub use memEngine::*;

mod iters;
pub use iters::*;

pub struct RocksDBEngine {
    db: DB,
}

