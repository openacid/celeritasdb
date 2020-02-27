use super::instance::Instance;
// use prost::{Message};

include!(concat!(env!("OUT_DIR"), "/message.rs"));

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

pub struct Request {}
pub struct Reply {}

/// Request is a protobuf message for all kinds of replication request.
/// See: https://github.com/openacid/celeritasdb/wiki/replication-algo#messages;
///
/// ```ignore
/// // create a `prepare` request for an instance:
/// let req = Request::prepare(some_instance);
/// ```
impl Request {
    pub fn common(inst: &Instance) -> Option<RequestCommon> {
        // TODO need filling to_replica_id
        Some(RequestCommon {
            to_replica_id: 0,
            ballot: inst.ballot,
            instance_id: inst.instance_id,
        })
    }

    pub fn fast_accept(inst: &Instance, deps_committed: &[bool]) -> FastAcceptRequest {
        FastAcceptRequest {
            cmn: Request::common(inst),
            cmds: inst.cmds.clone(),
            initial_deps: inst.initial_deps.clone(),
            deps_committed: deps_committed.into(),
        }
    }

    pub fn accept(inst: &Instance) -> AcceptRequest {
        AcceptRequest {
            cmn: Request::common(inst),
            final_deps: inst.final_deps.clone(),
        }
    }

    pub fn commit(inst: &Instance) -> CommitRequest {
        CommitRequest {
            cmn: Request::common(inst),
            cmds: inst.cmds.clone(),
            final_deps: inst.final_deps.clone(),
        }
    }

    pub fn prepare(inst: &Instance) -> PrepareRequest {
        PrepareRequest {
            cmn: Request::common(inst),
        }
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
    pub fn common(inst: &Instance) -> Option<ReplyCommon> {
        Some(ReplyCommon {
            last_ballot: inst.last_ballot,
            instance_id: inst.instance_id,
        })
    }

    pub fn fast_accept(inst: &Instance, deps_committed: &[bool]) -> FastAcceptReply {
        FastAcceptReply {
            cmn: Reply::common(inst),
            deps: inst.deps.clone(),
            deps_committed: deps_committed.into(),
        }
    }

    pub fn accept(inst: &Instance) -> AcceptReply {
        AcceptReply {
            cmn: Reply::common(inst),
        }
    }

    pub fn commit(inst: &Instance) -> CommitReply {
        CommitReply {
            cmn: Reply::common(inst),
        }
    }

    pub fn prepare(inst: &Instance) -> PrepareReply {
        PrepareReply {
            cmn: Reply::common(inst),
            deps: inst.deps.clone(),
            final_deps: inst.final_deps.clone(),
            committed: inst.committed,
        }
    }
}
