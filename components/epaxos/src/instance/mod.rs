use super::command::Command;
use super::tokey::ToKey;
use prost::Message;
use std::cmp::{Ord, Ordering};

include!(concat!(env!("OUT_DIR"), "/instance.rs"));

#[cfg(test)]
#[path = "./tests/instance_tests.rs"]
mod tests;

pub type InstanceIdx = i64;

// prost issue, it renames InstanceID to InstanceId
pub type InstanceID = InstanceId;

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
}
