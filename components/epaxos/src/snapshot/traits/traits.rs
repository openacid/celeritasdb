use crate::qpaxos::{Instance, InstanceId, ReplicaID};
use crate::tokey::ToKey;
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
    fn set_kv(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), Error>;

    /// get an existing value with key
    fn get_kv(&self, key: &Vec<u8>) -> Result<Vec<u8>, Error>;

    /// next_kv returns a key-value pair greater than the given one(include=false),
    /// or greater or equal the given one(include=true)
    fn next_kv(&self, key: &Vec<u8>, include: bool) -> Option<(Vec<u8>, Vec<u8>)>;

    /// prev_kv returns a key-value pair smaller than the given one(include=false),
    /// or smaller or equal the given one(include=true)
    fn prev_kv(&self, key: &Vec<u8>, include: bool) -> Option<(Vec<u8>, Vec<u8>)>;

    fn get_iter(&self, key: Vec<u8>, include: bool, reverse: bool) -> BaseIter;
}

pub type Storage =
    Arc<dyn InstanceEngine<ColumnId = ReplicaID, ObjId = InstanceId, Obj = Instance>>;

/// InstanceEngine offer functions to operate snapshot instances
pub trait InstanceEngine: TxEngine + ColumnedEngine {
    /// Find next available instance id and increase max-instance-id ref.
    fn next_instance_id(&self, rid: ReplicaID) -> Result<InstanceId, Error>;

    /// set an instance
    fn set_instance(&self, inst: &Instance) -> Result<(), Error>;

    /// get an instance with instance id
    fn get_instance(&self, iid: InstanceId) -> Result<Option<Instance>, Error>;

    /// get an iterator to scan all instances with a leader replica id
    fn get_instance_iter(&self, iid: InstanceId, include: bool, reverse: bool) -> InstanceIter;
}

/// TxEngine offer a transactional operation on a storage.
pub trait TxEngine {
    /// start a transaction
    fn trans_begin(&self);
    /// commit a transaction
    fn trans_commit(&self) -> Result<(), Error>;
    /// rollback a transaction
    fn trans_rollback(&self) -> Result<(), Error>;
    /// get a key to set exclusively, must be called in an transaction
    fn get_kv_for_update(&self, key: &Vec<u8>) -> Result<Vec<u8>, Error>;
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
        let value = self.encode_obj(obj)?;

        self.set_kv(key, value)
    }

    fn get_obj(&self, objid: Self::ObjId) -> Result<Option<Self::Obj>, Error> {
        let key = objid.to_key();
        let vbs = self.get_kv(&key);

        let vbs = match vbs {
            Ok(v) => v,
            Err(e) => match e {
                Error::NotFound => {
                    return Ok(None);
                }
                _ => {
                    return Err(e);
                }
            },
        };

        let itm = self.decode_obj(&vbs)?;
        Ok(Some(itm))
    }

    fn encode_obj(&self, itm: &Self::Obj) -> Result<Vec<u8>, Error> {
        let mut value = vec![];
        itm.encode(&mut value).unwrap();
        Ok(value)
    }

    fn decode_obj(&self, bs: &Vec<u8>) -> Result<Self::Obj, Error> {
        match Self::Obj::decode(bs.as_slice()) {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::DBError {
                msg: "parse instance id error".to_string(),
            }),
        }
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
        objid.encode(&mut value).unwrap();

        self.set_kv(key, value)
    }

    fn get_ref(&self, typ: &str, col_id: Self::ColumnId) -> Result<Self::ObjId, Error> {
        let key = self.make_ref_key(typ, col_id);
        let val_bytes = self.get_kv(&key)?;

        match Self::ObjId::decode(val_bytes.as_slice()) {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::DBError {
                msg: "parse instance id error".to_string(),
            }),
        }
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
        let r0 = self.get_ref(typ, col_id);
        let r0 = match r0 {
            Ok(v) => v,
            Err(e) => match e {
                Error::NotFound => default,
                _ => return Err(e),
            },
        };

        if cond(r0) {
            self.set_ref(typ, col_id, objid)
        } else {
            Ok(())
        }
    }
}
