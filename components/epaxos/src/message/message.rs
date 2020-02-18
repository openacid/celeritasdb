use super::super::data;
use super::super::instance::{BallotNum, Instance, InstanceID, InstanceStatus};
use super::super::replica::ReplicaID;
use protobuf::RepeatedField;

#[cfg(test)]
#[path = "./tests/message_tests.rs"]
mod tests;

pub enum MsgStatus {
    Ok,
    Failed,
}

impl From<MsgStatus> for bool {
    fn from(ms: MsgStatus) -> bool {
        match ms {
            MsgStatus::Ok => true,
            MsgStatus::Failed => false,
        }
    }
}

// re-export enum RequestType in data/message.rs
pub use data::RequestType;

pub use data::Command;
/// protocol message wrapper used in transmission
pub use data::Reply;
pub use data::Request;

/// Request is a protobuf message for all kinds of replication request.
/// See: https://github.com/openacid/celeritasdb/wiki/replication-algo#messages;
///
/// ```ignore
/// // create a `prepare` request for an instance:
/// let req = Request::prepare(some_instance);
/// ```
impl Request {
    pub fn of_instance(inst: &Instance, t: RequestType) -> Self {
        Self {
            req_type: t,
            ballot: inst.ballot.clone(),
            instance_id: inst.instance_id.clone(),
            ..Default::default()
        }
    }

    pub fn preaccept(inst: &Instance, deps_committed: &[bool]) -> Self {
        Self {
            cmds: inst.cmds.clone(),
            initial_deps: inst.initial_deps.clone(),
            deps_committed: deps_committed.into(),
            ..Self::of_instance(inst, RequestType::PreAccept)
        }
    }

    pub fn accept(inst: &Instance) -> Self {
        Self {
            final_deps: inst.final_deps.clone(),
            ..Self::of_instance(inst, RequestType::Accept)
        }
    }

    pub fn commit(inst: &Instance) -> Self {
        Self {
            cmds: inst.cmds.clone(),
            final_deps: inst.final_deps.clone(),
            ..Self::of_instance(inst, RequestType::Commit)
        }
    }

    pub fn prepare(inst: &Instance) -> Self {
        Self::of_instance(inst, RequestType::Prepare)
    }
}

/// Reply is a protobuf message for all kinds of replication replies.
/// See: https://github.com/openacid/celeritasdb/wiki/replication-algo#messages;
///
/// ```ignore
/// // create a `prepare` request for an instance:
/// let rep = Reply::prepare(some_instance);
/// ```
impl Reply {
    pub fn of_instance(inst: &Instance, t: RequestType) -> Self {
        Self {
            req_type: t,
            last_ballot: inst.last_ballot.clone(),
            instance_id: inst.instance_id.clone(),
            ..Default::default()
        }
    }

    pub fn preaccept(inst: &Instance, deps_committed: &[bool]) -> Self {
        Self {
            deps: inst.deps.clone(),
            deps_committed: deps_committed.into(),
            ..Self::of_instance(inst, RequestType::PreAccept)
        }
    }

    pub fn accept(inst: &Instance) -> Self {
        Self::of_instance(inst, RequestType::Accept)
    }

    pub fn commit(inst: &Instance) -> Self {
        Self::of_instance(inst, RequestType::Commit)
    }

    pub fn prepare(inst: &Instance) -> Self {
        Self {
            deps: inst.deps.clone(),
            final_deps: inst.final_deps.clone(),
            committed: inst.committed,
            ..Self::of_instance(inst, RequestType::Prepare)
        }
    }
}

/// used in Explict Prepare
/// (1) recovery
/// (2) execute-thread exceeds timeout, and can be considered as recovery too
// re-export struct PrepareReq in data/message.rs
pub use data::PrepareReq;
impl PrepareReq {
    // inst_id and ballot are moved, clone them outside if necessary
    pub fn new_prepare_req(
        leader: ReplicaID,
        replica: ReplicaID,
        inst_id: InstanceID,
        ballot: BallotNum,
    ) -> PrepareReq {
        let mut pr = PrepareReq::new();

        pr.set_leader_id(leader);
        pr.set_replica_id(replica);
        pr.set_instance_id(inst_id);
        pr.set_ballot(ballot);

        return pr;
    }
}

// re-export struct PrepareReply in data/message.rs
pub use data::PrepareReply;

impl PrepareReply {
    // inst_id, ballot and inst are moved, clone them outside if necessary
    pub fn new_prepare_reply(
        acceptor: ReplicaID,
        replica: ReplicaID,
        inst_id: InstanceID,
        ballot: BallotNum,
        ok: MsgStatus,
        inst: Instance,
    ) -> PrepareReply {
        let mut pr = PrepareReply::new();

        pr.set_acceptor_id(acceptor);
        pr.set_replica_id(replica);
        pr.set_instance_id(inst_id);
        pr.set_ballot(ballot);
        pr.set_ok(bool::from(ok));
        pr.set_instance(inst);

        return pr;
    }
}

// used in Phase-1
// re-export struct PreAcceptReq in data/message.rs
pub use data::PreAcceptReq;

impl PreAcceptReq {
    // inst_id, ballot and inst are moved, clone them outside if necessary
    pub fn new_pre_accept_req(
        leader: ReplicaID,
        replica: ReplicaID,
        inst_id: InstanceID,
        inst: Instance,
        ballot: BallotNum,
    ) -> PreAcceptReq {
        let mut pa_req = PreAcceptReq::new();

        pa_req.set_leader_id(leader);
        pa_req.set_replica_id(replica);
        pa_req.set_instance_id(inst_id);
        pa_req.set_instance(inst);
        pa_req.set_ballot(ballot);

        return pa_req;
    }
}

// re-export struct PreAcceptReply in data/message.rs
pub use data::PreAcceptReply;

impl PreAcceptReply {
    // inst, ballot are moved, clone them outside if necessary
    pub fn new_pre_accept_reply(
        replica: ReplicaID,
        inst: Instance,
        ok: MsgStatus,
        ballot: BallotNum,
        committed_deps: &[InstanceID],
    ) -> PreAcceptReply {
        let mut pa_reply = PreAcceptReply::new();

        pa_reply.set_replica_id(replica);
        pa_reply.set_instance(inst);
        pa_reply.set_ok(bool::from(ok));
        pa_reply.set_ballot(ballot);
        pa_reply.set_committed_deps(RepeatedField::from_slice(committed_deps));

        return pa_reply;
    }
}

// used in Paxos-Accept
// re-export struct AcceptReq in data/message.rs
pub use data::AcceptReq;

impl AcceptReq {
    // inst, ballot are moved, clone them outside if necessary
    pub fn new_accept_req(
        leader: ReplicaID,
        replica: ReplicaID,
        inst: Instance,
        ballot: BallotNum,
        count: i32,
    ) -> AcceptReq {
        let mut ar = AcceptReq::new();

        ar.set_leader_id(leader);
        ar.set_replica_id(replica);
        ar.set_instance(inst);
        ar.set_ballot(ballot);
        ar.set_count(count);

        return ar;
    }
}

// re-export struct AcceptReply in data/message.rs
pub use data::AcceptReply;

impl AcceptReply {
    // inst_id, ballot are moved, clone them outside if necessary
    pub fn new_accept_reply(
        replica: ReplicaID,
        inst_id: InstanceID,
        ok: MsgStatus,
        ballot: BallotNum,
    ) -> AcceptReply {
        let mut ar = AcceptReply::new();

        ar.set_replica_id(replica);
        ar.set_instance_id(inst_id);
        ar.set_ok(bool::from(ok));
        ar.set_ballot(ballot);

        return ar;
    }
}

// used in commit phase
// re-export struct CommitReq in data/message.rs
pub use data::CommitReq;

impl CommitReq {
    // inst is moved, clone them outside if necessary
    pub fn new_commit_req(leader: ReplicaID, replica: ReplicaID, inst: Instance) -> CommitReq {
        let mut cr = CommitReq::new();

        cr.set_leader_id(leader);
        cr.set_replica_id(replica);
        cr.set_instance(inst);

        return cr;
    }
}

// re-export struct CommitShort in data/message.rs
pub use data::CommitShort;

impl CommitShort {
    // inst_id is moved, clone them outside if necessary
    pub fn new_commit_short(
        leader: ReplicaID,
        replica: ReplicaID,
        inst_id: InstanceID,
        count: i32,
        seq: i32,
        deps: &[InstanceID],
    ) -> CommitShort {
        let mut cs = CommitShort::new();

        cs.set_leader_id(leader);
        cs.set_replica_id(replica);
        cs.set_instance_id(inst_id);
        cs.set_count(count);
        cs.set_seq(seq);
        cs.set_deps(RepeatedField::from_slice(deps));

        return cs;
    }
}

// re-export struct TryPreAcceptReq in data/message.rs
pub use data::TryPreAcceptReq;

impl TryPreAcceptReq {
    // inst_id, ballot and inst are moved, clone them outside if necessary
    pub fn new_try_pre_accept_req(
        leader: ReplicaID,
        replica: ReplicaID,
        inst_id: InstanceID,
        ballot: BallotNum,
        inst: Instance,
    ) -> TryPreAcceptReq {
        let mut tpa_req = TryPreAcceptReq::new();

        tpa_req.set_leader_id(leader);
        tpa_req.set_replica_id(replica);
        tpa_req.set_instance_id(inst_id);
        tpa_req.set_ballot(ballot);
        tpa_req.set_instance(inst);

        return tpa_req;
    }
}

// re-export struct TryPreAcceptReply in data/message.rs
pub use data::TryPreAcceptReply;

impl TryPreAcceptReply {
    // inst_id, ballot and conflict_inst_id :are moved, clone them outside if necessary
    pub fn new_try_pre_accept_reply(
        acceptor: ReplicaID,
        replica: ReplicaID,
        inst_id: InstanceID,
        ok: MsgStatus,
        ballot: BallotNum,
        conflict_replica: ReplicaID,
        conflict_inst_id: InstanceID,
        conflict_status: InstanceStatus,
    ) -> TryPreAcceptReply {
        let mut tpa_reply = TryPreAcceptReply::new();

        tpa_reply.set_acceptor_id(acceptor);
        tpa_reply.set_replica_id(replica);
        tpa_reply.set_instance_id(inst_id);
        tpa_reply.set_ok(bool::from(ok));
        tpa_reply.set_ballot(ballot);
        tpa_reply.set_conflict_replica(conflict_replica);
        tpa_reply.set_conflict_instance_id(conflict_inst_id);
        tpa_reply.set_conflict_status(conflict_status);

        return tpa_reply;
    }
}
