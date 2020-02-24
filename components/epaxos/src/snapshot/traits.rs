use crate::instance::{Instance, InstanceID};
use crate::replica::ReplicaID;

use protobuf::{parse_from_bytes, Message};

use super::Error;
use super::InstanceIter;

/// KVEngine offer functions to operate snapshot key-values
pub trait KVEngine {
    /// set a new key-value
    fn set_kv(&mut self, key: &Vec<u8>, value: &Vec<u8>) -> Result<(), Error>;
    /// get an existing value with key
    fn get_kv(&self, key: &Vec<u8>) -> Result<Vec<u8>, Error>;
}

/// InstanceEngine offer functions to operate snapshot instances
pub trait InstanceEngine<T>: KVEngine {
    /// set a new instance
    fn set_instance(&mut self, iid: &InstanceID, inst: Instance) -> Result<(), Error>;
    /// update an existing instance with instance id
    fn update_instance(&mut self, iid: &InstanceID, inst: Instance) -> Result<(), Error>;
    /// get an instance with instance id
    fn get_instance(&self, iid: &InstanceID) -> Result<Instance, Error>;
    /// get an iterator to scan all instances with a leader replica id
    fn get_instance_iter(&self, rid: ReplicaID) -> Result<InstanceIter<T>, Error>;
}

/// StatusEngine offer functions to operate snapshot status
pub trait StatusEngine: KVEngine {
    /// get current maximum instance id with a leader replica
    fn get_max_instance_id(&self, rid: ReplicaID) -> Result<InstanceID, Error>;

    /// get executed maximum continuous instance id with a leader replica
    fn get_max_exec_instance_id(&self, rid: ReplicaID) -> Result<InstanceID, Error>;

    fn max_instance_id_key(&self, rid: ReplicaID) -> Vec<u8> {
        format!("/status/max_instance_id/{:x}", rid).into_bytes()
    }

    fn max_exec_instance_id_key(&self, rid: ReplicaID) -> Vec<u8> {
        format!("/status/max_exec_instance_id/{:x}", rid).into_bytes()
    }

    fn set_instance_id(&mut self, key: &Vec<u8>, iid: InstanceID) -> Result<(), Error> {
        let value: Vec<u8> = iid.write_to_bytes().unwrap();
        self.set_kv(&key, &value)
    }
}

/// TransactionEngine offer a transaction to operate key-values and instances atomically
pub trait TransactionEngine<T>: StatusEngine + InstanceEngine<T> {
    /// start a transaction
    fn trans_begin(&mut self);
    /// commit a transaction
    fn trans_commit(&mut self) -> Result<(), Error>;
    /// rollback a transaction
    fn trans_rollback(&mut self) -> Result<(), Error>;
    /// get a key to set exclusively, must be called in an transaction
    fn get_kv_for_update(&self, key: &Vec<u8>) -> Result<Vec<u8>, Error>;
}
