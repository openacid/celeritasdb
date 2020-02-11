use super::super::command::Command;
use protobuf::RepeatedField;

use super::super::data;

#[cfg(test)]
#[path = "./tests/instance_tests.rs"]
mod tests;

pub type InstanceIdx = i64;

/// protocol buffer serialized
// re-export struct InstIDs in data/instance.rs
pub use data::InstIDs;

impl InstIDs {
    pub fn new_instance_ids(ids: &[InstanceID]) -> InstIDs {
        let mut inst_ids = InstIDs::new();

        inst_ids.set_ids(RepeatedField::from_slice(ids));

        return inst_ids;
    }
}

// re-export struct OpCode in data/instance.rs
pub use data::InstanceID;

impl InstanceID {
    pub fn new_instance_id(replica_id: i64, idx: i64) -> InstanceID {
        let mut inst_id = InstanceID::new();

        inst_id.set_replica_id(replica_id);
        inst_id.set_idx(idx);

        return inst_id;
    }
}

// re-export enum InstanceStatus in data/instance.rs
pub use data::InstanceStatus;

// re-export struct BallotNum in data/instance.rs
pub use data::BallotNum;

impl BallotNum {
    pub fn new_ballot_num(epoch: i32, num: i32, replica_id: i64) -> BallotNum {
        let mut ballot = BallotNum::new();

        ballot.set_epoch(epoch);
        ballot.set_num(num);
        ballot.set_replica_id(replica_id);

        return ballot;
    }
}

// re-export struct Instance in data/instance.rs
pub use data::Instance;

impl Instance {
    pub fn new_instance(
        status: InstanceStatus,
        cmds: &[Command],
        ballot: &BallotNum,
        deps: &[InstanceID],
    ) -> Instance {
        let mut inst = Instance::new();

        inst.set_status(status);
        inst.set_cmds(RepeatedField::from_slice(cmds));
        inst.set_ballot(ballot.clone());
        inst.set_initial_deps(InstIDs::new_instance_ids(deps));

        return inst;
    }
}
