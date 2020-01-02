use super::super::command::Command;
use super::super::replica::{BallotNum, ReplicaID};

#[cfg(test)]
#[path = "./tests/instance_tests.rs"]
mod tests;

/// protocol buffer serialized
pub type InstanceNum = i64;
pub struct InstanceID {
    pub replica_id: ReplicaID,
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

pub type Sequence = i64;

pub type InstIDs = Vec<InstanceID>;

pub struct Instance {
    pub status: InstanceStatus,
    pub cmds: Vec<Command>,
    pub ballot: BallotNum,
    pub seq: Sequence,
    pub deps: InstIDs,
}
