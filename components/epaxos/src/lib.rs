#![feature(is_sorted)]

#[macro_use]
extern crate maplit;

#[macro_use]
extern crate quick_error;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate slog_global;

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

use storage::AccessInstance;
use storage::AccessStatus;
use storage::DBColumnFamily;
use storage::Storage;
use storage::ToKey;
use storage::WriteEntry;

impl AccessInstance<InstanceId, Instance> for Storage {}
impl AccessStatus<ReplicaStatus, InstanceIds> for Storage {}

use prost::Message;
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
        return WriteEntry::Set(DBColumnFamily::Instance, inst.to_key(), v);
    }
}
