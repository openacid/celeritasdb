use crate::instance::{Instance, InstanceID};
use crate::replica::ReplicaID;

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
pub trait InstanceEngine {
    /// set a new instance
    fn set_instance(&mut self, inst: Instance) -> Result<(), Error>;
    /// update an existing instance with instance id
    fn update_instance(&mut self, inst: Instance) -> Result<(), Error>;
    /// get an instance with instance id
    fn get_instance(&self, inst_id: InstanceID) -> Result<Instance, Error>;
    /// get an iterator to scan all instances with a leader replica id
    fn get_instance_iter(&self, repl_id: ReplicaID) -> Result<InstanceIter, Error>;
}

/// StatusEngine offer functions to operate snapshot status
pub trait StatusEngine {
    /// get current maximum instance id with a leader replica
    fn get_max_instance_id(&self, repl_id: ReplicaID) -> Result<InstanceID, Error>;

    /// get executed maximum continuous instance id with a leader replica
    fn get_max_executed_instance_id(&self, repl_id: ReplicaID) -> Result<InstanceID, Error>;
}

/// TransactionEngine offer a transaction to operate key-values and instances atomically
pub trait TransactionEngine: KVEngine + InstanceEngine {
    /// start a transaction
    fn trans_begin(&mut self);
    /// commit a transaction
    fn trans_commit(&mut self) -> Result<(), Error>;
    /// rollback a transaction
    fn trans_rollback(&mut self) -> Result<(), Error>;
    /// get a key to set exclusively, must be called in an transaction
    fn get_kv_for_update(&self, key: &Vec<u8>) -> Result<Vec<u8>, Error>;
}
