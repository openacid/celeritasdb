use super::super::command;
use super::super::replica::{ReplicaID};
use super::super::instance::{Instance, InstanceID};

pub struct SMR {
    // SMR contains the following fields:

    // pub inst_space:     Vec<Vec<Instance>>, // the space of all instances (used and not yet used)
    // pub crt_inst:       Vec<InstanceID>,    // highest active instance numbers that this replica knows about
    // pub committed_inst: Vec<InstanceID>,    // highest committed instance per replica that this replica knows about
    // pub executed_inst:  Vec<InstanceID>,    // instance up to which all commands have been executed (including iteslf)

    // pub conflicts:       Vec<HashMap<command::Key, InstanceID>>,
}

impl SMR {
    // inst_space
    pub fn get_inst(&self, rid: ReplicaID, iid: InstanceID) -> Result<Instance, String> {
        Err("not implemented".to_string())
    }

    pub fn set_inst(&self, rid: ReplicaID, inst: Instance) -> Result<(), String> {
        Err("not implemented".to_string())
    }

    pub fn scan_insts(&self, rid: ReplicaID, siid: InstanceID, eiid: InstanceID) -> Result<Box<dyn Iterator<Item=Instance>>, String> {
        Err("not implemented".to_string())
    }

    // crt_inst
    pub fn get_crt_inst(&self, rid: ReplicaID) -> Result<Vec<InstanceID>, String> {
        Err("not implemented".to_string())
    }

    pub fn set_crt_inst(&self, rid: ReplicaID, ids: Vec<InstanceID>) -> Result<(), String> {
        Err("not implemented".to_string())
    }

    // committed_inst
    pub fn get_committed_inst(&self, rid: ReplicaID) -> Result<Vec<InstanceID>, String> {
        Err("not implemented".to_string())
    }

    pub fn set_committed_inst(&self, rid: ReplicaID, ids: Vec<InstanceID>) -> Result<(), String> {
        Err("not implemented".to_string())
    }

    // executed_inst
    pub fn get_executed_inst(&self, rid: ReplicaID) -> Result<Vec<InstanceID>, String> {
        Err("not implemented".to_string())
    }

    pub fn set_executed_inst(&self, rid: ReplicaID, ids: Vec<InstanceID>) -> Result<(), String> {
        Err("not implemented".to_string())
    }

    // conflicts
    pub fn get_conflicts(&self, rid: ReplicaID, key: command::Key) -> Result<InstanceID, String> {
        Err("not implemented".to_string())
    }

    pub fn set_confliects(&self, rid: ReplicaID, key: command::Key, iid: InstanceID) -> Result<(), String> {
        Err("not implemented".to_string())
    }
}
