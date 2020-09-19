use crate::qpaxos::*;
use crate::replica::get_accept_dep;
use crate::replica::get_fast_commit_dep;
use crate::replica::DepStatus;
use crate::replica::RepliedDep;
use crate::replica::ReplicationStatus;
use std::collections::HashMap;

#[cfg(test)]
use pretty_assertions::assert_eq;

macro_rules! get {
    ($container:expr, $key:expr, None) => {
        assert_eq!($container.get($key), None);
    };

    ($container:expr, $key:expr, $want:expr) => {
        assert_eq!($container.get($key), Some(&$want));
    };
}

#[test]
fn test_status_new() {
    let inst = inst!((1, 2), (3, 4, _), [("Set", "x", "1")], [(1, 1), (2, 0)],);

    let st = ReplicationStatus::new(7, inst.clone());

    assert_eq!(4, st.quorum);
    assert_eq!(5, st.fast_quorum);
    assert_eq!(st.instance, inst);

    assert!(st.prepared.contains_key(&1));
    assert!(st.prepared.contains_key(&2));
    assert!(st.prepared[&1].replied.contains(&1));
    assert!(!st.prepared[&1].replied.contains(&2));
    assert_eq!(
        vec![RepliedDep {
            idx: 1,
            seq: 0,
            committed: false
        },],
        st.prepared[&1].rdeps
    );
    assert_eq!(
        vec![RepliedDep {
            idx: 0,
            seq: 0,
            committed: false
        },],
        st.prepared[&2].rdeps
    );

    get!(st.accepted, &1, None);
    get!(st.accepted, &2, None);
}

#[test]
fn test_status_start_accept() {
    let inst = inst!((1, 2), (3, 4, _), [("Set", "x", "1")], [(1, 1), (2, 0)],);
    let mut st = ReplicationStatus::new(7, inst.clone());

    get!(st.accepted, &1, None);

    st.start_accept();

    assert!(st.accepted.contains(&1));
}

#[test]
#[allow(unused_mut)]
fn test_get_fast_commit_dep() {
    macro_rules! case(
        (
            $qf:expr,
            [$(($didx:expr, $committed:expr)),*],
            $want:expr
            ) => {
            (
                $qf,
                vec![$(RepliedDep{idx:$didx, seq:0, committed: $committed}),*],
                $want
            )
         };
    );

    let mut cases: Vec<(i32, Vec<RepliedDep>, Option<Dep>)> = vec![
        case!(1, [(2, false)], Some(dep!(1, 2))),
        case!(2, [(2, false)], None),
        case!(2, [(2, true)], None),
        case!(2, [(2, false), (3, false)], None),
        case!(2, [(2, true), (3, false)], None),
        case!(2, [(2, true), (3, true)], None),
        // TODO: proof it: If all equals to initial value, it does not need to be committed
        case!(2, [(2, false), (2, false)], Some(dep!(1, 2))),
        case!(2, [(2, false), (2, true)], Some(dep!(1, 2))),
        case!(2, [(2, false), (2, true), (3, false)], Some(dep!(1, 2))),
        case!(
            2,
            [(2, false), (2, false), (3, false), (3, false)],
            Some(dep!(1, 2))
        ),
        case!(
            2,
            [(2, false), (2, false), (3, false), (3, true)],
            Some(dep!(1, 2))
        ),
        case!(3, [(2, false), (2, false), (3, false), (3, true)], None),
        // same to initial value
        case!(
            3,
            [(2, false), (2, false), (2, false), (3, true)],
            Some(dep!(1, 2))
        ),
        case!(
            3,
            [(2, false), (2, false), (2, true), (3, false)],
            Some(dep!(1, 2))
        ),
        case!(3, [(2, true), (3, false), (3, false), (3, false)], None),
        case!(
            3,
            [(2, false), (3, false), (3, false), (3, true)],
            Some(dep!(1, 3))
        ),
    ];

    for (fast_quorum, deps, want) in cases.iter_mut() {
        let fdep = get_fast_commit_dep(1, deps, *fast_quorum);
        assert_eq!(*want, fdep, "deps:{:?}, qf:{}", deps, fast_quorum);
    }
}

#[test]
fn test_get_accept_dep() {
    macro_rules! case(
        (
            $qf:expr,
            [$($didx:expr),*],
            $want:expr
            ) => {
            (
                $qf,
                vec![$(RepliedDep{idx: $didx, seq: 0, committed: false}),*],
                $want
            )
         };
    );

    let mut cases: Vec<(i32, Vec<RepliedDep>, Option<Dep>)> = vec![
        case!(2, [2], None),
        case!(2, [2, 2], Some(dep!(1, 2))),
        case!(2, [2, 3], Some(dep!(1, 3))),
        case!(2, [2, 2, 3], Some(dep!(1, 2))),
        case!(2, [2, 2, 3, 3], Some(dep!(1, 2))),
        case!(3, [2, 2], None),
        case!(3, [2, 2, 2, 3], Some(dep!(1, 2))),
        case!(3, [2, 2, 3, 3], Some(dep!(1, 3))),
        case!(3, [2, 2, 3, 4], Some(dep!(1, 3))),
    ];

    for (quorum, deps, want) in cases.iter_mut() {
        let adep = get_accept_dep(1, deps, *quorum);
        assert_eq!(*want, adep, "deps:{:?}, f:{}", deps, quorum);
    }
}

#[test]
#[allow(unused_mut)]
fn test_status_get_fast_commit_deps() {
    macro_rules! case(
        (
            $n:expr,
            // fast_deps
            {
                $(
                [$(($didx:expr, $committed:expr)),*]
                ),*
            },
            $want:expr
            ) => {
            (
                $n,
                {
                    let mut m = ::std::collections::HashMap::new();
                    let mut rid = 0;
                    $(
                        rid += 1;
                        m.insert(rid, DepStatus{
                                rdeps: vec![$(RepliedDep{idx:$didx, seq:0,committed:$committed}),*],
                                ..Default::default()
                        }
                        );
                     )*
                    m
                },
                $want
            )
         };
    );

    let mut cases: Vec<(i32, HashMap<ReplicaId, DepStatus>, Option<Vec<Dep>>)> = vec![
        case!(1, { [(2, false)] }, Some(depvec![(1, 2)])),
        case!(1, { [(2, true)] }, Some(depvec![(1, 2)])),
        case!(2, {
            [(2,false),(3,true)],
            [(2,false)]
        },
        None),
        case!(2, {
            [(2,false),(2,false)],
            [(2,false),(2,false)]
        },
        Some(depvec![(1, 2), (2, 2)])),
        case!(2, {
            [(3,false),(3,true)],
            [(4,false),(4,true)]
        },
        Some(depvec![(1, 3), (2, 4)])),
        case!(3, {
            [(2,false)],
            [(3,false)],
            [(4,false)]
        },
        None),
        case!(3, {
            [(2,false),(2,false)],
            [(3,false),(3,false)],
            [(4,false),(4,false)]
        },
        Some(depvec![(1, 2), (2, 3), (3, 4)])),
        case!(3, {
            [(2,false),(2,false),(4,false)],
            [(3,false),(4,false),(4,true)],
            [(3,false),(5,false),(5,true)]
        },
        Some(depvec![(1, 2), (2, 4), (3, 5)])),
    ];

    for (n, deps, want) in cases.iter_mut() {
        let fq = fast_quorum(*n);
        let mut st = ReplicationStatus {
            fast_quorum: fq,
            prepared: deps.clone(),
            ..Default::default()
        };
        let cluster: Vec<i64> = (1..(*n + 1) as i64).map(|x| x).collect();
        let adep = st.get_fast_commit_deps(&cluster);
        assert_eq!(*want, adep, "deps:{:?}, n:{}, fq:{}", deps, n, fq);
    }
}

#[test]
fn test_status_get_accept_deps() {
    macro_rules! case(
        (
            $n:expr,
            {
                $(
                [$($didx:expr),*]
                ),*
            },
            $want:expr
            ) => {
            (
                $n,
                {
                    let mut m = ::std::collections::HashMap::new();
                    let mut rid = 0;
                    $(
                        rid += 1;
                        m.insert(rid, DepStatus{
                            rdeps: vec![$(RepliedDep{idx: $didx, seq: 0, committed: true}),*],
                            ..Default::default()
                        }
                    );
                     )*
                    m
                },
                $want
            )
         };
    );

    let mut cases: Vec<(i32, HashMap<ReplicaId, DepStatus>, Option<Vec<Dep>>)> = vec![
        case!(1, { [2] }, Some(depvec![(1, 2)])),
        case!(2, {[2], [2]}, None),
        case!(2, {[2, 3], [2, 4]}, Some(depvec![(1, 3), (2, 4)])),
        case!(3, {[2], [3], [4]}, None),
        case!(3, {[2, 2], [3, 4], [3, 5]}, Some(depvec![(1, 2), (2, 4), (3, 5)])),
        case!(3, {[2, 2, 4], [3, 4, 4], [3, 5, 5]}, Some(depvec![(1, 2), (2, 4), (3, 5)])),
    ];

    for (n, deps, want) in cases.iter_mut() {
        let q = quorum(*n);
        let mut st = ReplicationStatus {
            quorum: q,
            prepared: deps.clone(),
            ..Default::default()
        };
        let cluster: Vec<i64> = (1..(*n + 1) as i64).map(|x| x).collect();
        let adep = st.get_accept_deps(&cluster);
        assert_eq!(*want, adep, "deps:{:?}, n:{}, q:{}", deps, n, q);
    }
}
