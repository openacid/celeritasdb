use crate::qpaxos::*;
use std::collections::HashMap;
use std::collections::HashSet;

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

        if self.accepted_ballot != None {
            return InstanceStatus::Accepted;
        }

        if self.deps.is_some() {
            return InstanceStatus::FastAccepted;
        }

        InstanceStatus::Na
    }
}

/// RepliedDep stores a dependency replied from replica-j that is proposed by replica-i
#[derive(Debug, Default, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub struct RepliedDep {
    pub idx: i64,
    pub seq: i64,
    pub committed: bool,
}

#[derive(Debug, Default, Clone)]
pub struct DepStatus {
    pub replied: HashSet<ReplicaId>,
    pub rdeps: Vec<RepliedDep>,
}

/// Status tracks replication status during fast-accept, accept and commit phase.
#[derive(Debug, Default)]
pub struct Status {
    // TODO: to work with cluster membership updating, a single number quorum is not enough in future.
    pub fast_quorum: i32,
    pub quorum: i32,

    // With a cached instance it is possible to reduce storage access during replication.
    pub instance: Instance,

    /// prepared tracks prepare replies.
    ///
    /// prepared[i] stores the status of an instance proposed by replica-i.
    /// prepared[i].rdeps[j] stores replies from replica-j.
    pub prepared: HashMap<ReplicaId, DepStatus>,

    /// accept_oks tracks positive accept-replies.
    /// AcceptReply with error, delayed, or with lower ballot does not count.
    pub accept_oks: HashSet<ReplicaId>,
}

impl Status {
    /// new creates a Status with initial deps filled, as if it already fast-accepted from the
    /// instnace it serves.
    pub fn new(n_replica: i32, instance: Instance) -> Self {
        let mut st = Self {
            quorum: quorum(n_replica),
            fast_quorum: fast_quorum(n_replica),
            instance,

            prepared: HashMap::new(),

            accept_oks: HashSet::new(),
        };

        st.start_fast_accept();

        st
    }

    /// start_fast_accept performs a handle-fast-accept-reply for the instance it serves.
    pub fn start_fast_accept(&mut self) -> &mut Self {
        let iid = self.instance.instance_id.unwrap();

        let deps = self.instance.deps.as_ref().unwrap();
        for d in deps.iter() {
            let drid = d.replica_id;

            // TODO committed is not impl

            self.prepared.insert(
                drid,
                DepStatus {
                    replied: hashset! {iid.replica_id},
                    rdeps: vec![RepliedDep {
                        idx: d.idx,
                        seq: d.seq,
                        committed: false,
                    }],
                },
            );
        }

        self
    }

    /// start_accept initiates Status to enter Accept phase.
    pub fn start_accept(&mut self) -> &mut Self {
        // local instance accepts it.
        let iid = self.instance.instance_id.unwrap();
        let rid = iid.replica_id;
        self.accept_oks.insert(rid);

        self
    }

    /// get_fast_commit_deps returns a InstanceId Vec if current status satisfies fast-commit
    /// condition. Otherwise it returns None.
    pub fn get_fast_commit_deps(&mut self, cluster: &[ReplicaId]) -> Option<Vec<Dep>> {
        let mut rst: Vec<Dep> = Vec::with_capacity(cluster.len());
        for rid in cluster.iter() {
            // TODO do not need to sort every time calling this function.
            let deps = &mut self.prepared.get_mut(rid)?.rdeps;

            deps.sort();

            let fdep = get_fast_commit_dep(*rid, deps, self.fast_quorum)?;
            rst.push(fdep);
        }
        Some(rst)
    }

    /// get_accept_deps returns a InstanceId Vec for accept request.
    /// If current status accumulated enough fast-accept-replies. Otherwise it returns None.
    pub fn get_accept_deps(&mut self, cluster: &[ReplicaId]) -> Option<Vec<Dep>> {
        let mut rst: Vec<Dep> = Vec::with_capacity(cluster.len());
        for rid in cluster.iter() {
            // TODO do not need to sort every time calling this function.
            let deps = &mut self.prepared.get_mut(rid)?.rdeps;

            deps.sort();

            let fdep = get_accept_dep(*rid, deps, self.quorum)?;
            rst.push(fdep);
        }
        Some(rst)
    }
}

/// `get_fast_commit_dep` finds out the safe dependency by a leader for fast commit.
///
/// `rdeps`: replied dependent instances proposed by replica `rid`.
/// `rdeps` must be sorted.
///
/// The conditions of fast-commit are:
/// - the number of identical deps is at least fast-quorum,
/// - and this dep is committed.
///
/// If there is no safe dep for fast-commit, it returns None.
pub fn get_fast_commit_dep(
    rid: ReplicaId,
    rdeps: &Vec<RepliedDep>,
    fast_quorum: i32,
) -> Option<Dep> {
    let fast_quorum = fast_quorum as usize;
    let n = rdeps.len();

    assert!(n > 0);
    assert!(fast_quorum > 0);
    assert!(rdeps.is_sorted());

    if n < fast_quorum {
        return None;
    }

    // In a sorted vec, that i-th elt equals the (i+k)-th elt implies there are at least k+1 equal elts.
    for i in 0..=(n - fast_quorum) {
        let left = &rdeps[i];

        for j in (i + fast_quorum - 1)..n {
            let right = &rdeps[j];

            // TODO: add proof of it: equals to initial value does not need to be committed.

            if left.idx == right.idx && left.seq == right.seq && (i == 0 || right.committed) {
                return Some(Dep {
                    replica_id: rid,
                    idx: left.idx,
                    seq: left.seq,
                });
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
/// `rdeps` in Accept Request is the union of `rdeps` replied in fast-accept phase.
///
/// `rdeps` must be sorted.
pub fn get_accept_dep(rid: ReplicaId, rdeps: &Vec<RepliedDep>, quorum: i32) -> Option<Dep> {
    let quorum = quorum as usize;

    // the first elt in rdeps is the initial dep.

    let n = rdeps.len();
    assert!(n > 0);
    assert!(quorum > 0);
    assert!(rdeps.is_sorted());

    if n < quorum {
        return None;
    }

    let d = &rdeps[quorum - 1];
    return Some(Dep {
        replica_id: rid,
        idx: d.idx,
        seq: d.seq,
    });
}
