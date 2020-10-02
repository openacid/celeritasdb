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
pub use replication::*;
pub use serverdata::*;
pub use service::*;

pub use qpaxos::*;
use std::sync::Arc;
use storage::Engine;
pub use storage::*;
pub type Storage = Arc<dyn Engine<ReplicaId, InstanceId, InstanceId, Instance>>;

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

// just for set max exec ref
// we can not impl From<(&str, A: Into<InstanceId>)>
// because WriteEntry is not in this crate
impl From<InstanceId> for WriteEntry {
    fn from(iid: InstanceId) -> Self {
        let k = make_ref_key("exec", iid.replica_id);
        let mut v = vec![];
        iid.encode(&mut v).unwrap();

        return WriteEntry::Set(DBColumnFamily::Status, k, v);
    }
}
