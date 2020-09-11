use crate::qpaxos::*;

#[test]
fn test_instance_id_from() {
    let id = InstanceId {
        replica_id: 3,
        idx: 4,
    };
    assert_eq!(id, (3, 4).into());
    assert_eq!(id, (3i32, 4i64).into());
}

#[test]
fn test_macro_instid() {
    let id = instid!(1, 2);

    assert_eq!(InstanceId::from((1, 2)), id);
}

#[test]
fn test_macro_instids() {
    let ids = instids![(1, 2), (3i32, 4i64)];

    assert_eq!(InstanceId::from((1, 2)), ids[0]);
    assert_eq!(InstanceId::from((3, 4)), ids[1]);
    assert_eq!(2, ids.len());
}

#[test]
fn test_macro_ballot() {
    let b = ballot!(1, 2, 3);

    assert_eq!(BallotNum::from((1, 2, 3)), b);
}

#[test]
fn test_macro_inst_all_arg() {
    let want = Instance {
        instance_id: Some((1, 2).into()),
        ballot: Some((3, 4, 2).into()),
        cmds: vec![("Set", "x", "y").into(), ("Get", "a", "b").into()],
        deps: Some(DepVec {
            ids: vec![(12, 13).into(), (14, 15).into()],
        }),
        accepted: true,
        committed: true,
        executed: true,
    };

    assert_eq!(
        want,
        inst!(
            (1, 2),
            (3, 4, 2),
            [("Set", "x", "y"), ("Get", "a", "b")],
            [(12, 13), (14, 15)],
            true,
            true,
            true,
        )
    );

    // id is InstanceId
    assert_eq!(
        want,
        inst!(
            InstanceId::from((1, 2)),
            (3, 4, 2),
            [("Set", "x", "y"), ("Get", "a", "b")],
            [(12, 13), (14, 15)],
            true,
            true,
            true,
        )
    );
}

#[test]
fn test_macro_inst() {
    let mut want = Instance {
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

    // only initial_deps
    assert_eq!(
        want,
        inst!(
            (1, 2),
            (3, 4, _),
            [("Set", "x", "y"), ("Get", "a", "b")],
            [(11, 12), (13, 14)]
        )
    );

    // deps
    want.deps = Some(instids![(10, 0), (11, 12)].into());
    assert_eq!(
        want,
        inst!(
            (1, 2),
            (3, 4, _),
            [("Set", "x", "y"), ("Get", "a", "b")],
            [(10, 0), (11, 12)],
        )
    );

    // initial_deps is None
    want.deps = None;
    assert_eq!(
        want,
        inst!((1, 2), (3, 4, _), [("Set", "x", "y"), ("Get", "a", "b")],)
    );
}

#[test]
fn test_instance_conflict() {
    let nx = Command::from(("NoOp", "x", "1"));
    let gx = Command::from(("Get", "x", "1"));
    let sx = Command::from(("Set", "x", "1"));

    let ny = Command::from(("NoOp", "y", "1"));
    let _gy = Command::from(("Get", "y", "1"));
    let sy = Command::from(("Set", "y", "1"));

    let nxny = Instance::of(&[nx.clone(), ny.clone()], (0, 0, 0).into(), &[]);
    let gxny = Instance::of(&[gx.clone(), ny.clone()], (0, 0, 0).into(), &[]);
    let sxny = Instance::of(&[sx.clone(), ny.clone()], (0, 0, 0).into(), &[]);
    let sxsy = Instance::of(&[sx.clone(), sy.clone()], (0, 0, 0).into(), &[]);
    let gxsy = Instance::of(&[gx.clone(), sy.clone()], (0, 0, 0).into(), &[]);

    assert!(!nxny.conflict(&nxny));
    assert!(!nxny.conflict(&gxny));
    assert!(!nxny.conflict(&sxny));
    assert!(!nxny.conflict(&sxsy));
    assert!(!nxny.conflict(&gxsy));

    assert!(!gxny.conflict(&gxny));
    assert!(gxny.conflict(&sxny));
    assert!(gxny.conflict(&sxsy));
    assert!(!gxny.conflict(&gxsy));

    assert!(sxny.conflict(&sxny));
    assert!(sxny.conflict(&sxsy));
    assert!(sxny.conflict(&gxsy));

    assert!(sxsy.conflict(&sxsy));
    assert!(sxsy.conflict(&gxsy));

    assert!(gxsy.conflict(&gxsy));
}
