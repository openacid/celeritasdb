use rocksdb::DB;

mod errors;
pub use errors::*;

mod rocks;

pub struct Engine<'a> {
    _db: DB,
    _cf: &'a str,
}

pub enum DBPath {
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

pub enum DBColumnFamily {
    Default,
    Instance,
    Config,
    Conflict,
}

impl DBColumnFamily {
    pub fn as_str<'a>(&self) -> &'a str {
        match self {
            DBColumnFamily::Default => "default",
            DBColumnFamily::Instance => "instance",
            DBColumnFamily::Config => "config",
            DBColumnFamily::Conflict => "conflict",
        }
    }

    fn all<'a>() -> Vec<&'a str> {
        vec![
            DBColumnFamily::Default.as_str(),
            DBColumnFamily::Instance.as_str(),
            DBColumnFamily::Config.as_str(),
            DBColumnFamily::Conflict.as_str(),
        ]
    }
}
