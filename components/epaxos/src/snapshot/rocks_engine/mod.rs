use super::{Base, Error};
use rocksdb::DB;

mod rocks;
use rocks::*;

mod engine;
pub use engine::*;

#[cfg(test)]
mod test_engine;

pub struct RocksDBEngine {
    db: DB,
}
