use rocksdb::DB;

mod rocks;
use rocks::*;

mod engine;
pub use engine::*;

pub struct RocksDBEngine {
    db: DB,
}
