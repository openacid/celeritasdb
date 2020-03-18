use std::ops::Index;
use std::ops::{Deref, DerefMut};
use tonic;
use tonic::{Request, Response, Status};

use super::tokey::ToKey;
use derive_more;
use enum_utils;

include!(concat!(env!("OUT_DIR"), "/qpaxos.rs"));

#[cfg(test)]
mod t;

#[cfg(test)]
mod test_command;

#[cfg(test)]
mod test_instance;

pub type InstanceIdx = i64;
pub type ReplicaID = i64;

pub use q_paxos_client::*;
pub use q_paxos_server::*;

pub mod conflict;
pub use conflict::*;

pub struct MakeRequest {}
pub struct MakeReply {}

impl From<(&str, &str, &str)> for InvalidRequest {
    fn from(t: (&str, &str, &str)) -> InvalidRequest {
        InvalidRequest {
            field: t.0.into(),
            problem: t.1.into(),
            ctx: t.2.into(),
        }
    }
}

impl From<(&str, &str)> for InvalidRequest {
    fn from(t: (&str, &str)) -> InvalidRequest {
        InvalidRequest {
            field: t.0.into(),
            problem: t.1.into(),
            ctx: "".into(),
        }
    }
}

impl Command {
    pub fn of(op: OpCode, key: &[u8], value: &[u8]) -> Command {
        Command {
            op: op as i32,
            key: key.to_vec(),
            value: value.to_vec(),
        }
    }
}

impl Conflict for Command {
    /// conflict checks if two commands conflict.
    /// Two commands conflict iff: the execution order exchange, the result might be differnt.
    fn conflict(&self, with: &Self) -> bool {
        if self.op == OpCode::NoOp as i32 {
            return false;
        }

        if with.op == OpCode::NoOp as i32 {
            return false;
        }

        if self.op == OpCode::Set as i32 || with.op == OpCode::Set as i32 {
            return self.key == with.key;
        }

        false
    }
}

impl From<(OpCode, &str, &str)> for Command {
    fn from(t: (OpCode, &str, &str)) -> Command {
        Command::of(t.0, &t.1.as_bytes().to_vec(), &t.2.as_bytes().to_vec())
    }
}

impl From<(&str, &str, &str)> for Command {
    fn from(t: (&str, &str, &str)) -> Command {
        Command::of(
            t.0.parse().unwrap(),
            &t.1.as_bytes().to_vec(),
            &t.2.as_bytes().to_vec(),
        )
    }
}

impl ToKey for InstanceId {
    fn to_key(&self) -> Vec<u8> {
        format!("/instance/{:016x}/{:016x}", self.replica_id, self.idx).into_bytes()
    }
}

impl InstanceId {
    pub fn from_key(s: &str) -> Option<InstanceId> {
        let items: Vec<&str> = s.split("/").collect();
        if items[1] == "instance" && items.len() == 4 {
            let replica_id = match u64::from_str_radix(&items[2][..], 16) {
                Ok(v) => v as i64,
                Err(_) => return None,
            };

            let idx = match u64::from_str_radix(&items[3][..], 16) {
                Ok(v) => v as i64,
                Err(_) => return None,
            };

            return Some(InstanceId { replica_id, idx });
        }

        return None;
    }
}

/// Let user use method of Vec<InstanceId> directly.
impl Deref for InstanceIdVec {
    type Target = Vec<InstanceId>;
    fn deref(&self) -> &Self::Target {
        &self.ids
    }
}

/// Let user use method of Vec<InstanceId> directly.
impl DerefMut for InstanceIdVec {
    fn deref_mut(&mut self) -> &mut Vec<InstanceId> {
        &mut self.ids
    }
}

/// Let user use instance_id_vec[replic_id] to retreive an instance_id.
/// It panics if replica_id not found.
/// It returns the first match.
impl Index<ReplicaID> for InstanceIdVec {
    type Output = InstanceId;
    fn index(&self, rid: ReplicaID) -> &Self::Output {
        for inst in self.ids.iter() {
            if inst.replica_id == rid {
                return inst;
            }
        }
        panic!("NotFound instance_id with replica_id={}", rid);
    }
}

impl InstanceIdVec {
    /// get retreive an instance_id with specified replica_id.
    /// It returns the first match.
    fn get(&self, rid: ReplicaID) -> Option<InstanceId> {
        for inst in self.ids.iter() {
            if inst.replica_id == rid {
                return Some(*inst);
            }
        }
        None
    }
}

impl From<&[InstanceId]> for InstanceIdVec {
    fn from(v: &[InstanceId]) -> InstanceIdVec {
        InstanceIdVec { ids: v.into() }
    }
}

impl<A: Into<ReplicaID> + Copy, B: Into<i64> + Copy> From<&[(A, B)]> for InstanceIdVec {
    fn from(v: &[(A, B)]) -> InstanceIdVec {
        let x: Vec<InstanceId> = v
            .iter()
            .map(|x| InstanceId {
                replica_id: x.0.into(),
                idx: x.1.into(),
            })
            .collect();
        x.to_vec().into()
    }
}

impl<A> From<&[A; 0]> for InstanceIdVec {
    fn from(_v: &[A; 0]) -> InstanceIdVec {
        InstanceIdVec { ids: vec![] }
    }
}

impl<A> From<[A; 0]> for InstanceIdVec {
    fn from(_v: [A; 0]) -> InstanceIdVec {
        InstanceIdVec { ids: vec![] }
    }
}

macro_rules! impl_instance_id_vec {
    ($n:expr) => {
        impl<A: Into<ReplicaID> + Copy, B: Into<i64> + Copy> From<&[(A, B); $n]> for InstanceIdVec {
            fn from(v: &[(A, B); $n]) -> InstanceIdVec {
                let q: &[_] = v;
                q.into()
            }
        }

        impl<A: Into<ReplicaID> + Copy, B: Into<i64> + Copy> From<[(A, B); $n]> for InstanceIdVec {
            fn from(v: [(A, B); $n]) -> InstanceIdVec {
                let q: &[_] = &v;
                q.into()
            }
        }
    };
}

impl_instance_id_vec!(1);
impl_instance_id_vec!(2);
impl_instance_id_vec!(3);
impl_instance_id_vec!(4);
impl_instance_id_vec!(5);
impl_instance_id_vec!(6);
impl_instance_id_vec!(7);
impl_instance_id_vec!(8);

impl ToKey for Instance {
    fn to_key(&self) -> Vec<u8> {
        self.instance_id.as_ref().unwrap().to_key()
    }
}

impl Conflict for Instance {
    /// conflict checks if two instances conflict.
    /// Two instances conflict iff: command a from self and another command b from with conflict.
    fn conflict(&self, with: &Self) -> bool {
        for a in self.cmds.iter() {
            for b in with.cmds.iter() {
                if a.conflict(b) {
                    return true;
                }
            }
        }

        false
    }
}

impl Instance {
    pub fn of(cmds: &[Command], ballot: BallotNum, deps: &[InstanceId]) -> Instance {
        Instance {
            cmds: cmds.into(),
            ballot: Some(ballot),
            initial_deps: Some(deps.into()),
            ..Default::default()
        }
    }

    pub fn after(&self, other: &Instance) -> bool {
        let mut great = false;
        // TODO unwrap
        for iid in other.final_deps.as_ref().unwrap().iter() {
            // TODO unwrap
            let myiid = self.final_deps.as_ref().unwrap().get(iid.replica_id);
            let myiid = match myiid {
                Some(v) => v,
                None => continue,
            };

            if myiid.idx < iid.idx {
                return false;
            }
            if myiid.idx > iid.idx {
                great = true;
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
    pub fn common(to_replica_id: i64, inst: &Instance) -> Option<RequestCommon> {
        // TODO need filling to_replica_id
        Some(RequestCommon {
            to_replica_id,
            ballot: inst.ballot,
            instance_id: inst.instance_id,
        })
    }

    pub fn fast_accept(
        to_replica_id: i64,
        inst: &Instance,
        deps_committed: &[bool],
    ) -> FastAcceptRequest {
        FastAcceptRequest {
            cmn: Self::common(to_replica_id, inst),
            cmds: inst.cmds.clone(),
            initial_deps: inst.initial_deps.clone(),
            deps_committed: deps_committed.into(),
        }
    }

    pub fn accept(to_replica_id: i64, inst: &Instance) -> AcceptRequest {
        AcceptRequest {
            cmn: Self::common(to_replica_id, inst),
            final_deps: inst.final_deps.clone(),
        }
    }

    pub fn commit(to_replica_id: i64, inst: &Instance) -> CommitRequest {
        CommitRequest {
            cmn: Self::common(to_replica_id, inst),
            cmds: inst.cmds.clone(),
            final_deps: inst.final_deps.clone(),
        }
    }

    pub fn prepare(to_replica_id: i64, inst: &Instance) -> PrepareRequest {
        PrepareRequest {
            cmn: Self::common(to_replica_id, inst),
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
            // err: None,
        })
    }

    pub fn fast_accept(inst: &Instance, deps_committed: &[bool]) -> FastAcceptReply {
        FastAcceptReply {
            cmn: Self::common(inst),
            err: None,
            deps: inst.deps.clone(),
            deps_committed: deps_committed.into(),
        }
    }

    pub fn accept(inst: &Instance) -> AcceptReply {
        AcceptReply {
            cmn: Self::common(inst),
            err: None,
        }
    }

    pub fn commit(inst: &Instance) -> CommitReply {
        CommitReply {
            cmn: Self::common(inst),
            err: None,
        }
    }

    pub fn prepare(inst: &Instance) -> PrepareReply {
        PrepareReply {
            cmn: Self::common(inst),
            err: None,
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
