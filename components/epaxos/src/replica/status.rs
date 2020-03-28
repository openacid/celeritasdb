use crate::qpaxos::*;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InstanceStatus {
    Na,
    FastAccepted,
    Accepted,
    Committed,
    Executed,
}

impl Instance {
    pub fn status(&self) -> InstanceStatus {
        if self.executed {
            return InstanceStatus::Executed;
        }

        if self.committed {
            return InstanceStatus::Committed;
        }

        if self.final_deps.is_some() {
            return InstanceStatus::Accepted;
        }

        if self.deps.is_some() {
            return InstanceStatus::FastAccepted;
        }

        InstanceStatus::Na
    }
}

pub struct Status {
    pub quorum: i32,
    pub ok_count: i32,
}

impl Status {
    pub fn new(quorum: i32) -> Self {
        Self {
            quorum,
            ok_count: 1,
        }
    }

    pub fn finish(&mut self) -> bool {
        self.ok_count += 1;
        self.ok_count >= self.quorum
    }
}

pub type FastAcceptStatus = Status;
pub type AcceptStatus = Status;
pub type PrepareStatus = Status;

/// `get_fast_commit_dep` finds out the safe dependency by a leader for fast commit.
///
/// `deps` are instance-ids with the same `replica_id`. It contains the initial dep at the 0-th
/// slot, and updated deps from 1-th slot.
/// `deps` must be sorted.
/// `committed` stores which instance has been committed.
///
/// The conditions of fast-commit are:
/// - the number of identical deps is at least fast-quorum,
/// - and this dep is committed.
///
/// If there is no safe dep for fast-commit, it returns None.
pub fn get_fast_commit_dep(
    deps: &Vec<InstanceId>,
    committed: &HashMap<InstanceId, bool>,
    fast_quorum: i32,
) -> Option<InstanceId> {
    // TODO what if deps.len() is 0
    // the first elt in deps is the initial dep.
    //
    // TODO need to ensure the committed dep not depend on this instance, the second FP-condition.

    let n = deps.len() as i32;
    assert!(n > 0);
    assert!(deps.is_sorted());

    if n < fast_quorum {
        return None;
    }

    let x = (n - fast_quorum) as usize;

    for i in 0..=x {
        let dep = &deps[i];
        if dep == &deps[i + fast_quorum as usize - 1] {
            // TODO: add proof of it: equals to initial value does not need to be committed.
            if i == 0 || committed.get(dep) == Some(&true) {
                return Some(*dep);
            }
        }
    }

    return None;
}
