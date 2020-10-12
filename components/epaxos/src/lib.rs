#![feature(is_sorted)]

#[macro_use]
extern crate maplit;

#[macro_use]
extern crate quick_error;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate slog_global;

use prost::Message;

#[macro_use]
pub mod testutil;

pub mod conf;
mod iters;
mod serverdata;
mod service;

#[macro_use]
pub mod qpaxos;
pub mod replica;
pub mod replication;

pub use conf::*;
pub use iters::*;
pub use qpaxos::*;
pub use replication::*;
pub use serverdata::*;
pub use service::*;

#[cfg(test)]
mod test_qpaxos_storage;

use storage::AsStorageKey;
use storage::DBColumnFamily;
use storage::ObjectKV;
use storage::RawKV;
use storage::Storage;
use storage::StorageError;
use storage::WriteEntry;

/// AccessRecord provides API to access user key/value record.
/// TODO: do not need RawKV
pub trait StorageAPI: ObjectKV + RawKV {
    /// set status
    fn set_status(&self, key: &ReplicaStatus, value: &InstanceIds) -> Result<(), StorageError> {
        self.set(DBColumnFamily::Status, key, value)
    }

    /// get an status by key
    fn get_status(&self, key: &ReplicaStatus) -> Result<Option<InstanceIds>, StorageError> {
        self.get(DBColumnFamily::Status, key)
    }

    /// set an instance
    fn set_instance(&self, key: &InstanceId, v: &Instance) -> Result<(), StorageError> {
        self.set(DBColumnFamily::Instance, key, v)
    }

    /// get an instance by instance id
    fn get_instance(&self, k: &InstanceId) -> Result<Option<Instance>, StorageError> {
        self.get(DBColumnFamily::Instance, k)
    }

    fn set_kv(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        self.set_raw(DBColumnFamily::Record, key, value)
    }

    fn get_kv(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        self.get_raw(DBColumnFamily::Record, key)
    }

    fn delete_kv(&self, key: &[u8]) -> Result<(), StorageError> {
        self.delete_raw(DBColumnFamily::Record, key)
    }

    fn next_kv(&self, key: &[u8], include: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        self.next_raw(DBColumnFamily::Record, key, true, include)
    }

    fn prev_kv(&self, key: &[u8], include: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        self.next_raw(DBColumnFamily::Record, key, false, include)
    }
}

impl StorageAPI for Storage {}

impl From<&Command> for WriteEntry {
    fn from(c: &Command) -> Self {
        if OpCode::Set as i32 == c.op {
            return WriteEntry::Set(DBColumnFamily::Record, c.key.clone(), c.value.clone());
        } else if OpCode::Delete as i32 == c.op {
            return WriteEntry::Delete(DBColumnFamily::Record, c.key.clone());
        } else {
            return WriteEntry::Nil;
        }
    }
}

impl From<Instance> for WriteEntry {
    fn from(inst: Instance) -> Self {
        let mut v = vec![];
        inst.encode(&mut v).unwrap();
        return WriteEntry::Set(DBColumnFamily::Instance, inst.into_key(), v);
    }
}
