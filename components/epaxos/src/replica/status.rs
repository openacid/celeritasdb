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
#[derive(Debug, Default)]
pub struct Status {
    // TODO: to work with cluster membership updating, a single number quorum is not enough in future.
    pub fast_quorum: i32,
    pub quorum: i32,

    // With a cached instance it is possible to reduce storage access during replication.
    pub instance: Instance,

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

    /// accept_oks tracks positive accept-replies.
    /// AcceptReply with error, delayed, or with lower ballot does not count.
    pub accept_oks: HashMap<ReplicaID, bool>,
}

impl Status {
    /// new creates a Status with initial deps filled, as if it already fast-accepted from the
    /// instnace it serves.
    pub fn new(n_replica: i32, instance: Instance) -> Self {
        let mut st = Self {
            quorum: quorum(n_replica),
            fast_quorum: fast_quorum(n_replica),
            instance,

            fast_replied: HashMap::new(),
            fast_deps: HashMap::new(),
            fast_committed: HashMap::new(),

            accept_replied: HashMap::new(),
            accept_oks: HashMap::new(),
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
        let iid = self.instance.instance_id.unwrap();
        let rid = iid.replica_id;
        self.accept_replied.insert(rid, true);
        self.accept_oks.insert(rid, true);

        self
    }

    /// get_fast_commit_deps returns a InstanceId Vec if current status satisfies fast-commit
    /// condition. Otherwise it returns None.
    pub fn get_fast_commit_deps(&mut self, cluster: &[ReplicaID]) -> Option<Vec<InstanceId>> {
        let mut rst: Vec<InstanceId> = Vec::with_capacity(cluster.len());
        for rid in cluster.iter() {
            let deps = self.fast_deps.get_mut(rid)?;

            // TODO do not need to sort every time calling this function.
            deps.sort();

            let fdep = get_fast_commit_dep(deps, &self.fast_committed, self.fast_quorum)?;
            rst.push(fdep);
        }
        Some(rst)
    }

    /// get_accept_deps returns a InstanceId Vec for accept request.
    /// If current status accumulated enough fast-accept-replies. Otherwise it returns None.
    pub fn get_accept_deps(&mut self, cluster: &[ReplicaID]) -> Option<Vec<InstanceId>> {
        let mut rst: Vec<InstanceId> = Vec::with_capacity(cluster.len());
        for rid in cluster.iter() {
            let deps = self.fast_deps.get_mut(rid)?;

            // TODO do not need to sort every time calling this function.
            deps.sort();

            let fdep = get_accept_dep(deps, self.quorum)?;
            rst.push(fdep);
        }
        Some(rst)
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

    if fast_quorum == 0 {
        // only when n==1 fast_quorum could be 0
        assert!(n == 1);
        return Some(deps[0]);
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
///
/// E.g. If n=5 and quorum is 3 then if the leader `L` accumulated: [a, a, a, b].
/// Then we could just choose `a` instead of `b`.
/// Although in original paper it specifies the deps for accept is the union of all replied deps.
/// If the leader saw 3 `a`, it means leader of `b` did not commit when `L` initiated.
/// Thus `L` does not have to be after `b`.
///
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
