#[cfg(test)]
use pretty_assertions::assert_eq;

use crate::qpaxos::*;

// TODO move other macro test in test_instance here.

#[test]
fn test_macro_init_inst() {
    let want = Instance {
        instance_id: Some((1, 2).into()),
        last_ballot: None,
        ballot: Some((0, 0, 1).into()),
        cmds: vec![("Set", "x", "y").into(), ("Get", "a", "b").into()],
        initial_deps: Some(InstanceIdVec {
            ids: vec![(11, 12).into(), (13, 14).into()],
        }),
        deps: Some(InstanceIdVec {
            ids: vec![(11, 12).into(), (13, 14).into()],
        }),
        final_deps: None,
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
