use super::instance::Instance;
// use prost::{Message};

include!(concat!(env!("OUT_DIR"), "/message.rs"));
// include!(concat!(
//     env!("CARGO_MANIFEST_DIR"),
//     "/data/qpaxos.message.rs"
// ));

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
            req_type: t.into(),
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
            req_type: t.into(),
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
