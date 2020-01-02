use super::super::command::{Command, Proposal};
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

pub struct RecoveryInstance {
    pub cmds:     Vec<Command>,
    pub status:   InstanceStatus,
    pub seq:      Sequence,
    pub deps:     InstIDs,
    pub pa_count: i32, // preAccept count
    pub leader_responded: bool,
}

pub struct LeaderBookkeeping{
    pub client_proposals:  Vec<Proposal>,
    pub max_recv_ballot:   BallotNum,
    pub prepare_oks:       i32,
    pub all_equal:         bool,
    pub pa_oks:            i32, //preAccept_oks
    pub accept_oks:        i32,
    pub nacks:             i32,
    pub original_deps:     InstIDs,
    pub committed_deps:    InstIDs,
    pub recovery_inst:     RecoveryInstance,
    pub preparing:         bool,
    pub trying_pre_accept: bool,
    pub possible_quorum:   Vec<bool>,
    pub tpa_oks:           i32,
}

pub struct Instance {
    pub status: InstanceStatus,
    pub cmds: Vec<Command>,
    pub ballot: BallotNum,
    pub seq: Sequence,
    pub deps: InstIDs,
    pub lb: LeaderBookkeeping,
}
