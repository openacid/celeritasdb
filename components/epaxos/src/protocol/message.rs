//! all messages must be serialized by protocol-buffer
//! so define it in rust first.

enum RequestType {
    Prepare,
    TryPreAccept,
    PreAccept,
    Accept,
    Commit,
}

enum MsgType {
    Request,
    Reply,
}

/// protocol msg wrapper used in transmission
pub struct Message {
    req_type: RequestType,
    msg_type: MsgType,
    msg: Vec<u8>, // rust-protobuf use Vec<u8> as the type of serialization result
}

/// used in Explict Prepare
/// (1) recovery
/// (2) execute-thread exceeds timeout, and can be considered as recovery too
struct PrepareReq {
    leader_id: ReplicaID, // the replica owns the instance
    replica:   ReplicaID, // the replica who starts the explict prepare
    instance_id: InstanceID,
    ballot: BallotNum,
}

struct PrepareReply {
    acceptor_id: ReplicaID,
    replica: ReplicaID, // the replica who starts the explict prepare
    instance_id: InstanceID,
    ok: bool, // successful(true), or failed(false) in case ballot number of msg smaller than that in instance
    ballot: BallotNum,
    instance: Instance,
}

// used in Phase-1
struct PreAcceptReq {
    leader_id: ReplicaID,
    replica: ReplicaID, // the same value with leader_id, weird!
    instance_id: InstanceID,
    instance: Instance,
    ballot: BallotNum,
}

struct PreAcceptReply {
    replica: ReplicaID,
    instance: Instance,
    ok: bool,
    ballot: BallotNum,
    committed_deps: DepCmds,
}

// TODO(lsl): a type contains Instance and InstanceID may be used widely
// used in Paxos-Accept
struct AcceptReq {
    leader_id: ReplicaID, // the replica who sends the msg
    replica: ReplicaID, // the replica who starts the explicit_prepare or who starts the preaccept or who starts the try-preaccept
    instance: Instance,
    ballot: BallotNum,
    count: int32,
}

struct AcceptReply {
    replica: ReplicaID,
    instance_id: instanceId,
    ok: bool,
    ballot: BallotNum,
}

// used in commit phase
struct CommitReq {
    leader_id: ReplicaID,
    replica: ReplicaID,
    instance: Instance,
}

struct CommitShort {
    leader_id: ReplicaID,
    replica: ReplicaID,
    instance_id: InstanceID,
    count: int32,

    seq: int32,
    deps: Vec<(ReplicaID, InstanceID)>,
}

struct TryPreAcceptReq {
    leader_id: ReplicaID,
    replica: ReplicaID,
    instance_id: InstanceID,
    ballot: BallotNum,
    instance: Instance,
}

struct TryPreAcceptReply {
    acceptor_id: ReplicaID,
    replica: ReplicaID,
    instance_id: InstanceID,
    ok: bool,
    ballot: BallotNum,
    conflict_replica: ReplicaID,
    conflict_instance_id: InstanceID,
    conflict_status: InstanceStatus,
}
