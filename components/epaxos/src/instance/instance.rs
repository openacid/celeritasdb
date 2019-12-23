use super::super::command;
use super::super::replica;

#[cfg(test)]
#[path = "./tests/instance_tests.rs"]
mod tests;

/// protocol buffer serialized
pub type InstanceNum = i64;
pub struct InstanceID {
    pub replica_id: replica::ReplicaID,
    pub num: InstanceNum,
}

pub enum InstanceStatus {
    NA, // status not available means None, but `None` is a key workd.
    PreAccepted,
    PreAcceptedEQ,
    Accepted,
    Committed,
    Executed,
}

pub struct DepCmds(Vec<InstanceID>);

pub struct Instance {
    pub status: InstanceStatus,
    pub cmds: Vec<command::Command>,
    pub ballot: replica::BallotNum,
    pub seq: u64,
    pub deps: DepCmds,
}
