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

    fn set_kv(&self, key: &[u8], value: &Record) -> Result<(), StorageError> {
        self.set(DBColumnFamily::Record, key, value)
    }

    fn get_kv(&self, key: &[u8]) -> Result<Option<Record>, StorageError> {
        self.get(DBColumnFamily::Record, key)
    }

    fn delete_kv(&self, key: &[u8]) -> Result<(), StorageError> {
        self.delete(DBColumnFamily::Record, key)
    }

    fn next_kv(
        &self,
        key: &Vec<u8>,
        include: bool,
    ) -> Result<Option<(Vec<u8>, Record)>, StorageError> {
        self.next::<Vec<u8>, Record>(DBColumnFamily::Record, key, true, include)
    }

    fn prev_kv(
        &self,
        key: &Vec<u8>,
        include: bool,
    ) -> Result<Option<(Vec<u8>, Record)>, StorageError> {
        self.next(DBColumnFamily::Record, key, false, include)
    }

    fn make_cmd_entry(&self, c: &Command) -> WriteEntry {
        if OpCode::Set as i32 == c.op {
            let mut vbytes = vec![];
            Record::from(c.value.clone()).encode(&mut vbytes).unwrap();
            return WriteEntry::Set(DBColumnFamily::Record, self.prepend_ns(&c.key), vbytes);
        } else if OpCode::Delete as i32 == c.op {
            return WriteEntry::Delete(DBColumnFamily::Record, self.prepend_ns(&c.key));
        } else {
            return WriteEntry::Nil;
        }
    }
    fn make_inst_entry(&self, inst: &Instance) -> WriteEntry {
        let mut v = vec![];
        inst.encode(&mut v).unwrap();
        return WriteEntry::Set(
            DBColumnFamily::Instance,
            self.prepend_ns(&inst.into_key()),
            v,
        );
    }
}

impl StorageAPI for Storage {}
