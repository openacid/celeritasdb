#[macro_export]
#[allow(unused_macros)]
macro_rules! cmd {
    ($op:expr, $key:expr, $val:expr) => {
        Command::from(($op, $key, $val))
    };
}

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
macro_rules! dep {
    ($replica_id:expr, $idx:expr) => {
        Dep::from(($replica_id, $idx))
    };

    ($replica_id:expr, $idx:expr, $seq:expr) => {
        Dep::from(($replica_id, $idx, $seq))
    };
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! deps {
    [] => {
        Vec::<Dep>::new()
    };

    [$(($replica_id:expr, $idx:expr)),*] => {
        vec![$(Dep::from(($replica_id, $idx))),*]
    };
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! instidvec {
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
/// init_inst!(inst_id, cmds, deps)
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
              [$(($dep_rid, $dep_idx)),*]
        )
    };
}

/// Create an instance with:
/// instance_id: (replica_id, idx),
/// ballot: (epoch, num, _). the `_` is a place holder indicating to use replica_is from instance_id.
/// cmds: [("Set", "x", "y")...]
/// deps: [(replica_id, idx)...]
///
/// Supported pattern:
/// inst!(instance_id, ballot, cmds, deps, acceptted, committed, executed)
/// inst!(instance_id, ballot, cmds, deps)
/// inst!(instance_id, ballot, cmds)
#[macro_export]
#[allow(unused_macros)]
macro_rules! inst {
    // instance_id, ballot, cmds
    ($id:expr,
     ($epoch:expr, $num:expr, _),
     [$( ($op:expr, $key:expr, $val:expr)),*]
     $(,)*
     ) => {
        Instance {
            instance_id: Some($id.into()),
            ballot: Some(($epoch, $num, InstanceId::from($id).replica_id).into()),
            cmds: cmds![$( ($op, $key, $val)),*].into(),
            ..Default::default()
        }
    };

    // // instance_id, ballot, cmds, dep=[]
    // ($id:expr,
    //  ($epoch:expr, $num:expr, _),
    //  [$( ($op:expr, $key:expr, $val:expr)),*],
    //  []
    //  $(,)*
    //  ) => {
    //     Instance {
    //         instance_id: Some($id.into()),
    //         ballot: Some(($epoch, $num, InstanceId::from($id).replica_id).into()),
    //         cmds: cmds![$( ($op, $key, $val)),*].into(),
    //         deps: Some(Vec::<Dep>::new().into()),
    //         ..Default::default()
    //     }
    // };

    // instance_id, ballot, cmds, deps
    ($id:expr,
     ($epoch:expr, $num:expr, _),
     [$( ($op:expr, $key:expr, $val:expr)),*],
     [$( ($dep_rid:expr, $dep_idx:expr)),*]
     $(,)*
     ) => {
        Instance {
            instance_id: Some($id.into()),
            ballot: Some(($epoch, $num, InstanceId::from($id).replica_id).into()),
            cmds: cmds![$( ($op, $key, $val)),*].into(),
            deps: Some(
                deps![$( ($dep_rid, $dep_idx)),*].into()
            ),
            ..Default::default()
        }
    };

    // all arg
    ($id:expr,
     ($epoch:expr, $num:expr, $brid:expr),
     [$( ($op:expr, $key:expr, $val:expr)),*],
     [$( ($dep_rid:expr, $dep_idx:expr)),*],
     $accepted:expr,
     $committed:expr,
     $executed:expr
     $(,)*
     ) => {
        Instance {
            instance_id: Some($id.into()),
            ballot: Some(($epoch, $num, $brid).into()),
            cmds: cmds![$( ($op, $key, $val)),*].into(),
            deps: Some(
                deps![$( ($dep_rid, $dep_idx)),*].into()
            ),
            accepted:$accepted,
            committed:$committed,
            executed:$executed,
            ..Default::default()
        }
    };
}
