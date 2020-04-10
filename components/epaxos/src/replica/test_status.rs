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
