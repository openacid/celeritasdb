// so that downstream crate does not need use this trait to use FromStr.
pub use std::str::FromStr;

// to let user be able to call Phase::try_into() without use this trait
pub use std::convert::TryInto;

use derive_more;
use enum_utils;

use storage::AsStorageKey;

include!(concat!(env!("OUT_DIR"), "/qpaxos.rs"));

#[macro_use]
pub mod macros;

// impl Display for qpaxos data types.
pub mod conflict;
pub mod deps;
mod display;
pub mod errors;
mod instance_id_vec;
mod instance_ids;
pub mod quorums;

pub use conflict::*;
pub use deps::*;
pub use display::*;
pub use errors::*;
pub use instance_id_vec::*;
pub use instance_ids::*;
pub use macros::*;
pub use q_paxos_client::*;
pub use q_paxos_server::*;
pub use quorums::*;
pub use value::*;

#[cfg(test)]
mod t;
#[cfg(test)]
mod test_command;
#[cfg(test)]
mod test_deps;
#[cfg(test)]
mod test_display;
#[cfg(test)]
mod test_errors;
#[cfg(test)]
mod test_instance;
#[cfg(test)]
mod test_instance_id_as_key;
#[cfg(test)]
mod test_instance_id_vec;
#[cfg(test)]
mod test_instance_ids;
#[cfg(test)]
mod test_macros;
#[cfg(test)]
mod test_quorums;
#[cfg(test)]
mod test_record;
#[cfg(test)]
mod test_value;

pub type InstanceIdx = i64;
pub type ReplicaId = i64;

pub struct MakeRequest {}

pub enum ReplicaStatus {
    Exec,
    MaxInstance,
}

// TODO test
impl AsStorageKey for ReplicaStatus {
    fn into_key(&self) -> Vec<u8> {
        // TODO: use reference to reduce memory copy.
        match self {
            ReplicaStatus::Exec => "/exec".into(),
            ReplicaStatus::MaxInstance => "/max_inst".into(),
        }
    }

    fn key_len(&self) -> usize {
        // TODO: draft impl
        self.into_key().len()
    }

    fn from_key(buf: &[u8]) -> Self {
        if buf == "/exec".as_bytes() {
            ReplicaStatus::Exec
        } else if buf == "/max_inst".as_bytes() {
            ReplicaStatus::MaxInstance
        } else {
            panic!("invalid")
        }
    }
}

#[derive(Debug, Eq, PartialEq, enum_utils::FromStr)]
pub enum Direction {
    Request,
    Reply,
}

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
    /// kind returns one of there kinds of command: NoOp, Get, Set.
    /// In this way `Delete` is a `Set` kind command because it set the value to NULL.
    pub fn kind(&self) -> OpCode {
        if self.op == OpCode::NoOp as i32 {
            OpCode::NoOp
        } else if self.op == OpCode::Get as i32 {
            OpCode::Get
        } else {
            OpCode::Set
        }
    }
}

impl Conflict for Command {
    /// conflict checks if two commands conflict.
    /// Two commands conflict iff: the execution order exchange, the result might be differnt.
    fn conflict(&self, with: &Self) -> bool {
        match (self.kind(), with.kind()) {
            (OpCode::NoOp, _) => false,
            (_, OpCode::NoOp) => false,
            (OpCode::Get, OpCode::Get) => false,
            _ => self.key == with.key,
        }
    }
}

impl From<(OpCode, &[u8], &[u8])> for Command {
    fn from(t: (OpCode, &[u8], &[u8])) -> Command {
        Command {
            op: t.0 as i32,
            key: t.1.to_vec(),
            value: t.2.to_vec(),
        }
    }
}

impl From<(OpCode, &str, &str)> for Command {
    fn from(t: (OpCode, &str, &str)) -> Command {
        Command {
            op: t.0 as i32,
            key: t.1.as_bytes().to_vec(),
            value: t.2.as_bytes().to_vec(),
        }
    }
}

impl From<(&str, &str, &str)> for Command {
    fn from(t: (&str, &str, &str)) -> Command {
        Command {
            op: OpCode::from_str(t.0).unwrap() as i32,
            key: t.1.as_bytes().to_vec(),
            value: t.2.as_bytes().to_vec(),
        }
    }
}

// TODO test
impl PartialEq<InstanceId> for Dep {
    fn eq(&self, other: &InstanceId) -> bool {
        self.replica_id == other.replica_id && self.idx == other.idx
    }
}

impl<A: Into<ReplicaId> + Copy, B: Into<i64> + Copy> From<(A, B)> for Dep {
    fn from(t: (A, B)) -> Dep {
        Dep {
            replica_id: t.0.into(),
            idx: t.1.into(),
            ..Default::default()
        }
    }
}

impl<A: Into<ReplicaId> + Copy, B: Into<i64> + Copy, S: Into<i64> + Copy> From<(A, B, S)> for Dep {
    fn from(t: (A, B, S)) -> Dep {
        Dep {
            replica_id: t.0.into(),
            idx: t.1.into(),
            seq: t.2.into(),
            ..Default::default()
        }
    }
}

impl<A: Into<ReplicaId> + Copy, B: Into<i64> + Copy> From<&(A, B)> for Dep {
    fn from(t: &(A, B)) -> Dep {
        Dep {
            replica_id: t.0.into(),
            idx: t.1.into(),
            ..Default::default()
        }
    }
}

impl<A: Into<ReplicaId> + Copy, B: Into<i64> + Copy, S: Into<i64> + Copy> From<&(A, B, S)> for Dep {
    fn from(t: &(A, B, S)) -> Dep {
        Dep {
            replica_id: t.0.into(),
            idx: t.1.into(),
            seq: t.2.into(),
            ..Default::default()
        }
    }
}

impl From<&InstanceId> for Dep {
    fn from(iid: &InstanceId) -> Dep {
        Dep {
            replica_id: iid.replica_id,
            idx: iid.idx,
            ..Default::default()
        }
    }
}

// TODO test
impl AsStorageKey for InstanceId {
    fn into_key(&self) -> Vec<u8> {
        if self.idx < 0 {
            panic!("idx can not be less than 0:{}", self.idx);
        }
        format!("{:016x}/{:016x}", self.replica_id, self.idx).into_bytes()
    }

    fn key_len(&self) -> usize {
        16 + 1 + 16
    }

    fn from_key(buf: &[u8]) -> Self {
        let ridbuf = &buf[0..16];
        let ridstr = String::from_utf8_lossy(ridbuf);
        let rid = u64::from_str_radix(&ridstr, 16);
        let rid = match rid {
            Ok(v) => v,
            Err(_) => {
                panic!("invalid key replica id");
            }
        };
        // 1 for a "/" between replica id and idx
        let idxbuf = &buf[16 + 1..32 + 1];
        let idxstr = String::from_utf8_lossy(idxbuf);
        let idx = u64::from_str_radix(&idxstr, 16);
        let idx = match idx {
            Ok(v) => v,
            Err(e) => {
                panic!("invalid key idx: {:?}", e);
            }
        };
        InstanceId {
            replica_id: rid as i64,
            idx: idx as i64,
        }
    }
}

impl<A: Into<ReplicaId> + Copy, B: Into<i64> + Copy> From<(A, B)> for InstanceId {
    fn from(t: (A, B)) -> InstanceId {
        InstanceId {
            replica_id: t.0.into(),
            idx: t.1.into(),
        }
    }
}

impl<A: Into<ReplicaId> + Copy, B: Into<i64> + Copy> From<&(A, B)> for InstanceId {
    fn from(t: &(A, B)) -> InstanceId {
        InstanceId {
            replica_id: t.0.into(),
            idx: t.1.into(),
        }
    }
}

impl AsStorageKey for Instance {
    fn into_key(&self) -> Vec<u8> {
        self.instance_id.as_ref().unwrap().into_key()
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
    pub fn of(cmds: &[Command], ballot: BallotNum, deps: &[Dep]) -> Instance {
        Instance {
            cmds: cmds.into(),
            ballot: Some(ballot),
            deps: Some(deps.into()),
            ..Default::default()
        }
    }

    pub fn after(&self, other: &Instance) -> bool {
        let mut great = false;
        // TODO unwrap
        for iid in other.deps.as_ref().unwrap().iter() {
            // TODO unwrap
            let myiid = self.deps.as_ref().unwrap().get(iid.replica_id);
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
impl From<&str> for ValueEnum {
    fn from(t: &str) -> Self {
        ValueEnum::Vbytes(t.as_bytes().into())
    }
}

impl<T: Into<ValueEnum>> From<T> for Value {
    fn from(t: T) -> Self {
        Value {
            value_enum: Some(t.into()),
        }
    }
}

impl Value {
    pub fn to_vec(&self) -> Vec<u8> {
        let value = self.value_enum.as_ref().unwrap();
        match value {
            ValueEnum::Vi64(v) => format!("{}", v).as_bytes().into(),
            ValueEnum::Vbytes(v) => v.clone(),
        }
    }
}

impl<T: Into<Value>> From<T> for Record {
    fn from(t: T) -> Record {
        Record {
            value: Some(t.into()),
        }
    }
}

impl Record {
    pub fn to_vec(&self) -> Vec<u8> {
        self.value.as_ref().unwrap().to_vec()
    }
}

macro_rules! make_req {
    ($to_replica_id:expr, $inst:expr, $phase:expr) => {
        ReplicateRequest {
            to_replica_id: $to_replica_id,
            ballot: $inst.ballot,
            instance_id: $inst.instance_id,
            phase: Some($phase.into()),
        }
    };
}

/// MakeRequest is a protobuf message for all kinds of replication request.
/// See: https://github.com/openacid/celeritasdb/wiki/replication-algo#messages;
///
/// ```ignore
/// // create a `prepare` request for an instance:
/// let req = MakeRequest::prepare(1, some_instance);
/// ```
impl MakeRequest {
    pub fn prepare(
        to_replica_id: i64,
        inst: &Instance,
        deps_committed: &[bool],
    ) -> ReplicateRequest {
        let p = PrepareRequest {
            cmds: inst.cmds.clone(),
            deps: inst.deps.clone(),
            deps_committed: deps_committed.into(),
        };
        make_req!(to_replica_id, inst, p)
    }

    pub fn accept(to_replica_id: i64, inst: &Instance) -> ReplicateRequest {
        let p = AcceptRequest {
            deps: inst.deps.clone(),
        };
        make_req!(to_replica_id, inst, p)
    }

    pub fn commit(to_replica_id: i64, inst: &Instance) -> ReplicateRequest {
        let p = CommitRequest {
            cmds: inst.cmds.clone(),
            deps: inst.deps.clone(),
        };
        make_req!(to_replica_id, inst, p)
    }
}
