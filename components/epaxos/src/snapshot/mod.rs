use rocksdb::DB;

#[cfg(test)]
#[path = "tests/snapshot_test.rs"]
mod tests;

mod errors;
pub use errors::*;

mod cfg_handle;
pub use cfg_handle::*;

mod kv_handle;
mod inst_handle;
mod conflict_handle;
mod rocks;
mod util;

pub struct Engine<'a> {
    db: DB,
    cf: &'a str,
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
