pub use std::cmp::Ordering;
use std::ops::Index;
use std::ops::{Deref, DerefMut};

use crate::qpaxos::Dep;
use crate::qpaxos::DepVec;
use crate::qpaxos::InstanceId;
use crate::qpaxos::ReplicaId;

/// Let user use method of Vec<Dep> directly.
impl Deref for DepVec {
    type Target = Vec<Dep>;
    fn deref(&self) -> &Self::Target {
        &self.ids
    }
}

/// Let user use method of Vec<Dep> directly.
impl DerefMut for DepVec {
    fn deref_mut(&mut self) -> &mut Vec<Dep> {
        &mut self.ids
    }
}

/// Let user use instance_id_vec[replic_id] to retreive a Dep.
/// It panics if replica_id not found.
/// It returns the first match.
impl Index<ReplicaId> for DepVec {
    type Output = Dep;
    fn index(&self, rid: ReplicaId) -> &Self::Output {
        for inst in self.ids.iter() {
            if inst.replica_id == rid {
                return inst;
            }
        }
        panic!("NotFound dep with replica_id={}", rid);
    }
}

impl PartialEq<Dep> for DepVec {
    fn eq(&self, other: &Dep) -> bool {
        self.get(other.replica_id) == Some(*other)
    }
}

impl PartialOrd<Dep> for DepVec {
    fn partial_cmp(&self, other: &Dep) -> Option<Ordering> {
        let mine = self.get(other.replica_id);
        match mine {
            Some(iid) => Some(iid.cmp(other)),
            None => Some(Ordering::Less),
        }
    }
}

impl PartialEq<InstanceId> for DepVec {
    fn eq(&self, other: &InstanceId) -> bool {
        let mine = self.get(other.replica_id);
        match mine {
            Some(iid) => iid.replica_id == other.replica_id && iid.idx == other.idx,
            None => false,
        }
    }
}

impl PartialOrd<InstanceId> for DepVec {
    fn partial_cmp(&self, other: &InstanceId) -> Option<Ordering> {
        let mine = self.get(other.replica_id);
        match mine {
            Some(iid) => Some(iid.idx.cmp(&other.idx)),
            None => Some(Ordering::Less),
        }
    }
}

impl DepVec {
    /// get retreive a Dep with specified replica_id.
    /// It returns the first match.
    pub fn get(&self, rid: ReplicaId) -> Option<Dep> {
        for iid in self.ids.iter() {
            if iid.replica_id == rid {
                return Some(*iid);
            }
        }
        None
    }

    /// set add an instanceId into it or overrides an existing one with the same replicaId.
    /// It returns the index the added Dep is, along with an Option of the replaced value.
    pub fn set(&mut self, inst_id: Dep) -> (usize, Option<Dep>) {
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

impl From<&[Dep]> for DepVec {
    fn from(v: &[Dep]) -> DepVec {
        DepVec { ids: v.into() }
    }
}

impl From<&[InstanceId]> for DepVec {
    fn from(v: &[InstanceId]) -> DepVec {
        let mut depvec: Vec<Dep> = Vec::with_capacity(v.len());
        for iid in v.iter() {
            depvec.push(iid.into());
        }
        depvec.into()
    }
}

impl From<Vec<InstanceId>> for DepVec {
    fn from(v: Vec<InstanceId>) -> DepVec {
        let mut depvec: Vec<Dep> = Vec::with_capacity(v.len());
        for iid in v.iter() {
            depvec.push(iid.into());
        }
        depvec.into()
    }
}

impl<A: Into<ReplicaId> + Copy, B: Into<i64> + Copy> From<&[(A, B)]> for DepVec {
    fn from(v: &[(A, B)]) -> DepVec {
        v.iter().map(|x| x.into()).collect::<Vec<Dep>>().into()
    }
}

impl<A> From<&[A; 0]> for DepVec {
    fn from(_v: &[A; 0]) -> DepVec {
        DepVec { ids: vec![] }
    }
}

impl<A> From<[A; 0]> for DepVec {
    fn from(_v: [A; 0]) -> DepVec {
        DepVec { ids: vec![] }
    }
}

macro_rules! impl_instance_id_vec {
    ($n:expr) => {
        impl<A: Into<ReplicaId> + Copy, B: Into<i64> + Copy> From<&[(A, B); $n]> for DepVec {
            fn from(v: &[(A, B); $n]) -> DepVec {
                let q: &[_] = v;
                q.into()
            }
        }

        impl<A: Into<ReplicaId> + Copy, B: Into<i64> + Copy> From<[(A, B); $n]> for DepVec {
            fn from(v: [(A, B); $n]) -> DepVec {
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
