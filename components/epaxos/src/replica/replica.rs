use super::super::instance;

#[cfg(test)]
#[path = "./tests/replica_tests.rs"]
mod tests;

type ReplicaID = i64;

pub struct BallotNum {
    pub epoch: u32,
    pub num: u32,
    pub replica_id: ReplicaID,
}

pub struct Replica {
    inst: instance::Instance,
}
