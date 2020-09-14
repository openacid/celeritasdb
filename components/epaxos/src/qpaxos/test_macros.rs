#[cfg(test)]
use pretty_assertions::assert_eq;

use crate::qpaxos::*;

// TODO move other macro test in test_instance here.

#[test]
fn test_macro_cmd() {
    assert_eq!(
        Command::from(("Get", "x", "y")),
        cmd!("Get", "x", "y"));

    assert_eq!(
        Command::from(("Get", "x", "")),
        cmd!(x));

    assert_eq!(
        Command::from(("Set", "x", "y")),
        cmd!(x=y));

    assert_eq!(
        Command::from(("Set", "x", "yyy")),
        cmd!(x="yyy"));

    assert_eq!(
        Command::from(("Set", "x", "y")),
        cmd!(x="y"));
}

#[test]
fn test_macro_cmdvec() {
    assert_eq!(
        Vec::<Command>::new(),
        cmdvec![]
    );

    assert_eq!(
        vec![Command::from(("Get", "x", "")),
        Command::from(("Set", "x", "y")),
        Command::from(("Set", "a", "b")),
        ],
        cmdvec![(x), (x=y), ("Set", "a", "b")]);
}

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
fn test_macro_depvec() {
    {
        // implicit type
        let depvec = depvec![];
        assert_eq!(Vec::<Dep>::new(), depvec);
    }

    let depvec = depvec![(1, 2)];
    assert_eq!(vec![Dep::from((1, 2))], depvec);

    let depvec = depvec!(0, [1, 2]);
    assert_eq!(vec![Dep::from((0, 1)), Dep::from((1, 2))], depvec);

    let depvec = depvec!(3, [1, 2]);
    assert_eq!(vec![Dep::from((3, 1)), Dep::from((4, 2))], depvec);
}

#[test]
fn test_macro_instidvec() {
    {
        // implicit type
        let instidvec = instidvec![];
        assert_eq!(Vec::<Dep>::new(), instidvec);
    }

    let instidvec = instidvec![(1, 2)];
    assert_eq!(vec![InstanceId::from((1, 2))], instidvec);
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

#[test]
fn test_macro_inst() {
    // instance_id, ballot, cmds
    let want = Instance {
        instance_id: Some((1, 2).into()),
        ballot: Some((3, 4, 1).into()),
        cmds: vec![("Set", "x", "y").into(), ("Get", "a", "b").into()],
        deps: None,
        accepted: false,
        committed: false,
        executed: false,
    };

    assert_eq!(
        want,
        inst!((1, 2), (3, 4, _), [("Set", "x", "y"), ("Get", "a", "b")])
    );

    // instance_id, cmds
    let want = Instance {
        instance_id: Some((1, 2).into()),
        ballot: None,
        cmds: vec![("Set", "x", "y").into(), ("Get", "a", "").into()],
        deps: None,
        accepted: false,
        committed: false,
        executed: false,
    };

    assert_eq!(
        want,
        inst!((1, 2), [("Set", "x", "y"), ("Get", "a", "")])
    );
    assert_eq!(
        want,
        inst!((1, 2), [(x=y), (a)])
    );

    // instance_id, cmds, deps
    let want = Instance {
        instance_id: Some((1, 2).into()),
        ballot: None,
        cmds: vec![("Set", "x", "y").into(), ("Get", "a", "").into()],
        deps:Some(vec![dep!(11, 12), dep!(12, 13)].into()),
        accepted: false,
        committed: false,
        executed: false,
    };

    assert_eq!(
        want,
        inst!((1, 2), [(x=y), (a)], [(11, 12), (12, 13)])
    );
    assert_eq!(
        want,
        inst!((1, 2), [(x=y), (a)], (11, [12, 13]))
    );

    // instance_id, ballot, cmds, deps
    let want = Instance {
        instance_id: Some((1, 2).into()),
        ballot: Some((3, 4, 1).into()),
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
        inst!(
            (1, 2),
            (3, 4, _),
            [("Set", "x", "y"), ("Get", "a", "b")],
            [(11, 12), (13, 14)]
        )
    );

    // all arg
    let want = Instance {
        instance_id: Some((1, 2).into()),
        ballot: Some((3, 4, 3).into()),
        cmds: vec![("Set", "x", "y").into(), ("Get", "a", "b").into()],
        deps: Some(DepVec {
            ids: vec![(11, 12).into(), (13, 14).into()],
        }),
        accepted: true,
        committed: true,
        executed: true,
    };

    assert_eq!(
        want,
        inst!(
            (1, 2),
            (3, 4, 3),
            [("Set", "x", "y"), ("Get", "a", "b")],
            [(11, 12), (13, 14)],
            true,
            true,
            true
        )
    );
}

#[test]
fn test_macro_inst_kv() {
    assert_eq!(
        Instance {
            instance_id: Some((1, 2).into()),
            ..Default::default()
        },
        inst!(instance_id:(1, 2))
    );

    assert_eq!(
        Instance {
            instance_id: None,
            ..Default::default()
        },
        inst!(instance_id: None)
    );

    // with id
    assert_eq!(
        Instance {
            instance_id: Some((1, 2).into()),
            cmds: vec![("Set", "x", "1").into()],
            ..Default::default()
        },
        inst!((1, 2), cmds:[("Set", "x", "1")])
    );

    assert_eq!(
        Instance {
            ballot: Some((1, 2, 3).into()),
            ..Default::default()
        },
        inst!(ballot:(1, 2, 3))
    );

    assert_eq!(
        Instance {
            ballot: None,
            ..Default::default()
        },
        inst!(ballot: None)
    );

    assert_eq!(
        Instance {
            cmds: vec![("Set", "x", "1").into()],
            ..Default::default()
        },
        inst!(cmds:[("Set", "x", "1")])
    );

    assert_eq!(
        Instance {
            cmds: Vec::<Command>::new(),
            ..Default::default()
        },
        inst!(cmds:[])
    );

    assert_eq!(
        Instance {
            deps: Some(vec![Dep::from((1, 2))].into()),
            ..Default::default()
        },
        inst!(deps:[(1, 2)])
    );
}
