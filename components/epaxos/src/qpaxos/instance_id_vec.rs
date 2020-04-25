pub use std::cmp::Ordering;
use std::ops::Index;
use std::ops::{Deref, DerefMut};

use crate::qpaxos::InstanceId;
use crate::qpaxos::InstanceIdVec;
use crate::qpaxos::ReplicaId;

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
impl Index<ReplicaId> for InstanceIdVec {
    type Output = InstanceId;
    fn index(&self, rid: ReplicaId) -> &Self::Output {
        for inst in self.ids.iter() {
            if inst.replica_id == rid {
                return inst;
            }
        }
        panic!("NotFound instance_id with replica_id={}", rid);
    }
}

impl PartialEq<InstanceId> for InstanceIdVec {
    fn eq(&self, other: &InstanceId) -> bool {
        self.get(other.replica_id) == Some(*other)
    }
}

impl PartialOrd<InstanceId> for InstanceIdVec {
    fn partial_cmp(&self, other: &InstanceId) -> Option<Ordering> {
        let mine = self.get(other.replica_id);
        match mine {
            Some(iid) => Some(iid.cmp(other)),
            None => Some(Ordering::Less),
        }
    }
}

impl InstanceIdVec {
    /// get retreive an instance_id with specified replica_id.
    /// It returns the first match.
    pub fn get(&self, rid: ReplicaId) -> Option<InstanceId> {
        for iid in self.ids.iter() {
            if iid.replica_id == rid {
                return Some(*iid);
            }
        }
        None
    }

    /// set add an instanceId into it or overrides an existing one with the same replicaId.
    /// It returns the index the added InstanceId is, along with an Option of the replaced value.
    pub fn set(&mut self, inst_id: InstanceId) -> (usize, Option<InstanceId>) {
        let mut idx: i64 = -1;
        for (i, iid) in self.ids.iter().enumerate() {
            if iid.replica_id == inst_id.replica_id {
                idx = i as i64;
                break;
            }
        }

        if idx == -1 {
            let l = self.ids.len();
            self.ids.push(inst_id);
            (l, None)
        } else {
            let old = self.ids[idx as usize];
            self.ids[idx as usize] = inst_id;
            (idx as usize, Some(old))
        }
    }
}

impl From<&[InstanceId]> for InstanceIdVec {
    fn from(v: &[InstanceId]) -> InstanceIdVec {
        InstanceIdVec { ids: v.into() }
    }
}

impl<A: Into<ReplicaId> + Copy, B: Into<i64> + Copy> From<&[(A, B)]> for InstanceIdVec {
    fn from(v: &[(A, B)]) -> InstanceIdVec {
        v.iter()
            .map(|x| x.into())
            .collect::<Vec<InstanceId>>()
            .into()
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
        impl<A: Into<ReplicaId> + Copy, B: Into<i64> + Copy> From<&[(A, B); $n]> for InstanceIdVec {
            fn from(v: &[(A, B); $n]) -> InstanceIdVec {
                let q: &[_] = v;
                q.into()
            }
        }

        impl<A: Into<ReplicaId> + Copy, B: Into<i64> + Copy> From<[(A, B); $n]> for InstanceIdVec {
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
