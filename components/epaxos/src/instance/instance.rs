use super::super::command::{Command, Propose};
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
    cmds:     Vec<Command>,
    status:   InstanceStatus,
    seq:      Sequence,
    deps:     Vec<InstanceID>,
    pa_count: i32, // preAccept count
    leader_responded: bool,
}

pub struct LeaderBookkeeping{
    client_proposals:  Vec<Propose>,
    max_recv_ballot:   BallotNum,
    prepare_oks:       i32,
    all_equal:         bool,
    pa_oks:            i32, //preAccept_oks
    accept_oks:        i32,
    nacks:             i32,
    original_deps:     Vec<InstanceID>,
    committed_deps:    Vec<InstanceID>,
    recovery_inst:     RecoveryInstance,
    preparing:         bool,
    trying_pre_accept: bool,
    possible_quorum:   Vec<bool>,
    tpa_oks:           i32,
}

pub struct Instance {
    pub status: InstanceStatus,
    pub cmds: Vec<Command>,
    pub ballot: BallotNum,
    pub seq: Sequence,
    pub deps: InstIDs,
    pub lb: LeaderBookkeeping,
}
