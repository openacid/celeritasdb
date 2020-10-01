pub use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::qpaxos::InstanceId;
use crate::qpaxos::InstanceIds;
use crate::qpaxos::ReplicaId;

/// Let user use method of Vec<InstanceId> directly.
impl Deref for InstanceIds {
    type Target = HashMap<ReplicaId, i64>;
    fn deref(&self) -> &Self::Target {
        &self.ids
    }
}

/// Let user use method of Vec<InstanceId> directly.
impl DerefMut for InstanceIds {
    fn deref_mut(&mut self) -> &mut HashMap<ReplicaId, i64> {
        &mut self.ids
    }
}

impl PartialEq<InstanceId> for InstanceIds {
    fn eq(&self, other: &InstanceId) -> bool {
        self.get(&other.replica_id) == Some(&other.idx)
    }
}

impl PartialOrd<InstanceId> for InstanceIds {
    fn partial_cmp(&self, other: &InstanceId) -> Option<Ordering> {
        let mine_idx = self.get(&other.replica_id);
        match mine_idx {
            Some(idx) => Some(idx.cmp(&other.idx)),
            None => Some(Ordering::Less),
        }
    }
}

impl From<&[InstanceId]> for InstanceIds {
    fn from(v: &[InstanceId]) -> InstanceIds {
        let mut ids = InstanceIds {
            ids: HashMap::new(),
        };
        for iid in v.iter() {
            ids.insert(iid.replica_id, iid.idx);
        }
        ids
    }
}

impl From<Vec<InstanceId>> for InstanceIds {
    fn from(v: Vec<InstanceId>) -> InstanceIds {
        let mut ids = InstanceIds {
            ids: HashMap::new(),
        };
        for iid in v.iter() {
            ids.insert(iid.replica_id, iid.idx);
        }
        ids
    }
}

impl<A: Into<ReplicaId> + Copy, B: Into<i64> + Copy> From<&[(A, B)]> for InstanceIds {
    fn from(v: &[(A, B)]) -> InstanceIds {
        let mut ids = InstanceIds {
            ids: HashMap::new(),
        };
        for iid in v.iter() {
            ids.insert(iid.0.into(), iid.1.into());
        }
        ids
    }
}

impl<A> From<&[A; 0]> for InstanceIds {
    fn from(_v: &[A; 0]) -> InstanceIds {
        InstanceIds {
            ..Default::default()
        }
    }
}

impl<A> From<[A; 0]> for InstanceIds {
    fn from(_v: [A; 0]) -> InstanceIds {
        InstanceIds {
            ..Default::default()
        }
    }
}

macro_rules! impl_instance_ids {
    ($n:expr) => {
        impl<A: Into<ReplicaId> + Copy, B: Into<i64> + Copy> From<&[(A, B); $n]> for InstanceIds {
            fn from(v: &[(A, B); $n]) -> InstanceIds {
                let q: &[_] = v;
                q.into()
            }
        }

        impl<A: Into<ReplicaId> + Copy, B: Into<i64> + Copy> From<[(A, B); $n]> for InstanceIds {
            fn from(v: [(A, B); $n]) -> InstanceIds {
                let q: &[_] = &v;
                q.into()
            }
        }
    };
}

impl_instance_ids!(1);
impl_instance_ids!(2);
impl_instance_ids!(3);
impl_instance_ids!(4);
impl_instance_ids!(5);
impl_instance_ids!(6);
impl_instance_ids!(7);
impl_instance_ids!(8);
