use crate::qpaxos::{Instance, InstanceID};
use crate::replica::ReplicaID;
use num;

// required by encode/decode
use prost::Message;

use super::Error;
use super::InstanceIter;

pub struct BaseIter<'a> {
    pub cursor: Vec<u8>,
    pub include: bool,
    pub engine: &'a dyn Base,
}

impl<'a> Iterator for BaseIter<'a> {
    type Item = (Vec<u8>, Vec<u8>);

    // TODO add unittest.
    fn next(&mut self) -> Option<Self::Item> {
        let r = self.engine.next_kv(&self.cursor, self.include);
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
    fn set_kv(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<(), Error>;
    /// get an existing value with key
    fn get_kv(&self, key: &Vec<u8>) -> Result<Vec<u8>, Error>;

    /// next_kv returns a key-value pair greater than the given one(include=false),
    /// or greater or equal the given one(include=true)
    fn next_kv(&self, key: &Vec<u8>, include: bool) -> Option<(Vec<u8>, Vec<u8>)>;

    fn get_iter(&self, key: Vec<u8>, include: bool) -> BaseIter;
}

/// InstanceEngine offer functions to operate snapshot instances
pub trait InstanceEngine: StatusEngine {
    /// set a new instance
    fn set_instance(&mut self, iid: InstanceID, inst: Instance) -> Result<(), Error>;
    /// update an existing instance with instance id
    fn update_instance(&mut self, iid: InstanceID, inst: Instance) -> Result<(), Error>;
    /// get an instance with instance id
    fn get_instance(&self, iid: &InstanceID) -> Result<Instance, Error>;
    /// get an iterator to scan all instances with a leader replica id
    fn get_instance_iter(&self, rid: ReplicaID) -> Result<InstanceIter, Error>;
}

/// StatusEngine offer functions to operate snapshot status
pub trait StatusEngine: TxEngine + Base {
    type RID: num::Integer + std::fmt::LowerHex;
    type Item: Message + std::default::Default;

    /// get current maximum instance id with a leader replica
    fn get_max_instance_id(&self, rid: Self::RID) -> Result<Self::Item, Error> {
        let key = self.max_instance_id_key(rid);
        let val_bytes: Vec<u8> = self.get_kv(&key)?;

        match Self::Item::decode(val_bytes.as_slice()) {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::DBError {
                msg: "parse instance id error".to_string(),
            }),
        }
    }

    /// get executed maximum continuous instance id with a leader replica
    fn get_max_exec_instance_id(&self, rid: Self::RID) -> Result<Self::Item, Error> {
        let key = self.max_exec_instance_id_key(rid);
        let val_bytes: Vec<u8> = self.get_kv(&key)?;

        match Self::Item::decode(val_bytes.as_slice()) {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::DBError {
                msg: "parse instance id error".to_string(),
            }),
        }
    }

    fn max_instance_id_key(&self, rid: Self::RID) -> Vec<u8> {
        format!("/status/max_instance_id/{:016x}", rid).into_bytes()
    }

    fn max_exec_instance_id_key(&self, rid: Self::RID) -> Vec<u8> {
        format!("/status/max_exec_instance_id/{:016x}", rid).into_bytes()
    }

    fn set_instance_id(&mut self, key: Vec<u8>, iid: InstanceID) -> Result<(), Error> {
        let mut value = vec![];
        iid.encode(&mut value).unwrap();
        self.set_kv(key, value)
    }
}

/// TxEngine offer a transactional operation on a storage.
pub trait TxEngine {
    /// start a transaction
    fn trans_begin(&mut self);
    /// commit a transaction
    fn trans_commit(&mut self) -> Result<(), Error>;
    /// rollback a transaction
    fn trans_rollback(&mut self) -> Result<(), Error>;
    /// get a key to set exclusively, must be called in an transaction
    fn get_kv_for_update(&self, key: &Vec<u8>) -> Result<Vec<u8>, Error>;
}
