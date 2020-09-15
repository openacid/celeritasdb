/// cmd creates a command. Supported pattern:
/// cmd!()  // NoOp
/// cmd!(x)  // Get x
/// cmd!(x=y)  // Set value of x to be y
/// cmd!(x="complex expr")  // Set value of x to be "complex expr"
/// cmd!("NoOp"|"Get"|"Set", "key", "value")  // build a command with operation, key and value.
#[macro_export]
#[allow(unused_macros)]
macro_rules! cmd {
    ($op:expr, $key:expr, $val:expr) => {
        Command::from(($op, $key, $val))
    };

    () => {
        Command::from(("NoOp", "", ""))
    };

    (NoOp) => {
        Command::from(("NoOp", "", ""))
    };

    // create a Get command with rust style syntax, just a single var `x`
    ($v:ident) => {
        Command::from(("Get", stringify!($v), ""))
    };

    // create a Set command with `x=val`
    ($v:ident = $what:ident) => {
        Command::from(("Set", stringify!($v), stringify!($what)))
    };

    ($v:ident = $what:expr) => {
        Command::from(("Set", stringify!($v), $what))
    };
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! cmdvec {
    [] => {
        Vec::<Command>::new()
    };

    [$($cmd:tt),*] => {
        vec![$($crate::cmd!$cmd),*]
    };
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
macro_rules! depvec {
    [] => {
        Vec::<Dep>::new()
    };

    [$(($replica_id:expr, $idx:expr)),*] => {
        vec![$(Dep::from(($replica_id, $idx))),*]
    };

    // create a Deps with replica id rid, rid+1, rid+2..
    ($rid:expr, [$($idx:expr),*]) => {
        {
            let mut v = Vec::<Dep>::new();
            let mut rid = $rid;
            let v0 =  vec![$($idx),*];
            for d in v0.iter() {
                v.push(Dep::from((rid, *d)));
                rid+=1;
            }

            v
        }
    };
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! instidvec {
    [] => {
        Vec::<InstanceId>::new()
    };

    [$(($replica_id:expr, $idx:expr)),*] => {
        vec![$(InstanceId::from(($replica_id, $idx))),*]
    };
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

#[doc(hidden)]
#[macro_export]
macro_rules! __instance_fields {
    ($field:expr, None) => { None };

    (instance_id, $v:tt) => { Some($crate::instid!$v) };
    (ballot, $v:tt) => { Some($crate::ballot!$v) };
    (accepted_ballot, $v:tt) => { Some($crate::ballot!$v) };
    (cmds, $v:tt) => { $crate::cmdvec!$v.into() };
    (deps, $v:tt) => { Some($crate::depvec!$v.into()) };

    // other fields
    ($field:expr, $v:tt) => { $v };
}

/// Create an instance with:
/// instance_id: (replica_id, idx),
/// ballot: (epoch, num, _). the `_` is a place holder indicating to use replica_is from instance_id.
/// cmds: [("Set", "x", "y")...]
/// deps: [(replica_id, idx)...]
///
/// Supported pattern:
/// inst!(instance_id, ballot, cmds, deps, accepted_ballot, committed, executed)
/// inst!(instance_id, ballot, cmds, deps)
/// inst!(instance_id, ballot, cmds)
/// inst!(instance_id, cmds, deps)
/// inst!(instance_id, cmds)
/// inst!(instance_id:(1, 2, 3), cmds:[("Set", "x", "2")], ...)
/// inst!(instance_id, cmds:[("Set", "x", "2")], ...)
///
/// Example:
/// ```
/// #[macro_use] extern crate epaxos;
/// use epaxos::qpaxos::*;
///
/// let inst = inst!((0, 1),
///            [(x="1"), (y="2")],
///            (1, [0, 0, 0]));
/// println!("{:?}", inst);
/// ```
#[macro_export]
#[allow(unused_macros)]
macro_rules! inst {

    // key:value...
    ( $($field:ident : $val:tt),*
    ) => {
        Instance {
            $( $field: $crate::__instance_fields!($field, $val) ),*
                ,
            ..Default::default()
        }
    };


    // instance_id, key:value...
    ($id:tt,
     $($field:ident : $val:tt),*
    ) => {
        Instance {
            instance_id: Some($crate::instid!$id),
            $( $field: $crate::__instance_fields!($field, $val) ),*
                ,
            ..Default::default()
        }
    };


    // instance_id, cmds
    ($id:expr,
     [$($cmd:tt),*]
     $(,)*
     ) => {
        Instance {
            instance_id: Some($id.into()),
            cmds: $crate::cmdvec![$($cmd),*].into(),
            ..Default::default()
        }
    };

    // instance_id, cmds, deps
    ($id:expr,
     [$($cmd:tt),*],
     $deps:tt
     $(,)*
     ) => {
        Instance {
            instance_id: Some($id.into()),
            cmds: $crate::cmdvec![$($cmd),*].into(),
            deps: Some($crate::depvec!$deps.into()),
            ..Default::default()
        }
    };

    // instance_id, ballot, cmds
    ($id:expr,
     ($epoch:expr, $num:expr, _),
     [$( ($op:expr, $key:expr, $val:expr)),*]
     $(,)*
     ) => {
        Instance {
            instance_id: Some($id.into()),
            ballot: Some(($epoch, $num, InstanceId::from($id).replica_id).into()),
            cmds: $crate::cmdvec![$( ($op, $key, $val)),*].into(),
            ..Default::default()
        }
    };

    // instance_id, ballot, cmds, deps
    ($id:expr,
     ($epoch:expr, $num:expr, _),
     [$($cmd:tt),*],
     $deps:tt
     $(,)*
     ) => {
        Instance {
            instance_id: Some($id.into()),
            ballot: Some(($epoch, $num, InstanceId::from($id).replica_id).into()),
            cmds: $crate::cmdvec![$($cmd),*].into(),
            deps: Some($crate::depvec!$deps.into()),
            ..Default::default()
        }
    };

    // all arg
    ($id:expr,
     ($epoch:expr, $num:expr, $brid:expr),
     [$( ($op:expr, $key:expr, $val:expr)),*],
     [$( ($dep_rid:expr, $dep_idx:expr)),*],
     $accepted_ballot:expr,
     $committed:expr,
     $executed:expr
     $(,)*
     ) => {
        Instance {
            instance_id: Some($id.into()),
            ballot: Some(($epoch, $num, $brid).into()),
            cmds: cmdvec![$( ($op, $key, $val)),*].into(),
            deps: Some(
                depvec![$( ($dep_rid, $dep_idx)),*].into()
            ),
            accepted_ballot:Some($accepted_ballot.into()),
            committed:$committed,
            executed:$executed
        }
    };
}
