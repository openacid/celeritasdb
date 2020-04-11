use super::{Base, BaseIter, Error};
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DBColumnFamily {
    Default,
    Instance,
    Status,
}

impl DBColumnFamily {
    fn all() -> Vec<DBColumnFamily> {
        vec![
            DBColumnFamily::Default,
            DBColumnFamily::Instance,
            DBColumnFamily::Status,
        ]
    }
}

impl From<&DBColumnFamily> for &str {
    fn from(cf: &DBColumnFamily) -> Self {
        match cf {
            DBColumnFamily::Default => return "default",
            DBColumnFamily::Instance => return "instance",
            DBColumnFamily::Status => return "status",
        }
    }
}

impl From<DBColumnFamily> for &str {
    fn from(cf: DBColumnFamily) -> Self {
        (&cf).into()
    }
}
