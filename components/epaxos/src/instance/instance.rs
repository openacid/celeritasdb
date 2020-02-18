use super::super::command::Command;
use protobuf::RepeatedField;
use protobuf::SingularPtrField;

use super::super::data;

#[cfg(test)]
#[path = "./tests/instance_tests.rs"]
mod tests;

pub type InstanceIdx = i64;

// re-export struct OpCode in data/instance.rs
pub use data::InstanceID;

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
