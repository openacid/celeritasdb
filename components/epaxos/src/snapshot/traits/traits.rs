use crate::qpaxos::{Instance, InstanceId, ReplicaID};
use crate::snapshot::Command;
use crate::tokey::ToKey;
use std::marker::Send;
use std::marker::Sync;
use std::sync::Arc;

// required by encode/decode
use prost::Message;

use super::super::Error;
use super::super::InstanceIter;

pub struct BaseIter<'a> {
    pub cursor: Vec<u8>,
    pub include: bool,
    pub engine: &'a dyn Base,
    pub reverse: bool,
}

impl<'a> Iterator for BaseIter<'a> {
    type Item = (Vec<u8>, Vec<u8>);

    // TODO add unittest.
    fn next(&mut self) -> Option<Self::Item> {
        let r = if self.reverse {
            self.engine.prev_kv(&self.cursor, self.include)
        } else {
            self.engine.next_kv(&self.cursor, self.include)
        };

        self.include = false;
        match r {
            Some(kv) => {
                self.cursor = kv.0.clone();
                Some(kv)
            }
            None => None,
        }
    }
}

/// Base offer basic key-value access
pub trait Base {
    /// set a new key-value
    fn set_kv(&self, key: &Vec<u8>, value: &Vec<u8>) -> Result<(), Error>;

    /// get an existing value with key
    fn get_kv(&self, key: &Vec<u8>) -> Result<Option<Vec<u8>>, Error>;

    /// delete a key
    fn delete_kv(&self, key: &Vec<u8>) -> Result<(), Error>;

    /// next_kv returns a key-value pair greater than the given one(include=false),
    /// or greater or equal the given one(include=true)
    fn next_kv(&self, key: &Vec<u8>, include: bool) -> Option<(Vec<u8>, Vec<u8>)>;

    /// prev_kv returns a key-value pair smaller than the given one(include=false),
    /// or smaller or equal the given one(include=true)
    fn prev_kv(&self, key: &Vec<u8>, include: bool) -> Option<(Vec<u8>, Vec<u8>)>;

    fn get_iter(&self, key: Vec<u8>, include: bool, reverse: bool) -> BaseIter;

    fn write_batch(&self, cmds: &Vec<Command>) -> Result<(), Error>;
}

pub type Storage =
    Arc<dyn InstanceEngine<ColumnId = ReplicaID, ObjId = InstanceId, Obj = Instance> + Sync + Send>;

/// InstanceEngine offer functions to operate snapshot instances
pub trait InstanceEngine: ColumnedEngine {
    /// Find next available instance id and increase max-instance-id ref.
    fn next_instance_id(&self, rid: ReplicaID) -> Result<InstanceId, Error>;

    /// set an instance
    fn set_instance(&self, inst: &Instance) -> Result<(), Error>;

    /// get an instance with instance id
    fn get_instance(&self, iid: InstanceId) -> Result<Option<Instance>, Error>;

    /// get an iterator to scan all instances with a leader replica id
    fn get_instance_iter(&self, iid: InstanceId, include: bool, reverse: bool) -> InstanceIter;
}

/// ObjectEngine wraps bytes based storage engine into an object based engine.
/// Structured object can be stored and retreived with similar APIs.
/// An object is serialized into bytes with protobuf engine prost.
///
/// TODO example
pub trait ObjectEngine: Base {
    /// ObjId defines the type of object id.
    /// It must be able to convert to a key in order to store an object.
    /// Alsot it needs to be serialized as Message in order to be stored as an object too.
    type ObjId: ToKey + Message + std::default::Default;

    /// Obj defines the type of an object.
    type Obj: Message + std::default::Default;

    fn set_obj(&self, objid: Self::ObjId, obj: &Self::Obj) -> Result<(), Error> {
        let key = objid.to_key();
        let mut value = vec![];
        obj.encode(&mut value)?;

        self.set_kv(&key, &value)
    }

    fn get_obj(&self, objid: Self::ObjId) -> Result<Option<Self::Obj>, Error> {
        let key = objid.to_key();
        let vbs = self.get_kv(&key)?;

        let r = match vbs {
            Some(v) => Self::Obj::decode(v.as_slice())?,
            None => return Ok(None),
        };

        Ok(Some(r))
    }
}

/// ColumnedEngine organizes object in columns.
/// Because the underlying storage is a simple object store,
/// it introduces ColumnId to classify objects.
/// And also it provides APIs to track objects in different columns.
///
/// set_ref(type, col_id, obj_id) to store a column reference of `type` to be `obj_id`.
///
/// E.g.: `set_ref("max", 1, (1, 2))` to set the "max" object in column 1 to be object with object-id
/// (1, 2)
///
/// A User should implement make_ref_key() to make reference keys.
pub trait ColumnedEngine: ObjectEngine {
    type ColumnId: Copy;

    fn make_ref_key(&self, typ: &str, col_id: Self::ColumnId) -> Vec<u8>;

    fn set_ref(&self, typ: &str, col_id: Self::ColumnId, objid: Self::ObjId) -> Result<(), Error> {
        let key = self.make_ref_key(typ, col_id);

        let mut value = vec![];
        objid.encode(&mut value)?;

        self.set_kv(&key, &value)
    }

    fn get_ref(&self, typ: &str, col_id: Self::ColumnId) -> Result<Option<Self::ObjId>, Error> {
        let key = self.make_ref_key(typ, col_id);
        let val = self.get_kv(&key)?;

        let val = match val {
            Some(v) => v,
            None => return Ok(None),
        };

        Ok(Some(Self::ObjId::decode(val.as_slice())?))
    }

    /// set_ref_if set ref if the current value satisifies specified condition.
    /// The condition is a lambda takes one arguments: the current value of the ref.
    /// This method should be called with concurrency control.
    ///
    /// # Arguments:
    ///
    /// `typ`: ref type.
    /// `col_id`: column id of type Self::ColumnId.
    /// `objid`: object id of type Self::ObjId.
    /// `default`: the default value to feed to `cond` if ref is not found.
    /// `cond`: a lambda takes one argument of type Self::ObjId.
    fn set_ref_if<P>(
        &self,
        typ: &str,
        col_id: Self::ColumnId,
        objid: Self::ObjId,
        default: Self::ObjId,
        cond: P,
    ) -> Result<(), Error>
    where
        Self: Sized,
        P: Fn(Self::ObjId) -> bool,
    {
        let r0 = self.get_ref(typ, col_id)?;
        let r0 = r0.unwrap_or(default);

        if cond(r0) {
            self.set_ref(typ, col_id, objid)
        } else {
            Ok(())
        }
    }
}
