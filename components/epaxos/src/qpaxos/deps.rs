pub use std::cmp::Ordering;
use std::ops::Index;
use std::ops::{Deref, DerefMut};

use crate::qpaxos::Dep;
use crate::qpaxos::Deps;
use crate::qpaxos::InstanceId;
use crate::qpaxos::ReplicaId;

/// Let user use method of Vec<Dep> directly.
impl Deref for Deps {
    type Target = Vec<Dep>;
    fn deref(&self) -> &Self::Target {
        &self.dep_vec
    }
}

/// Let user use method of Vec<Dep> directly.
impl DerefMut for Deps {
    fn deref_mut(&mut self) -> &mut Vec<Dep> {
        &mut self.dep_vec
    }
}

/// Let user use instance_id_vec[replic_id] to retreive a Dep.
/// It panics if replica_id not found.
/// It returns the first match.
impl Index<ReplicaId> for Deps {
    type Output = Dep;
    fn index(&self, rid: ReplicaId) -> &Self::Output {
        for inst in self.dep_vec.iter() {
            if inst.replica_id == rid {
                return inst;
            }
        }
        panic!("NotFound dep with replica_id={}", rid);
    }
}

impl PartialEq<Dep> for Deps {
    fn eq(&self, other: &Dep) -> bool {
        self.get(other.replica_id) == Some(*other)
    }
}

impl PartialOrd<Dep> for Deps {
    fn partial_cmp(&self, other: &Dep) -> Option<Ordering> {
        let mine = self.get(other.replica_id);
        match mine {
            Some(iid) => Some(iid.cmp(other)),
            None => Some(Ordering::Less),
        }
    }
}

impl PartialEq<InstanceId> for Deps {
    fn eq(&self, other: &InstanceId) -> bool {
        let mine = self.get(other.replica_id);
        match mine {
            Some(iid) => iid.replica_id == other.replica_id && iid.idx == other.idx,
            None => false,
        }
    }
}

impl PartialOrd<InstanceId> for Deps {
    fn partial_cmp(&self, other: &InstanceId) -> Option<Ordering> {
        let mine = self.get(other.replica_id);
        match mine {
            Some(iid) => Some(iid.idx.cmp(&other.idx)),
            None => Some(Ordering::Less),
        }
    }
}

impl Deps {
    /// get retreive a Dep with specified replica_id.
    /// It returns the first match.
    pub fn get(&self, rid: ReplicaId) -> Option<Dep> {
        for iid in self.dep_vec.iter() {
            if iid.replica_id == rid {
                return Some(*iid);
            }
        }
        None
    }

    /// set add an instanceId into it or overrides an existing one with the same replicaId.
    /// It returns the index the added Dep is, along with an Option of the replaced value.
    pub fn set(&mut self, dep: Dep) -> (usize, Option<Dep>) {
        let mut idx: i64 = -1;
        for (i, iid) in self.dep_vec.iter().enumerate() {
            if iid.replica_id == dep.replica_id {
                idx = i as i64;
                break;
            }
        }

        if idx == -1 {
            let l = self.dep_vec.len();
            self.dep_vec.push(dep);
            (l, None)
        } else {
            let old = self.dep_vec[idx as usize];
            self.dep_vec[idx as usize] = dep;
            (idx as usize, Some(old))
        }
    }
}

impl From<&[Dep]> for Deps {
    fn from(v: &[Dep]) -> Deps {
        Deps { dep_vec: v.into() }
    }
}

impl From<&[InstanceId]> for Deps {
    fn from(v: &[InstanceId]) -> Deps {
        let mut depvec: Vec<Dep> = Vec::with_capacity(v.len());
        for iid in v.iter() {
            depvec.push(iid.into());
        }
        depvec.into()
    }
}

impl From<Vec<InstanceId>> for Deps {
    fn from(v: Vec<InstanceId>) -> Deps {
        let mut depvec: Vec<Dep> = Vec::with_capacity(v.len());
        for iid in v.iter() {
            depvec.push(iid.into());
        }
        depvec.into()
    }
}

impl<A: Into<ReplicaId> + Copy, B: Into<i64> + Copy> From<&[(A, B)]> for Deps {
    fn from(v: &[(A, B)]) -> Deps {
        v.iter().map(|x| x.into()).collect::<Vec<Dep>>().into()
    }
}

impl<A> From<&[A; 0]> for Deps {
    fn from(_v: &[A; 0]) -> Deps {
        Deps { dep_vec: vec![] }
    }
}

impl<A> From<[A; 0]> for Deps {
    fn from(_v: [A; 0]) -> Deps {
        Deps { dep_vec: vec![] }
    }
}

macro_rules! impl_deps_from_arr {
    ($n:expr) => {
        impl<A: Into<ReplicaId> + Copy, B: Into<i64> + Copy> From<&[(A, B); $n]> for Deps {
            fn from(v: &[(A, B); $n]) -> Deps {
                let q: &[_] = v;
                q.into()
            }
        }

        impl<A: Into<ReplicaId> + Copy, B: Into<i64> + Copy> From<[(A, B); $n]> for Deps {
            fn from(v: [(A, B); $n]) -> Deps {
                let q: &[_] = &v;
                q.into()
            }
        }
    };
}

impl_deps_from_arr!(1);
impl_deps_from_arr!(2);
impl_deps_from_arr!(3);
impl_deps_from_arr!(4);
impl_deps_from_arr!(5);
impl_deps_from_arr!(6);
impl_deps_from_arr!(7);
impl_deps_from_arr!(8);
