use rocksdb::DB;

#[macro_use]
mod enum_str;

mod errors;
pub use errors::*;

mod rocks;

pub struct Engine {
    _db: DB,
    _cf: &'static str,
}

enum_str! {
    pub DBPath {
        Product("snapshot.db")
        Test("test.db")
    }
}

enum_str! {
    pub DBColumnFamily {
        Default("default")
        Instance("instance")
        Config("config")
        Conflict("conflict")
    }
}

impl DBColumnFamily {
    fn all<'a>() -> Vec<&'a str> {
        vec![
            DBColumnFamily::Default.as_str(),
            DBColumnFamily::Instance.as_str(),
            DBColumnFamily::Config.as_str(),
            DBColumnFamily::Conflict.as_str(),
        ]
    }
}
