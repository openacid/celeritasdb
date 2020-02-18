use super::super::command::Command;
use super::super::tokey::ToKey;
use protobuf::RepeatedField;
use protobuf::SingularPtrField;

use super::super::data;

#[cfg(test)]
#[path = "./tests/instance_tests.rs"]
mod tests;

pub type InstanceIdx = i64;

// re-export struct OpCode in data/instance.rs
pub use data::InstanceID;

impl ToKey for InstanceID {
    fn to_key(&self) -> Vec<u8> {
        format!("/instance/{:016x}/{:016x}", self.replica_id, self.idx).into_bytes()
    }
}

impl InstanceID {
    pub fn of(replica_id: i64, idx: i64) -> InstanceID {
        InstanceID {
            replica_id: replica_id,
            idx: idx,
            ..Default::default()
        }
    }
}

// re-export enum InstanceStatus in data/instance.rs
pub use data::InstanceStatus;

// re-export struct BallotNum in data/instance.rs
pub use data::BallotNum;

impl BallotNum {
    pub fn of(epoch: i32, num: i32, replica_id: i64) -> BallotNum {
        BallotNum {
            epoch: epoch,
            num: num,
            replica_id: replica_id,
            ..Default::default()
        }
    }
}

// re-export struct Instance in data/instance.rs
pub use data::Instance;

impl ToKey for Instance {
    fn to_key(&self) -> Vec<u8> {
        self.instance_id.get_ref().to_key()
    }
}

impl Instance {
    pub fn of(cmds: &[Command], ballot: &BallotNum, deps: &[InstanceID]) -> Instance {
        Instance {
            cmds: cmds.into(),
            ballot: SingularPtrField::some(ballot.clone()),
            initial_deps: deps.into(),
            ..Default::default()
        }
    }
}
