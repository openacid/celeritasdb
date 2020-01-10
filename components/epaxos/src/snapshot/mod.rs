use rocksdb::DB;

mod errors;
pub use errors::*;

mod rocks;

pub struct Engine<'a> {
    _db: DB,
    _cf: &'a str,
}

pub enum DBPath{
    Product,
    Test,
}

impl DBPath {
    fn as_str<'a>(&self) -> &'a str {
        match self {
            DBPath::Product => "snapshot.db",
            DBPath::Test => "test.db",
        }
    }
}

pub enum DBCF {
    Default,
    Instance,
    Config,
    Conflict,
}

impl DBCF {
    pub fn as_str<'a>(&self) -> &'a str {
        match self {
            DBCF::Default => "default",
            DBCF::Instance => "instance",
            DBCF::Config => "config",
            DBCF::Conflict => "conflict",
        }
    }

    fn all<'a>() -> Vec<&'a str> {
        vec![
            DBCF::Default.as_str(),
            DBCF::Instance.as_str(),
            DBCF::Config.as_str(),
            DBCF::Conflict.as_str(),
        ]
    }
}
