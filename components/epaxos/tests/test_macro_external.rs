// test using macro outside a crate.
// every test is a single crate

use epaxos::inst;
use epaxos::qpaxos::BallotNum;
use epaxos::qpaxos::Command;
use epaxos::qpaxos::Dep;
use epaxos::qpaxos::Instance;
use epaxos::qpaxos::InstanceId;

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
