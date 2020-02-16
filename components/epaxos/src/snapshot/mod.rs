use rocksdb::DB;

#[macro_use]
mod enum_str;

mod errors;
pub use errors::*;

mod traits;
pub use traits::*;

mod rocks;

mod memDB;
pub use memDB::*;

mod itersDB;
pub use itersDB::*;

pub struct Engine {
    _db: DB,
    _cf: &'static str,
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
