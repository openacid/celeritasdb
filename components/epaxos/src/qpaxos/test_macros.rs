#[cfg(test)]
use pretty_assertions::assert_eq;

use crate::qpaxos::*;

// TODO move other macro test in test_instance here.

#[test]
fn test_macro_dep() {
    {
        // implicit type
        let _dep = dep!(1, 2);
    }

    let dep = dep!(1, 2);
    assert_eq!(
        Dep {
            replica_id: 1,
            idx: 2,
            seq: 0,
        },
        dep
    );

    let dep = dep!(1, 2, 3);
    assert_eq!(
        Dep {
            replica_id: 1,
            idx: 2,
            seq: 3,
        },
        dep
    );
}

#[test]
fn test_macro_deps() {
    {
        // implicit type
        let depvec = depvec![];
        assert_eq!(Vec::<Dep>::new(), depvec);
    }

    let depvec = depvec![(1, 2)];
    assert_eq!(vec![Dep::from((1, 2))], depvec);
}

#[test]
fn test_macro_init_inst() {
    let want = Instance {
        instance_id: Some((1, 2).into()),
        ballot: Some((0, 0, 1).into()),
        cmds: vec![("Set", "x", "y").into(), ("Get", "a", "b").into()],
        deps: Some(DepVec {
            ids: vec![(11, 12).into(), (13, 14).into()],
        }),
        accepted: false,
        committed: false,
        executed: false,
    };

    assert_eq!(
        want,
        init_inst!(
            (1, 2),
            [("Set", "x", "y"), ("Get", "a", "b")],
            [(11, 12), (13, 14)]
        )
    );
}
