#[cfg(test)]
use pretty_assertions::assert_eq;

use crate::qpaxos::*;

// TODO move other macro test in test_instance here.

#[test]
fn test_macro_cmd() {
    // noop
    assert_eq!(Command::from(("NoOp", "", "")), cmd!());
    assert_eq!(Command::from(("NoOp", "", "")), cmd!(NoOp));

    assert_eq!(Command::from(("Get", "x", "y")), cmd!("Get", "x", "y"));

    assert_eq!(Command::from(("Get", "x", "")), cmd!(x));

    assert_eq!(Command::from(("Set", "x", "y")), cmd!(x = y));

    assert_eq!(Command::from(("Set", "x", "yyy")), cmd!(x = "yyy"));

    assert_eq!(Command::from(("Set", "x", "y")), cmd!(x = "y"));

    assert_eq!(Command::from(("Delete", "x", "")), cmd!(del x));
}

#[test]
fn test_macro_cmdvec() {
    assert_eq!(Vec::<Command>::new(), cmdvec![]);

    assert_eq!(
        vec![
            Command::from(("Get", "x", "")),
            Command::from(("Set", "x", "y")),
            Command::from(("Set", "a", "b")),
            Command::from(("Delete", "a", "")),
        ],
        cmdvec![(x), (x = y), ("Set", "a", "b"), (del a)]
    );
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
fn test_macro_instids() {
    {
        // implicit type
        let instids = instids! {};
        assert_eq!(
            InstanceIds {
                ..Default::default()
            },
            instids
        );
    }

    let instids = instids![(1, 2), (3, 4)];
    assert_eq!(
        InstanceIds {
            ids: hashmap! {
                1 => 2,
                3 => 4,
            }
        },
        instids
    );
}

#[test]
fn test_macro_optinstid() {
    let smiid: Option<InstanceId> = Some(InstanceId::from((1, 2)));

    assert_eq!(smiid, optinstid!((1, 2)));
    assert_eq!(smiid, optinstid!(InstanceId::from((1, 2))));

    let iid = InstanceId::from((1, 2));
    assert_eq!(smiid, optinstid!(iid));

    assert_eq!(None, optinstid!(None));
}

#[test]
fn test_macro_optdeps() {
    let smdeps: Option<Deps> = Some(Deps::from(depvec![(1, 2), (3, 4)]));

    assert_eq!(smdeps, optdeps!([(1, 2), (3, 4)]));
    assert_eq!(None, optdeps!(None));
}

#[test]
fn test_macro_instance_fields() {
    // instance_id

    let smiid: Option<InstanceId> = Some(InstanceId::from((1, 2)));
    assert_eq!(
        Option::<InstanceId>::None,
        __instance_fields!(instance_id, None)
    );
    assert_eq!(smiid, __instance_fields!(instance_id, (1, 2)));
    assert_eq!(
        smiid,
        __instance_fields!(instance_id, InstanceId::from((1, 2)))
    );

    // deps
    let smdeps: Option<Deps> = Some(Deps::from(depvec![(1, 2), (3, 4)]));
    assert_eq!(smdeps, __instance_fields!(deps, [(1, 2), (3, 4)]));
    assert_eq!(Option::<Deps>::None, __instance_fields!(deps, None));
}

#[test]
fn test_macro_inst_instid() {
    assert_eq!(
        Instance {
            instance_id: None,
            ..Default::default()
        },
        inst!(None)
    );

    let want = Instance {
        instance_id: Some(InstanceId::from((1, 2))),
        ..Default::default()
    };
    assert_eq!(want, inst!((1, 2)));

    let iid = InstanceId::from((1, 2));
    assert_eq!(want, inst!(iid));
}
#[test]
fn test_macro_inst_instid_cmds() {
    let want = Instance {
        instance_id: Some((1, 2).into()),
        cmds: vec![("Set", "x", "y").into(), ("Get", "a", "").into()],
        ..Default::default()
    };

    let iid = InstanceId::from((1, 2));

    assert_eq!(want, inst!((1, 2), [("Set", "x", "y"), ("Get", "a", "")]));
    assert_eq!(want, inst!((1, 2), [(x = y), (a)]));
    assert_eq!(want, inst!(iid, [(x = y), (a)]));

    // instance_id accept None
    assert_eq!(
        Instance {
            instance_id: None,
            cmds: vec![
                ("Set", "x", "y").into(),
                ("Get", "a", "").into(),
                ("Delete", "x", "").into(),
            ],
            ..Default::default()
        },
        inst!(None, [(x = y), (a), (del x)])
    );
}
#[test]
fn test_macro_inst_instid_cmds_deps() {
    let mut want = Instance {
        instance_id: Some((1, 2).into()),
        cmds: vec![("Set", "x", "y").into(), ("Get", "a", "").into()],
        deps: Some(vec![dep!(11, 12), dep!(12, 13)].into()),
        ..Default::default()
    };

    assert_eq!(want, inst!((1, 2), [(x = y), (a)], [(11, 12), (12, 13)]));
    assert_eq!(want, inst!((1, 2), [(x = y), (a)], (11, [12, 13])));

    let iid = InstanceId::from((1, 2));
    assert_eq!(want, inst!(iid, [(x = y), (a)], (11, [12, 13])));

    want.instance_id = None;
    assert_eq!(want, inst!(None, [(x = y), (a)], (11, [12, 13])));
}

#[test]
fn test_macro_inst_instid_blt_cmds() {
    let want = Instance {
        instance_id: Some((1, 2).into()),
        ballot: Some((3, 1).into()),
        cmds: vec![("Set", "x", "y").into(), ("Get", "a", "").into()],
        ..Default::default()
    };

    assert_eq!(want, inst!((1, 2), (3, _), [(x = y), (a)]));

    let iid = InstanceId::from((1, 2));
    assert_eq!(want, inst!(iid, (3, _), [(x = y), (a)]));
}
#[test]
fn test_macro_inst_instid_blt_cmds_deps() {
    let want = Instance {
        instance_id: Some((1, 2).into()),
        ballot: Some((3, 1).into()),
        cmds: vec![("Set", "x", "y").into(), ("Get", "a", "").into()],
        deps: Some(vec![dep!(11, 12), dep!(12, 13)].into()),
        ..Default::default()
    };

    assert_eq!(
        want,
        inst!((1, 2), (3, _), [(x = y), (a)], [(11, 12), (12, 13)])
    );
    assert_eq!(want, inst!((1, 2), (3, _), [(x = y), (a)], (11, [12, 13])));

    let iid = InstanceId::from((1, 2));
    assert_eq!(want, inst!(iid, (3, _), [(x = y), (a)], (11, [12, 13])));
}

#[test]
fn test_macro_inst() {
    // instance_id, ballot, cmds
    let want = Instance {
        instance_id: Some((1, 2).into()),
        ballot: Some((4, 1).into()),
        cmds: vec![("Set", "x", "y").into(), ("Get", "a", "b").into()],
        deps: None,
        vballot: None,
        committed: false,
    };

    assert_eq!(
        want,
        inst!((1, 2), (4, _), [("Set", "x", "y"), ("Get", "a", "b")])
    );

    // instance_id, ballot, cmds, deps
    let want = Instance {
        instance_id: Some((1, 2).into()),
        ballot: Some((4, 1).into()),
        cmds: vec![("Set", "x", "y").into(), ("Get", "a", "b").into()],
        deps: Some(Deps {
            dep_vec: vec![(11, 12).into(), (13, 14).into()],
        }),
        vballot: None,
        committed: false,
    };

    assert_eq!(
        want,
        inst!(
            (1, 2),
            (4, _),
            [("Set", "x", "y"), ("Get", "a", "b")],
            [(11, 12), (13, 14)]
        )
    );

    // all arg
    let want = Instance {
        instance_id: Some((1, 2).into()),
        ballot: Some((4, 3).into()),
        cmds: vec![("Set", "x", "y").into(), ("Get", "a", "b").into()],
        deps: Some(Deps {
            dep_vec: vec![(11, 12).into(), (13, 14).into()],
        }),
        vballot: Some((2, 3).into()),
        committed: true,
    };

    assert_eq!(
        want,
        inst!(
            (1, 2),
            (4, 3),
            [("Set", "x", "y"), ("Get", "a", "b")],
            [(11, 12), (13, 14)],
            (2, 3),
            true
        )
    );
}

#[test]
fn test_macro_inst_kv() {
    let want = Instance {
        instance_id: Some((1, 2).into()),
        ..Default::default()
    };
    assert_eq!(want, inst!(instance_id:(1, 2)));

    let want = Instance {
        instance_id: None,
        ..Default::default()
    };
    assert_eq!(want, inst!(instance_id: None));

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
            ballot: Some((2, 3).into()),
            ..Default::default()
        },
        inst!(ballot:(2, 3))
    );

    assert_eq!(
        Instance {
            vballot: Some((2, 3).into()),
            ..Default::default()
        },
        inst!(vballot:(2, 3))
    );

    assert_eq!(
        Instance {
            ballot: None,
            ..Default::default()
        },
        inst!(ballot: None)
    );

    let want = Instance {
        cmds: vec![("Set", "x", "y").into()],
        ..Default::default()
    };

    assert_eq!(want, inst!(cmds:[("Set", "x", "y")]));
    assert_eq!(want, inst!(cmds:[(x=y)]));

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
