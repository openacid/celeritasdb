//! all messages must be serialized by protocol-buffer
//! so define it in rust first.

use super::super::instance::{InstIDs, Instance, InstanceID, InstanceStatus, Sequence};
use super::super::replica::{BallotNum, ReplicaID};

#[cfg(test)]
#[path = "./tests/message_tests.rs"]
mod tests;

pub enum RequestType {
    Prepare,
    TryPreAccept,
    PreAccept,
    Accept,
    Commit,
}

pub enum MessageType {
    Request,
    Reply,
}

/// protocol message wrapper used in transmission
pub struct Message {
    pub req_type: RequestType,
    pub msg_type: MessageType,
    pub msg: Vec<u8>, // rust-protobuf use Vec<u8> as the type of serialization result
}

/// used in Explict Prepare
/// (1) recovery
/// (2) execute-thread exceeds timeout, and can be considered as recovery too
pub struct PrepareReq {
    pub leader_id: ReplicaID,  // the replica owns the instance
    pub replica_id: ReplicaID, // the replica who starts the explict prepare
    pub instance_id: InstanceID,
    pub ballot: BallotNum,
}

pub struct PrepareReply {
    pub acceptor_id: ReplicaID,
    pub replica_id: ReplicaID, // the replica who starts the explict prepare
    pub instance_id: InstanceID,
    pub ok: bool, // successful(true), or failed(false) in case ballot number of msg smaller than that in instance
    pub ballot: BallotNum,
    pub instance: Instance,
}

// used in Phase-1
pub struct PreAcceptReq {
    pub leader_id: ReplicaID,
    pub replica_id: ReplicaID, // the same value with leader_id, weird!
    pub instance_id: InstanceID,
    pub instance: Instance,
    pub ballot: BallotNum,
}

pub struct PreAcceptReply {
    pub replica_id: ReplicaID,
    pub instance: Instance,
    pub ok: bool,
    pub ballot: BallotNum,
    pub committed_deps: InstIDs,
}

// TODO(lsl): a type contains Instance and InstanceID may be used widely
// used in Paxos-Accept
pub struct AcceptReq {
    pub leader_id: ReplicaID,  // the replica who sends the msg
    pub replica_id: ReplicaID, // the replica who starts the explicit_prepare or who starts the preaccept or who starts the try-preaccept
    pub instance: Instance,
    pub ballot: BallotNum,
    pub count: i32,
}

pub struct AcceptReply {
    pub replica_id: ReplicaID,
    pub instance_id: InstanceID,
    pub ok: bool,
    pub ballot: BallotNum,
}

// used in commit phase
pub struct CommitReq {
    pub leader_id: ReplicaID,
    pub replica_id: ReplicaID,
    pub instance: Instance,
}

pub struct CommitShort {
    pub leader_id: ReplicaID,
    pub replica_id: ReplicaID,
    pub instance_id: InstanceID,
    pub count: i32,

    pub seq: Sequence,
    pub deps: InstIDs,
}

pub struct TryPreAcceptReq {
    pub leader_id: ReplicaID,
    pub replica_id: ReplicaID,
    pub instance_id: InstanceID,
    pub ballot: BallotNum,
    pub instance: Instance,
}

pub struct TryPreAcceptReply {
    pub acceptor_id: ReplicaID,
    pub replica_id: ReplicaID,
    pub instance_id: InstanceID,
    pub ok: bool,
    pub ballot: BallotNum,
    pub conflict_replica: ReplicaID,
    pub conflict_instance_id: InstanceID,
    pub conflict_status: InstanceStatus,
}
