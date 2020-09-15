// test using macro outside a crate.
// every test is a single crate

use epaxos::inst;
use epaxos::dep;
use epaxos::depvec;
use epaxos::cmdvec;
use epaxos::qpaxos::BallotNum;
use epaxos::qpaxos::Command;
use epaxos::qpaxos::Dep;
use epaxos::qpaxos::Deps;
use epaxos::qpaxos::Instance;
use epaxos::qpaxos::InstanceId;

#[test]
fn test_external_macro_inst() {
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
        deps: Some(Deps {
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
        deps: Some(Deps {
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
fn test_external_macro_inst_kv() {
    assert_eq!(
        Instance {
            instance_id: Some((1, 2).into()),
            ..Default::default()
        },
        inst!(instance_id:(1, 2))
    );

    assert_eq!(
        Instance {
            ballot: Some((1, 2, 3).into()),
            ..Default::default()
        },
        inst!(ballot:(1, 2, 3))
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
