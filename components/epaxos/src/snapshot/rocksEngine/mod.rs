use rocksdb::DB;
use super::{Error};

mod dbutil;
use dbutil::*;

mod rocks;
pub use rocks::*;

pub struct RocksDBEngine {
    db: DB,
}

enum_str! {
    pub DBColumnFamily {
        Default("default")
        Instance("instance")
        Status("status")
    }
}

impl DBColumnFamily {
    fn all<'a>() -> Vec<&'a str> {
        vec![
            DBColumnFamily::Default.as_str(),
            DBColumnFamily::Instance.as_str(),
            DBColumnFamily::Status.as_str(),
        ]
    }
}

#[test]
fn test_db_columnfamily() {
    let exp = vec!["default", "instance", "status"];

    assert_eq!(exp, DBColumnFamily::all());
}
