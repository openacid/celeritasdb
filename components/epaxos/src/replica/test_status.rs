use crate::qpaxos::*;
use crate::replica::get_accept_dep;
use crate::replica::get_fast_commit_dep;
use crate::replica::Status;
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
    let inst = inst!(
        (1, 2),
        (3, 4, _),
        [("Set", "x", "1")],
        [(1, 1), (2, 0)],
        "withdeps"
    );
    let iid = inst.instance_id.unwrap();
    let replica_id = iid.replica_id;

    let st = Status::new(7, inst.clone());

    assert_eq!(4, st.quorum);
    assert_eq!(5, st.fast_quorum);
    assert_eq!(st.instance, inst);

    get!(st.fast_replied, &replica_id, true);

    get!(st.fast_deps, &1, vec![instid!(1, 1)]);
    get!(st.fast_deps, &2, vec![instid!(2, 0)]);
    get!(st.fast_deps, &3, None);

    get!(st.accept_replied, &1, None);
    get!(st.accept_replied, &2, None);
    assert_eq!(st.accept_ok, 0);
}

#[test]
#[allow(unused_mut)]
fn test_get_fast_commit_dep() {
    macro_rules! case(
        (
            $qf:expr,
            [$($didx:expr),*],
            {$($cidx:expr),*},
            $want:expr
            ) => {
            (
                $qf,
                instids![$((1, $didx)),*],
                {
                    let mut m = ::std::collections::HashMap::new();
                    $(
                        m.insert(instid!(1, $cidx), true);
                     )*
                    m
                },
                $want
            )
         };
    );

    let mut cases: Vec<(
        i32,
        Vec<InstanceId>,
        HashMap<InstanceId, bool>,
        Option<InstanceId>,
    )> = vec![
        case!(0, [2], {}, Some(instid!(1, 2))),
        case!(1, [2], {}, Some(instid!(1, 2))),
        case!(2, [2], {}, None),
        case!(2, [2], { 2 }, None),
        case!(2, [2, 3], {}, None),
        case!(2, [2, 3], { 2 }, None),
        case!(2, [2, 3], {2, 3}, None),
        // TODO: proof it: If all equals to initial value, it does not need to be committed
        case!(2, [2, 2], {}, Some(instid!(1, 2))),
        case!(2, [2, 2], { 2 }, Some(instid!(1, 2))),
        case!(2, [2, 2, 3], { 2 }, Some(instid!(1, 2))),
        case!(2, [2, 2, 3, 3], {}, Some(instid!(1, 2))),
        case!(2, [2, 2, 3, 3], { 3 }, Some(instid!(1, 2))),
        case!(3, [2, 2, 3, 3], { 3 }, None),
        // same to initial value
        case!(3, [2, 2, 2, 3], { 3 }, Some(instid!(1, 2))),
        case!(3, [2, 2, 2, 3], { 2 }, Some(instid!(1, 2))),
        case!(3, [2, 3, 3, 3], { 2 }, None),
        case!(3, [2, 3, 3, 3], { 3 }, Some(instid!(1, 3))),
    ];

    for (fast_quorum, deps, committed, want) in cases.iter_mut() {
        let fdep = get_fast_commit_dep(deps, committed, *fast_quorum);
        assert_eq!(
            *want, fdep,
            "deps:{:?}, committed:{:?}, qf:{}",
            deps, committed, fast_quorum
        );
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
                instids![$((1, $didx)),*],
                $want
            )
         };
    );

    let mut cases: Vec<(i32, Vec<InstanceId>, Option<InstanceId>)> = vec![
        case!(2, [2], None),
        case!(2, [2, 2], Some(instid!(1, 2))),
        case!(2, [2, 3], Some(instid!(1, 3))),
        case!(2, [2, 2, 3], Some(instid!(1, 2))),
        case!(2, [2, 2, 3, 3], Some(instid!(1, 2))),
        case!(3, [2, 2], None),
        case!(3, [2, 2, 2, 3], Some(instid!(1, 2))),
        case!(3, [2, 2, 3, 3], Some(instid!(1, 3))),
        case!(3, [2, 2, 3, 4], Some(instid!(1, 3))),
    ];

    for (quorum, deps, want) in cases.iter_mut() {
        let adep = get_accept_dep(deps, *quorum);
        assert_eq!(*want, adep, "deps:{:?}, f:{}", deps, quorum);
    }
}

#[test]
fn test_status_get_fast_commit_deps() {
    macro_rules! case(
        (
            $n:expr,
            // fast_deps
            {
                $(
                [$($didx:expr),*]
                ),*
            },
            // fast_committed
            {
                $(
                ($rid:expr, $iid:expr)
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
                        m.insert(rid, instids![$((rid, $didx)),*]);
                     )*
                    m
                },
                {
                    let mut m = ::std::collections::HashMap::new();
                    $(
                        m.insert(instid!($rid, $iid), true);
                     )*
                    m
                },
                $want
            )
         };
    );

    let mut cases: Vec<(
        i32,
        HashMap<ReplicaID, Vec<InstanceId>>,
        HashMap<InstanceId, bool>,
        Option<Vec<InstanceId>>,
    )> = vec![
        case!(1, { [2] }, {}, Some(instids![(1, 2)])),
        case!(1, { [2] }, { (1, 2) }, Some(instids![(1, 2)])),
        case!(2, {
            [2, 3],
            [2]
        },
        {(1, 3)},
        None),
        case!(2, {
            [2, 2],
            [2, 2]
        },
        {},
        Some(instids![(1, 2), (2, 2)])),
        case!(2, {
            [3, 3],
            [4, 4]
        },
        {(1, 3), (2, 4)},
        Some(instids![(1, 3), (2, 4)])),
        case!(3, {
            [2],
            [3],
            [4]
        },
        {},
        None),
        case!(3, {
            [2, 2],
            [3, 3],
            [4, 4]
        },
        {},
        Some(instids![(1, 2), (2, 3), (3, 4)])),
        case!(3, {
            [2, 2, 4],
            [3, 4, 4],
            [3, 5, 5]
        },
        {(2, 4), (3, 5)},
        Some(instids![(1, 2), (2, 4), (3, 5)])),
    ];

    for (n, deps, committed, want) in cases.iter_mut() {
        let fq = fast_quorum(*n);
        let mut st = Status {
            fast_quorum: fq,
            fast_deps: deps.clone(),
            fast_committed: committed.clone(),
            ..Default::default()
        };
        let cluster: Vec<i64> = (1..(*n + 1) as i64).map(|x| x).collect();
        let adep = st.get_fast_commit_deps(&cluster);
        assert_eq!(
            *want, adep,
            "deps:{:?}, committed:{:?}, n:{}, fq:{}",
            deps, committed, n, fq
        );
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
                        m.insert(rid, instids![$((rid, $didx)),*]);
                     )*
                    m
                },
                $want
            )
         };
    );

    let mut cases: Vec<(
        i32,
        HashMap<ReplicaID, Vec<InstanceId>>,
        Option<Vec<InstanceId>>,
    )> = vec![
        case!(1, { [2] }, Some(instids![(1, 2)])),
        case!(2, {[2], [2]}, None),
        case!(2, {[2, 3], [2, 4]}, Some(instids![(1, 3), (2, 4)])),
        case!(3, {[2], [3], [4]}, None),
        case!(3, {[2, 2], [3, 4], [3, 5]}, Some(instids![(1, 2), (2, 4), (3, 5)])),
        case!(3, {[2, 2, 4], [3, 4, 4], [3, 5, 5]}, Some(instids![(1, 2), (2, 4), (3, 5)])),
    ];

    for (n, deps, want) in cases.iter_mut() {
        let q = quorum(*n);
        let mut st = Status {
            quorum: q,
            fast_deps: deps.clone(),
            ..Default::default()
        };
        let cluster: Vec<i64> = (1..(*n + 1) as i64).map(|x| x).collect();
        let adep = st.get_accept_deps(&cluster);
        assert_eq!(*want, adep, "deps:{:?}, n:{}, q:{}", deps, n, q);
    }
}
