use crate::qpaxos::Command;
use crate::qpaxos::OpCode;
use crate::qpaxos::{Instance, InstanceId, ReplicaID};
use crate::tokey::ToKey;
use std::sync::Arc;

// required by encode/decode
use prost::Message;

use super::super::BaseIter;
use super::super::Error;
use super::super::InstanceIter;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DBColumnFamily {
    Default,
    Instance,
    Status,
}

impl DBColumnFamily {
    pub fn all() -> Vec<DBColumnFamily> {
        vec![
            DBColumnFamily::Default,
            DBColumnFamily::Instance,
            DBColumnFamily::Status,
        ]
    }
}

impl From<&DBColumnFamily> for &str {
    fn from(cf: &DBColumnFamily) -> Self {
        match cf {
            DBColumnFamily::Default => return "default",
            DBColumnFamily::Instance => return "instance",
            DBColumnFamily::Status => return "status",
        }
    }
}

impl From<DBColumnFamily> for &str {
    fn from(cf: DBColumnFamily) -> Self {
        (&cf).into()
    }
}

pub enum WriteEntry {
    Nil,
    Set(DBColumnFamily, Vec<u8>, Vec<u8>),
    Delete(DBColumnFamily, Vec<u8>),
}

impl From<&Command> for WriteEntry {
    fn from(c: &Command) -> Self {
        if OpCode::Set as i32 == c.op {
            return WriteEntry::Set(DBColumnFamily::Default, c.key.clone(), c.value.clone());
        } else if OpCode::Delete as i32 == c.op {
            return WriteEntry::Delete(DBColumnFamily::Default, c.key.clone());
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

impl<A: Into<InstanceId> + Copy> From<(&str, A)> for WriteEntry {
    fn from(args: (&str, A)) -> Self {
        let iid = args.1.into();
        let k = make_ref_key(args.0, iid.replica_id);
        let mut v = vec![];
        iid.encode(&mut v).unwrap();

        return WriteEntry::Set(DBColumnFamily::Status, k, v);
    }
}

pub fn make_ref_key(typ: &str, rid: ReplicaID) -> Vec<u8> {
    match typ {
        "max" => format!("/status/max_instance_id/{:016x}", rid).into_bytes(),
        "exec" => format!("/status/max_exec_instance_id/{:016x}", rid).into_bytes(),
        _ => panic!("unknown type ref"),
    }
}

/// Base offer basic key-value access
pub trait Base {
    /// set a new key-value
    fn set(&self, cf: DBColumnFamily, key: &Vec<u8>, value: &Vec<u8>) -> Result<(), Error>;

    /// get an existing value with key
    fn get(&self, cf: DBColumnFamily, key: &Vec<u8>) -> Result<Option<Vec<u8>>, Error>;

    /// delete a key
    fn delete(&self, cf: DBColumnFamily, key: &Vec<u8>) -> Result<(), Error>;

    /// next_kv returns a key-value pair greater than the given one(include=false),
    /// or greater or equal the given one(include=true)
    fn next(&self, cf: DBColumnFamily, key: &Vec<u8>, include: bool) -> Option<(Vec<u8>, Vec<u8>)>;

    /// prev_kv returns a key-value pair smaller than the given one(include=false),
    /// or smaller or equal the given one(include=true)
    fn prev(&self, cf: DBColumnFamily, key: &Vec<u8>, include: bool) -> Option<(Vec<u8>, Vec<u8>)>;

    fn write_batch(&self, entrys: &Vec<WriteEntry>) -> Result<(), Error>;
}

pub trait KV: Base {
    fn set_kv(&self, key: &Vec<u8>, value: &Vec<u8>) -> Result<(), Error> {
        self.set(DBColumnFamily::Default, key, value)
    }

    fn get_kv(&self, key: &Vec<u8>) -> Result<Option<Vec<u8>>, Error> {
        self.get(DBColumnFamily::Default, key)
    }

    fn delete_kv(&self, key: &Vec<u8>) -> Result<(), Error> {
        self.delete(DBColumnFamily::Default, key)
    }

    fn next_kv(&self, key: &Vec<u8>, include: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        self.next(DBColumnFamily::Default, key, include)
    }

    fn prev_kv(&self, key: &Vec<u8>, include: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        self.prev(DBColumnFamily::Default, key, include)
    }
}

/// ColumnedEngine organizes object in columns.
/// Because the underlying storage is a simple object store,
/// it introduces ColumnId to classify objects.
/// And also it provides APIs to track objects in different columns.
///
/// set_ref(type, ReplicaID, InstanceId) to store InstanceId of `type` in ReplicaID.
///
/// E.g.: `set_ref("max", 1, (1, 2).into())` to set the "max" InstanceId of Replica 1.
pub trait ColumnedEngine: Base {
    fn set_ref(&self, typ: &str, rid: ReplicaID, iid: InstanceId) -> Result<(), Error> {
        let key = make_ref_key(typ, rid);

        let mut value = vec![];
        iid.encode(&mut value)?;

        self.set(DBColumnFamily::Status, &key, &value)
    }

    fn get_ref(&self, typ: &str, rid: ReplicaID) -> Result<Option<InstanceId>, Error> {
        let key = make_ref_key(typ, rid);
        let val = self.get(DBColumnFamily::Status, &key)?;

        let val = match val {
            Some(v) => v,
            None => return Ok(None),
        };

        Ok(Some(InstanceId::decode(val.as_slice())?))
    }

    /// set_ref_if set ref if the current value satisifies specified condition.
    /// The condition is a lambda takes one arguments: the current value of the ref.
    /// This method should be called with concurrency control.
    ///
    /// # Arguments:
    ///
    /// `typ`: ref type.
    /// `rid`: column id of type InstanceId.
    /// `iid`: object id of type InstanceId.
    /// `default`: the default value to feed to `cond` if ref is not found.
    /// `cond`: a lambda takes one argument of type InstanceId.
    fn set_ref_if<P>(
        &self,
        typ: &str,
        rid: ReplicaID,
        iid: InstanceId,
        default: InstanceId,
        cond: P,
    ) -> Result<(), Error>
    where
        Self: Sized,
        P: Fn(InstanceId) -> bool,
    {
        let r0 = self.get_ref(typ, rid)?;
        let r0 = r0.unwrap_or(default);

        if cond(r0) {
            self.set_ref(typ, rid, iid)
        } else {
            Ok(())
        }
    }
}

pub trait Iter: Base {
    fn get_iter(
        &self,
        cursor: Vec<u8>,
        include: bool,
        reverse: bool,
        cf: DBColumnFamily,
    ) -> BaseIter;

    fn get_instance_iter(&self, iid: InstanceId, include: bool, reverse: bool) -> InstanceIter;
}

/// InstanceEngine offer functions to operate snapshot instances
pub trait InstanceEngine: ColumnedEngine + KV + Iter + Send + Sync {
    /// Find next available instance id and increase max-instance-id ref.
    fn next_instance_id(&self, rid: ReplicaID) -> Result<InstanceId, Error> {
        // TODO locking TODO Need to incr max-ref and add new-instance in a single tx.
        //      Or iterator may encounter an empty instance slot.
        let max = self.get_ref("max", rid)?;
        let mut max = max.unwrap_or((rid, -1).into());
        max.idx += 1;
        self.set_ref("max", rid, max)?;
        Ok(max)
    }

    /// set an instance
    fn set_instance(&self, inst: &Instance) -> Result<(), Error> {
        // TODO does not guarantee in a transaction
        let iid = inst.instance_id.unwrap().to_key();
        let mut value = vec![];
        inst.encode(&mut value)?;

        self.set(DBColumnFamily::Instance, &iid, &value)
    }

    /// get an instance with instance id
    fn get_instance(&self, iid: InstanceId) -> Result<Option<Instance>, Error> {
        let key = iid.to_key();
        let vbs = self.get(DBColumnFamily::Instance, &key)?;
        let r = match vbs {
            Some(v) => Instance::decode(v.as_slice())?,
            None => return Ok(None),
        };

        Ok(Some(r))
    }
}

impl<T> KV for T where T: Base + Send + Sync {}
impl<T> ColumnedEngine for T where T: Base + Send + Sync {}
impl<T> InstanceEngine for T where T: Base + Send + Sync {}

impl<T> Iter for T
where
    T: Base + Send + Sync,
{
    fn get_iter(
        &self,
        cursor: Vec<u8>,
        include: bool,
        reverse: bool,
        cf: DBColumnFamily,
    ) -> BaseIter {
        BaseIter {
            cursor,
            include,
            engine: self,
            reverse,
            cf,
        }
    }

    fn get_instance_iter(&self, iid: InstanceId, include: bool, reverse: bool) -> InstanceIter {
        InstanceIter {
            curr_inst_id: iid,
            include,
            engine: self,
            reverse,
        }
    }
}
pub type Storage = Arc<dyn InstanceEngine>;
