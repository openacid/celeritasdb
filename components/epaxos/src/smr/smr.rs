use std::collections::HashMap;

use super::super::command;
use super::super::instance::{Instance, InstanceID, Sequence};

pub struct SMR {
    pub replica_nums:   i32,
    pub durable:        bool, // log to a stable storage?

    pub inst_space:     Vec<Vec<Instance>>, // the space of all instances (used and not yet used)
    pub crt_inst:       Vec<InstanceID>,    // highest active instance numbers that this replica knows about
    pub committed_inst: Vec<InstanceID>,    // highest committed instance per replica that this replica knows about
    pub executed_inst:  Vec<InstanceID>,    // instance up to which all commands have been executed (including iteslf)

    pub conflicts:       Vec<HashMap<command::Key, InstanceID>>,
    pub max_seq_per_key: HashMap<command::Key, Sequence>,
    pub max_seq:         Sequence,
}
