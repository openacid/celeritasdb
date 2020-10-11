use std::sync::Arc;

use crate::qpaxos::Direction;
use crate::qpaxos::ReplicateReply;
use crate::qpaxos::*;
use crate::replica::*;
use crate::replication::*;
use crate::testutil;
use crate::StorageAPI;
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

/// deps makes a Some(Deps) or None
/// Supported pattern:
/// deps!(None)
/// deps!([instid, instid...])
macro_rules! deps {
    (None) => {
        None
    };
    ([$(($rid:expr, $idx:expr)),*]) => {
        Some(depvec![$(($rid, $idx)),*].into())
    };
}

macro_rules! frepl {
    () => {
        ReplicateReply {
            phase: Some(
                PrepareReply {
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
                PrepareReply {
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
                PrepareReply {
                    deps: deps!($deps),
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
                PrepareReply {
                    deps: deps!($deps),
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
fn test_handle_prepare_reply_err() {
    let inst = inst!((1, 2), (0, _), [(x = "1")], [(1, 1)]);

    let cases: Vec<(ReplicateReply, RpcHandlerError)> = vec![
        (
            frepl!((None, None)),
            ProtocolError::LackOf("instance_id".into()).into(),
        ),
        (
            frepl!(((2, 3), None)),
            ProtocolError::LackOf("instance_id".into()).into(),
        ),
        (
            frepl!(((2, 3), (1, 2)), None),
            ProtocolError::LackOf("phase".into()).into(),
        ),
        (
            ReplicateReply {
                last_ballot: blt!((0, 1)),
                instance_id: iid!((1, 2)),
                phase: Some(
                    CommitReply {
                        ..Default::default()
                    }
                    .into(),
                ),
                ..Default::default()
            },
            ProtocolError::LackOf("phase::Prepare".into()).into(),
        ),
        (
            frepl!(((0, 1), (1, 2)), (None)),
            ProtocolError::LackOf("phase::Prepare.deps".into()).into(),
        ),
        (
            frepl!(((0, 1), (1, 2)), ([(1, 2), (2, 3)], vec![true])),
            ProtocolError::Incomplete("phase::Prepare.deps_committed".into(), 2, 1).into(),
        ),
    ];

    for (repl, want) in cases.iter() {
        let mut st = ReplicationStatus::new(3, inst.clone());
        let r = handle_prepare_reply(&mut st, 3, repl.clone());
        assert_eq!(r.err().unwrap(), *want, "Prepare-reply: {:?}", repl);
    }
}

#[test]
fn test_handle_prepare_reply() {
    let inst = inst!((1, 2), (0, _), [(x = "1")], []);
    let mut st = ReplicationStatus::new(3, inst.clone());

    {
        // positive reply updates the Status.
        let repl: ReplicateReply = frepl!(((0, 1), (1, 2)), ([(1, 2), (2, 3)], vec![false, true]));
        let from_rid = 5;

        let r = handle_prepare_reply(&mut st, from_rid, repl.clone());
        assert_eq!(r.unwrap(), ());
        assert!(st.prepared[&1].replied.contains(&from_rid));

        assert_eq!(
            st.prepared[&1].rdeps,
            vec![RepliedDep {
                idx: 2,
                seq: 0,
                committed: false
            }]
        );
        assert_eq!(
            st.prepared[&2].rdeps,
            vec![RepliedDep {
                idx: 3,
                seq: 0,
                committed: true
            }]
        );
    }
    {
        // greater ballot should be ignored
        let repl: ReplicateReply = frepl!(((100, 1), (1, 2)), ([(3, 4)], vec![true]));
        let from_rid = 4;

        let r = handle_prepare_reply(&mut st, from_rid, repl.clone());
        assert_eq!(
            r.err().unwrap(),
            RpcHandlerError::StaleBallot((0, 1).into(), (100, 1).into())
        );
        assert_eq!(
            false,
            st.prepared[&1].replied.contains(&from_rid),
            "reply with higher ballot is still be recorded"
        );

        assert_eq!(false, st.prepared.contains_key(&3));
    }
    {
        // duplicated message

        let inst = inst!((1, 2), (0, _), [(x = "1")], []);
        let mut st = ReplicationStatus::new(3, inst.clone());

        let repl: ReplicateReply = frepl!(((0, 1), (1, 2)), ([(3, 4)], vec![true]));
        let from_rid = 4;

        let r = handle_prepare_reply(&mut st, from_rid, repl.clone());
        assert_eq!(None, r.err());

        let r = handle_prepare_reply(&mut st, from_rid, repl.clone());
        assert_eq!(
            r.err().unwrap(),
            RpcHandlerError::DupRpc(
                InstanceStatus::Prepared,
                Direction::Reply,
                from_rid,
                st.instance.instance_id.unwrap()
            )
        );
    }

    {
        // reply contains `err` is ignored
        let mut repl: ReplicateReply = frepl!(((0, 1), (1, 2)), ([(3, 4)], vec![true]));
        repl.err = Some(QError {
            ..Default::default()
        });
        let from_rid = 6;

        let r = handle_prepare_reply(&mut st, from_rid, repl.clone());
        assert_eq!(
            r.err().unwrap(),
            RpcHandlerError::RemoteError(repl.err.unwrap())
        );
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

    let mut inst = inst!((1, 2), (0, _), [(x = "1")], []);
    inst.deps = Some(depvec![].into());
    rp.storage.set_instance(&inst).unwrap();
    let n = rp.group_replica_ids.len() as i32;

    {
        // with high ballot num
        let mut st = ReplicationStatus::new(n, inst.clone());
        st.start_accept();
        let repl = ReplicateReply {
            last_ballot: Some((10, replica_id).into()),
            ..Default::default()
        };
        let r = handle_accept_reply(&mut st, 0, repl.clone());
        println!("{:?}", r);
        assert!(r.is_err());

        assert_eq!(st.get_slowpath_deps(&rp.group_replica_ids), None);
        assert_eq!(1, st.accepted.len());
    }

    {
        // with reply err
        let mut st = ReplicationStatus::new(n, inst.clone());
        st.start_accept();
        let repl = ReplicateReply {
            err: Some(ProtocolError::LackOf("test".to_string()).into()),
            ..Default::default()
        };
        let r = handle_accept_reply(&mut st, 0, repl.clone());
        println!("{:?}", r);
        assert!(r.is_err());

        assert_eq!(st.get_slowpath_deps(&rp.group_replica_ids), None);

        assert_eq!(1, st.accepted.len());
    }

    {
        // success
        inst.vballot = Some((2, 3).into());
        let mut st = ReplicationStatus::new(n, inst.clone());
        st.start_accept();
        let repl = ReplicateReply {
            err: None,
            last_ballot: Some((0, 0).into()),
            instance_id: inst.instance_id,
            phase: Some(AcceptReply {}.into()),
        };
        let r = handle_accept_reply(&mut st, 0, repl.clone());
        println!("{:?}", r);
        assert!(r.is_ok());

        assert_eq!(2, st.accepted.len());
    }
}
