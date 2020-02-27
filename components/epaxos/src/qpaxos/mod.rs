use tonic;
use tonic::{Request, Response, Status};

use super::tokey::ToKey;
use derive_more;
use std::cmp::{Ord, Ordering};

include!(concat!(env!("OUT_DIR"), "/qpaxos.rs"));

#[cfg(test)]
mod t;

pub type InstanceIdx = i64;

// prost issue, it renames InstanceID to InstanceId
pub type InstanceID = InstanceId;

pub use q_paxos_client::*;
pub use q_paxos_server::*;

pub struct MakeRequest {}
pub struct MakeReply {}

impl Command {
    pub fn of(op: OpCode, key: &[u8], value: &[u8]) -> Command {
        Command {
            op: op as i32,
            key: key.to_vec(),
            value: value.to_vec(),
            ..Default::default()
        }
    }
}

impl ToKey for InstanceID {
    fn to_key(&self) -> Vec<u8> {
        format!("/instance/{:016x}/{:016x}", self.replica_id, self.idx).into_bytes()
    }
}

impl Eq for InstanceID {}

impl Ord for InstanceID {
    fn cmp(&self, other: &Self) -> Ordering {
        let _ = match self.replica_id.cmp(&other.replica_id) {
            Ordering::Greater => return Ordering::Greater,
            Ordering::Less => return Ordering::Less,
            Ordering::Equal => Ordering::Equal,
        };

        self.idx.cmp(&other.idx)
    }
}

impl PartialOrd for InstanceID {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl InstanceID {
    pub fn of(replica_id: i64, idx: i64) -> InstanceID {
        InstanceID {
            replica_id,
            idx,
            ..Default::default()
        }
    }

    pub fn of_key(s: &str) -> Option<InstanceID> {
        let items: Vec<&str> = s.split("/").collect();
        if items[1] == "instance" && items.len() == 4 {
            let rid = match items[2].parse::<i64>() {
                Ok(v) => v,
                Err(_) => return None,
            };

            let idx = match items[3].parse::<i64>() {
                Ok(v) => v,
                Err(_) => return None,
            };

            return Some(InstanceID {
                replica_id: rid,
                idx: idx,
                ..Default::default()
            });
        }

        return None;
    }
}

impl BallotNum {
    pub fn of(epoch: i32, num: i32, replica_id: i64) -> BallotNum {
        BallotNum {
            epoch,
            num,
            replica_id,
            ..Default::default()
        }
    }
}

impl ToKey for Instance {
    fn to_key(&self) -> Vec<u8> {
        self.instance_id.as_ref().unwrap().to_key()
    }
}

impl Instance {
    pub fn of(cmds: &[Command], ballot: &BallotNum, deps: &[InstanceID]) -> Instance {
        Instance {
            cmds: cmds.into(),
            ballot: Some(ballot.clone()),
            initial_deps: deps.into(),
            ..Default::default()
        }
    }

    pub fn after(&self, other: &Instance) -> bool {
        let mut great = false;
        for iid in other.final_deps.iter() {
            match self
                .final_deps
                .iter()
                .find(|x| x.replica_id == iid.replica_id)
            {
                Some(my_iid) => {
                    if my_iid.idx < iid.idx {
                        return false;
                    }

                    if my_iid.idx > iid.idx {
                        great = true;
                    }
                }
                None => continue,
            }
        }

        great
    }
}

/// MakeRequest is a protobuf message for all kinds of replication request.
/// See: https://github.com/openacid/celeritasdb/wiki/replication-algo#messages;
///
/// ```ignore
/// // create a `prepare` request for an instance:
/// let req = MakeRequest::prepare(some_instance);
/// ```
impl MakeRequest {
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
            cmn: MakeRequest::common(inst),
            cmds: inst.cmds.clone(),
            initial_deps: inst.initial_deps.clone(),
            deps_committed: deps_committed.into(),
        }
    }

    pub fn accept(inst: &Instance) -> AcceptRequest {
        AcceptRequest {
            cmn: MakeRequest::common(inst),
            final_deps: inst.final_deps.clone(),
        }
    }

    pub fn commit(inst: &Instance) -> CommitRequest {
        CommitRequest {
            cmn: MakeRequest::common(inst),
            cmds: inst.cmds.clone(),
            final_deps: inst.final_deps.clone(),
        }
    }

    pub fn prepare(inst: &Instance) -> PrepareRequest {
        PrepareRequest {
            cmn: MakeRequest::common(inst),
        }
    }
}

/// MakeReply is a protobuf message for all kinds of replication replies.
/// See: https://github.com/openacid/celeritasdb/wiki/replication-algo#messages;
///
/// ```ignore
/// // create a `prepare` request for an instance:
/// let rep = MakeReply::prepare(some_instance);
/// ```
impl MakeReply {
    pub fn common(inst: &Instance) -> Option<ReplyCommon> {
        Some(ReplyCommon {
            last_ballot: inst.last_ballot,
            instance_id: inst.instance_id,
        })
    }

    pub fn fast_accept(inst: &Instance, deps_committed: &[bool]) -> FastAcceptReply {
        FastAcceptReply {
            cmn: MakeReply::common(inst),
            deps: inst.deps.clone(),
            deps_committed: deps_committed.into(),
        }
    }

    pub fn accept(inst: &Instance) -> AcceptReply {
        AcceptReply {
            cmn: MakeReply::common(inst),
        }
    }

    pub fn commit(inst: &Instance) -> CommitReply {
        CommitReply {
            cmn: MakeReply::common(inst),
        }
    }

    pub fn prepare(inst: &Instance) -> PrepareReply {
        PrepareReply {
            cmn: MakeReply::common(inst),
            deps: inst.deps.clone(),
            final_deps: inst.final_deps.clone(),
            committed: inst.committed,
        }
    }
}

#[derive(Debug, Default)]
pub struct MyQPaxos {}

#[tonic::async_trait]
impl QPaxos for MyQPaxos {
    async fn fast_accept(
        &self,
        request: Request<FastAcceptRequest>,
    ) -> Result<Response<FastAcceptReply>, Status> {
        // TODO I did nothing but let the test pass happily
        let inst = Instance {
            ..Default::default()
        };

        let reply = MakeReply::fast_accept(&inst, &[]);
        println!("Got a request: {:?}", request);
        Ok(Response::new(reply))
    }
    async fn accept(
        &self,
        request: Request<AcceptRequest>,
    ) -> Result<Response<AcceptReply>, Status> {
        // TODO I did nothing but let the test pass happily
        let inst = Instance {
            ..Default::default()
        };

        let reply = MakeReply::accept(&inst);
        println!("Got a request: {:?}", request);
        Ok(Response::new(reply))
    }
    async fn commit(
        &self,
        request: Request<CommitRequest>,
    ) -> Result<Response<CommitReply>, Status> {
        // TODO I did nothing but let the test pass happily
        let inst = Instance {
            ..Default::default()
        };

        let reply = MakeReply::commit(&inst);
        println!("Got a request: {:?}", request);
        Ok(Response::new(reply))
    }
    async fn prepare(
        &self,
        request: Request<PrepareRequest>,
    ) -> Result<Response<PrepareReply>, Status> {
        // TODO I did nothing but let the test pass happily
        let inst = Instance {
            ..Default::default()
        };

        let reply = MakeReply::prepare(&inst);
        println!("Got a request: {:?}", request);
        Ok(Response::new(reply))
    }
}