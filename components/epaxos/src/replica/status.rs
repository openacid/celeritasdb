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

/// Status tracks replication status during fast-accept, accept and commit phase.
#[derive(Debug)]
pub struct Status<'a> {
    // TODO: to work with cluster membership updating, a single number quorum is not enough in future.
    pub fast_quorum: i32,
    pub quorum: i32,

    // With a cached instance it is possible to reduce storage access during replication.
    pub instance: &'a Instance,

    /// fast_replied tracks what replica has sent back FastAcceptReply.
    /// It is used to de-dup duplicated messages.
    pub fast_replied: HashMap<ReplicaID, bool>,

    /// fast_deps collects `deps` received in fast-accept phase.
    /// They are stored by dependency instance leader.
    pub fast_deps: HashMap<ReplicaID, Vec<InstanceId>>,

    /// fast_committed tracks what updated dep instance is committed.
    pub fast_committed: HashMap<InstanceId, bool>,

    /// accept_replie tracks what replica has sent back AcceptReply.
    /// It does include the leader itself, although the leader update instance status
    /// to "accept" locally.
    pub accept_replied: HashMap<ReplicaID, bool>,

    pub accept_ok: i32,
}

impl<'a> Status<'a> {
    /// new creates a Status with initial deps filled, as if it already fast-accepted from the
    /// instnace it serves.
    pub fn new(n_replica: i32, instance: &'a Instance) -> Self {
        let mut st = Self {
            quorum: quorum(n_replica),
            fast_quorum: fast_quorum(n_replica),
            instance,

            fast_replied: HashMap::new(),
            fast_deps: HashMap::new(),
            fast_committed: HashMap::new(),

            accept_replied: HashMap::new(),
            accept_ok: 0,
        };

        st.start_fast_accept();

        st
    }

    /// start_fast_accept performs a handle-fast-accept-reply for the instance it serves.
    pub fn start_fast_accept(&mut self) -> &mut Self {
        let iid = self.instance.instance_id.unwrap();
        let rid = iid.replica_id;

        self.fast_replied.insert(rid, true);

        let deps = self.instance.deps.as_ref().unwrap();
        for d in deps.iter() {
            let rid = d.replica_id;
            if !self.fast_deps.contains_key(&rid) {
                self.fast_deps.insert(rid, Vec::new());
            }

            self.fast_deps.get_mut(&rid).unwrap().push(*d);
        }

        self
    }

    /// start_accept initiates Status to enter Accept phase.
    pub fn start_accept(&mut self) -> &mut Self {
        // local instance accepts it.
        self.accept_ok = 1;
        self
    }

    pub fn finish(&mut self) -> bool {
        self.accept_ok += 1;
        self.accept_ok >= self.quorum
    }
}

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

/// get_accept_dep returns the dep for accept-request if a quorum of replies received.
/// Otherwise it returns None.
/// It always choose an as low instance as possible to reduce conflict.
/// It contains the initial dep at the 0-th slot, and updated deps from 1-th slot.
/// `deps` in Accept Request is the union of `deps` replied in fast-accept phase.
///
/// `deps` must be sorted.
pub fn get_accept_dep(deps: &Vec<InstanceId>, quorum: i32) -> Option<InstanceId> {
    // TODO what if deps.len() is 0
    // the first elt in deps is the initial dep.

    let n = deps.len() as i32;
    assert!(n > 0);
    assert!(deps.is_sorted());

    if n < quorum {
        return None;
    }

    return Some(deps[(quorum - 1) as usize]);
}
