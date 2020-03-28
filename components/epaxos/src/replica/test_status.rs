use crate::qpaxos::*;
use crate::replica::get_fast_commit_dep;
use std::collections::HashMap;

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn test_get_safe_dep() {
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
