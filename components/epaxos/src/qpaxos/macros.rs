#[macro_export]
#[allow(unused_macros)]
macro_rules! cmds {
    [$(($op:expr, $key:expr, $val:expr)),*] => {
        vec![$(Command::from(($op, $key, $val))),*]
    }
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! instid {
    ($replica_id:expr, $idx:expr) => {
        InstanceId::from(($replica_id, $idx))
    };
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! instids {
    [$(($replica_id:expr, $idx:expr)),*] => {
        vec![$(InstanceId::from(($replica_id, $idx))),*]
    }
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! ballot {
    ($epoch:expr, $num:expr, $replica_id:expr) => {
        BallotNum {
            epoch: $epoch as i32,
            num: $num as i32,
            replica_id: $replica_id as i64,
        }
    };
}

/// Create a just initialized instance.
/// supported pattern:
/// init_inst!(inst_id, cmds, initial_deps)
///
/// Example:
/// ```
/// #[macro_use] extern crate epaxos;
/// use epaxos::qpaxos::*;
///
/// let inst = init_inst!((0, 1),
///            [("Set", "x", "1"), ("Set", "y", "2")],
///            [(1, 0), (2, 0), (3, 0)]);
/// println!("{:?}", inst);
/// ```
#[macro_export]
#[allow(unused_macros)]
macro_rules! init_inst {
    ($id:expr,
     [$( ($op:expr, $key:expr, $val:expr)),*],
     [$(($dep_rid:expr, $dep_idx:expr)),* $(,)*]
    ) => {
        inst!($id, (0, 0, _),
              [$( ($op, $key, $val)),*],
              [$(($dep_rid, $dep_idx)),*],
              "withdeps"
        )
    };
}

/// Create an instance with:
/// instance_id: (replica_id, idx),
/// ballot: (epoch, num, _). the `_` is a place holder indicating to use replica_is from instance_id.
/// cmds: [("Set", "x", "y")...]
/// initial_deps: [(replica_id, idx)...]
/// deps: [(replica_id, idx)...]; `deps` can be "withdeps" as a flag: clone initial_deps as its value.
///
/// Supported pattern:
/// inst!(instance_id, last_ballot, ballot, cmds, initial_deps, deps, final_deps, committed, executed)
/// inst!(instance_id, ballot, cmds, initial_deps, deps)
/// inst!(instance_id, ballot, cmds, initial_deps, "withdeps")
/// inst!(instance_id, ballot, cmds, initial_deps)
/// inst!(instance_id, ballot, cmds)
#[macro_export]
#[allow(unused_macros)]
macro_rules! inst {
    // instance_id, ballot, cmds, initial_deps=None
    ($id:expr,
     ($epoch:expr, $num:expr, _),
     [$( ($op:expr, $key:expr, $val:expr)),*]
     $(,)*
     ) => {
        Instance {
            initial_deps: None,
            ..inst!($id, ($epoch, $num, _),
                    [$(($op, $key, $val)),*],
                    [],
            )
        }
    };

    // instance_id, ballot, cmds, initial_deps
    ($id:expr,
     ($epoch:expr, $num:expr, _),
     [$( ($op:expr, $key:expr, $val:expr)),*],
     [$( ($idep_rid:expr, $idep_idx:expr)),*]
     $(,)*
     ) => {
        Instance {
            instance_id: Some($id.into()),
            ballot: Some(($epoch, $num, InstanceId::from($id).replica_id).into()),
            cmds: cmds![$( ($op, $key, $val)),*].into(),
            initial_deps: Some(
                instids![$( ($idep_rid, $idep_idx)),*].into()
            ),
            ..Default::default()
        }
    };

    // instance_id, ballot, cmds, initial_deps, deps=initial_deps
    ($id:expr,
     ($epoch:expr, $num:expr, _),
     [$( ($op:expr, $key:expr, $val:expr)),*],
     [$( ($idep_rid:expr, $idep_idx:expr)),*],
     "withdeps"
     $(,)*
     ) => {
        Instance {
            instance_id: Some($id.into()),
            ballot: Some(($epoch, $num, InstanceId::from($id).replica_id).into()),
            cmds: cmds![$( ($op, $key, $val)),*].into(),
            initial_deps: Some(
                instids![$( ($idep_rid, $idep_idx)),*].into()
            ),
            deps: Some(
                instids![$( ($idep_rid, $idep_idx)),*].into()
            ),
            ..Default::default()
        }
    };

    // instance_id, ballot, cmds, initial_deps, specified deps
    ($id:expr,
     ($epoch:expr, $num:expr, _),
     [$( ($op:expr, $key:expr, $val:expr)),*],
     [$( ($idep_rid:expr, $idep_idx:expr)),*],
     [$( ($dep_rid:expr, $dep_idx:expr)),*]
     $(,)*
     ) => {
        Instance {
            instance_id: Some($id.into()),
            ballot: Some(($epoch, $num, InstanceId::from($id).replica_id).into()),
            cmds: cmds![$( ($op, $key, $val)),*].into(),
            initial_deps: Some(
                instids![$( ($idep_rid, $idep_idx)),*].into()
            ),
            deps: Some(
                instids![$( ($dep_rid, $dep_idx)),*].into()
            ),
            ..Default::default()
        }
    };

    // all arg
    ($id:expr,
     ($lepoch:expr, $lnum:expr, $lbrid:expr),
     ($epoch:expr, $num:expr, $brid:expr),
     [$( ($op:expr, $key:expr, $val:expr)),*],
     [$( ($idep_rid:expr, $idep_idx:expr)),*],
     [$( ($dep_rid:expr, $dep_idx:expr)),*],
     [$( ($fdep_rid:expr, $fdep_idx:expr)),*],
     $committed:expr,
     $executed:expr
     $(,)*
     ) => {
        Instance {
            instance_id: Some($id.into()),
            last_ballot: Some(($lepoch, $lnum, $lbrid).into()),
            ballot: Some(($epoch, $num, $brid).into()),
            cmds: cmds![$( ($op, $key, $val)),*].into(),
            initial_deps: Some(
                instids![$( ($idep_rid, $idep_idx)),*].into()
            ),
            deps: Some(
                instids![$( ($dep_rid, $dep_idx)),*].into()
            ),
            final_deps: Some(
                instids![$( ($fdep_rid, $fdep_idx)),*].into()
            ),
            committed:$committed,
            executed:$executed,
            ..Default::default()
        }
    };
}
