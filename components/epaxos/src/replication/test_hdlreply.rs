use std::sync::Arc;

use crate::qpaxos::Direction;
use crate::qpaxos::ReplicateReply;
use crate::qpaxos::*;
use crate::replica::*;
use crate::replication::*;
use crate::testutil;
use storage::MemEngine;

#[cfg(test)]
use pretty_assertions::assert_eq;

macro_rules! blt {
    (None) => {
        None
    };
    ($blt:expr) => {
        Some($blt.into())
    };
}

macro_rules! iid {
    (None) => {
        None
    };
    ($id:expr) => {
        Some($id.into())
    };
}

macro_rules! get {
    ($container:expr, $key:expr, None) => {
        assert_eq!($container.get($key), None);
    };

    ($container:expr, $key:expr, $want:expr) => {
        assert_eq!($container.get($key), Some(&$want));
    };
}

/// depvec makes a Some(DepVec) or None
/// Supported pattern:
/// depvec!(None)
/// depvec!([instid, instid...])
macro_rules! depvec {
    (None) => {
        None
    };
    ([$(($rid:expr, $idx:expr)),*]) => {
        Some(deps![$(($rid, $idx)),*].into())
    };
}

macro_rules! frepl {
    () => {
        ReplicateReply {
            phase: Some(
                FastAcceptReply {
                    ..Default::default()
                }
                .into(),
            ),
            ..Default::default()
        }
    };
    (($blt:tt, $id:tt)) => {
        ReplicateReply {
            last_ballot: blt!($blt),
            instance_id: iid!($id),
            phase: Some(
                FastAcceptReply {
                    ..Default::default()
                }
                .into(),
            ),
            ..Default::default()
        }
    };
    (($blt:tt, $id:tt), None) => {
        ReplicateReply {
            last_ballot: blt!($blt),
            instance_id: iid!($id),
            phase: None,
            ..Default::default()
        }
    };
    (($blt:tt, $id:tt), ($deps:tt)) => {
        ReplicateReply {
            last_ballot: blt!($blt),
            instance_id: iid!($id),
            phase: Some(
                FastAcceptReply {
                    deps: depvec!($deps),
                    ..Default::default()
                }
                .into(),
            ),
            ..Default::default()
        }
    };
    (($blt:tt, $id:tt), ($deps:tt, $cmts:expr)) => {
        ReplicateReply {
            last_ballot: blt!($blt),
            instance_id: iid!($id),
            phase: Some(
                FastAcceptReply {
                    deps: depvec!($deps),
                    deps_committed: $cmts,
                    ..Default::default()
                }
                .into(),
            ),
            ..Default::default()
        }
    };
}

#[test]
fn test_handle_fast_accept_reply_err() {
    let inst = init_inst!((1, 2), [("Set", "x", "1")], [(1, 1)]);

    let cases: Vec<(ReplicateReply, RpcHandlerError)> = vec![
        (
            frepl!((None, None)),
            ProtocolError::LackOf("instance_id".into()).into(),
        ),
        (
            frepl!(((1, 2, 3), None)),
            ProtocolError::LackOf("instance_id".into()).into(),
        ),
        (
            frepl!(((1, 2, 3), (1, 2)), None),
            ProtocolError::LackOf("phase".into()).into(),
        ),
        (
            ReplicateReply {
                last_ballot: blt!((0, 0, 1)),
                instance_id: iid!((1, 2)),
                phase: Some(
                    CommitReply {
                        ..Default::default()
                    }
                    .into(),
                ),
                ..Default::default()
            },
            ProtocolError::LackOf("phase::Fast".into()).into(),
        ),
        (
            frepl!(((0, 0, 1), (1, 2)), (None)),
            ProtocolError::LackOf("phase::Fast.deps".into()).into(),
        ),
        (
            frepl!(((0, 0, 1), (1, 2)), ([(1, 2), (2, 3)], vec![true])),
            ProtocolError::Incomplete("phase::Fast.deps_committed".into(), 2, 1).into(),
        ),
    ];

    for (repl, want) in cases.iter() {
        let mut st = Status::new(3, inst.clone());
        let r = handle_fast_accept_reply(&mut st, 3, repl.clone());
        assert_eq!(r.err().unwrap(), *want, "fast-reply: {:?}", repl);
    }
}

#[test]
fn test_handle_fast_accept_reply() {
    let inst = init_inst!((1, 2), [("Set", "x", "1")], []);
    let mut st = Status::new(3, inst.clone());

    {
        // positive reply updates the Status.
        let repl: ReplicateReply =
            frepl!(((0, 0, 1), (1, 2)), ([(1, 2), (2, 3)], vec![false, true]));
        let from_rid = 5;

        let r = handle_fast_accept_reply(&mut st, from_rid, repl.clone());
        assert_eq!(r.unwrap(), ());
        assert_eq!(st.fast_replied[&from_rid], true);
        get!(st.fast_oks, &from_rid, true);

        assert_eq!(st.fast_deps[&1], vec![Dep::from((1, 2))]);
        assert_eq!(st.fast_deps[&2], vec![Dep::from((2, 3))]);
        assert!(st.fast_committed[&instid!(2, 3)]);
        assert_eq!(None, st.fast_committed.get(&instid!(1, 2)));
    }
    {
        // greater ballot should be ignored
        let repl: ReplicateReply = frepl!(((100, 0, 1), (1, 2)), ([(3, 4)], vec![true]));
        let from_rid = 4;

        let r = handle_fast_accept_reply(&mut st, from_rid, repl.clone());
        assert_eq!(
            r.err().unwrap(),
            RpcHandlerError::StaleBallot((0, 0, 1).into(), (100, 0, 1).into())
        );
        assert_eq!(
            true,
            st.fast_replied.contains_key(&from_rid),
            "reply with higher ballot is still be recorded"
        );

        get!(st.fast_oks, &from_rid, None);

        assert_eq!(false, st.fast_deps.contains_key(&3));
        assert_eq!(false, st.fast_committed.contains_key(&instid!(3, 4)));
    }
    {
        // duplicated message
        let repl: ReplicateReply = frepl!(((0, 0, 1), (1, 2)), ([(3, 4)], vec![true]));
        let from_rid = 4;

        let r = handle_fast_accept_reply(&mut st, from_rid, repl.clone());
        assert_eq!(
            r.err().unwrap(),
            RpcHandlerError::DupRpc(
                InstanceStatus::FastAccepted,
                Direction::Reply,
                from_rid,
                st.instance.instance_id.unwrap()
            )
        );
        assert_eq!(true, st.fast_replied.contains_key(&from_rid));

        get!(st.fast_oks, &from_rid, None);

        assert_eq!(false, st.fast_deps.contains_key(&3));
        assert_eq!(false, st.fast_committed.contains_key(&instid!(3, 4)));
    }

    {
        // reply contains `err` is ignored
        let mut repl: ReplicateReply = frepl!(((0, 0, 1), (1, 2)), ([(3, 4)], vec![true]));
        repl.err = Some(QError {
            ..Default::default()
        });
        let from_rid = 6;

        let r = handle_fast_accept_reply(&mut st, from_rid, repl.clone());
        assert_eq!(
            r.err().unwrap(),
            RpcHandlerError::RemoteError(repl.err.unwrap())
        );
        assert_eq!(
            true,
            st.fast_replied.contains_key(&from_rid),
            "error reply should be recorded"
        );

        get!(st.fast_oks, &from_rid, None);

        assert_eq!(false, st.fast_deps.contains_key(&3));
        assert_eq!(false, st.fast_committed.contains_key(&instid!(3, 4)));
    }
}

#[test]
fn test_handle_accept_reply() {
    let replica_id = 2;
    let rp = testutil::new_replica(
        replica_id,
        vec![0, 1, 2],
        vec![],
        Arc::new(MemEngine::new().unwrap()),
    );

    let mut inst = init_inst!((1, 2), [("Set", "x", "1")], []);
    inst.deps = Some(deps![].into());
    rp.storage.set_instance(&inst).unwrap();
    let n = rp.group_replica_ids.len() as i32;

    {
        // with high ballot num
        let mut st = Status::new(n, inst.clone());
        st.start_accept();
        let repl = ReplicateReply {
            last_ballot: Some((10, 2, replica_id).into()),
            ..Default::default()
        };
        let r = handle_accept_reply(&mut st, 0, repl.clone());
        println!("{:?}", r);
        assert!(r.is_err());

        assert_eq!(st.get_accept_deps(&rp.group_replica_ids), None);
        assert_eq!(2, st.accept_replied.len());
        assert_eq!(1, st.accept_oks.len());
    }

    {
        // with reply err
        let mut st = Status::new(n, inst.clone());
        st.start_accept();
        let repl = ReplicateReply {
            err: Some(ProtocolError::LackOf("test".to_string()).into()),
            ..Default::default()
        };
        let r = handle_accept_reply(&mut st, 0, repl.clone());
        println!("{:?}", r);
        assert!(r.is_err());

        assert_eq!(st.get_accept_deps(&rp.group_replica_ids), None);

        assert_eq!(2, st.accept_replied.len());
        assert_eq!(1, st.accept_oks.len());
    }

    {
        // success
        inst.accepted = true;
        let mut st = Status::new(n, inst.clone());
        st.start_accept();
        let repl = ReplicateReply {
            err: None,
            last_ballot: Some((0, 0, 0).into()),
            instance_id: inst.instance_id,
            phase: Some(AcceptReply {}.into()),
        };
        let r = handle_accept_reply(&mut st, 0, repl.clone());
        println!("{:?}", r);
        assert!(r.is_ok());

        assert_eq!(2, st.accept_replied.len());
        assert_eq!(2, st.accept_oks.len());
    }
}
