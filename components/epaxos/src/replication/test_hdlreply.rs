use crate::qpaxos::*;
use crate::replica::*;
use crate::replication::*;

#[cfg(test)]
use pretty_assertions::assert_eq;

/// replcmn makes a ReplyCommon.
/// Supported pattern:
/// replcmn!(None, None)
/// replcmn!(None, instid)
/// replcmn!(ballot, None)
/// replcmn!(ballot, instid)
macro_rules! replcmn {
    (None, None) => {
        ReplyCommon {
            last_ballot: None,
            instance_id: None,
        }
    };
    (None, $id:expr) => {
        ReplyCommon {
            last_ballot: None,
            instance_id: Some($id.into()),
        }
    };
    ($blt:expr, None) => {
        ReplyCommon {
            last_ballot: Some($blt.into()),
            instance_id: None,
        }
    };
    ($blt:expr, $id:expr) => {
        ReplyCommon {
            last_ballot: Some($blt.into()),
            instance_id: Some($id.into()),
        }
    };
}

/// deps makes a Some(InstanceIdVec) or None
/// Supported pattern:
/// deps!(None)
/// deps!([instid, instid...])
macro_rules! deps {
    (None) => {
        None
    };
    ([$(($rid:expr, $idx:expr)),*]) => {
        Some(instids![$(($rid, $idx)),*].into())
    };
}

#[test]
fn test_handle_fast_accept_reply_err() {
    macro_rules! frepl {
        () => {
            FastAcceptReply {
                ..Default::default()
            }
        };
        (($blt:tt, $id:tt)) => {
            FastAcceptReply {
                cmn: Some(replcmn!($blt, $id)),
                ..Default::default()
            }
        };
        (($blt:tt, $id:tt), $deps:tt) => {
            FastAcceptReply {
                cmn: Some(replcmn!($blt, $id)),
                deps: deps!($deps),
                ..Default::default()
            }
        };
        (($blt:tt, $id:tt), $deps:tt, $cmts:expr) => {
            FastAcceptReply {
                cmn: Some(replcmn!($blt, $id)),
                deps: deps!($deps),
                deps_committed: $cmts,
                ..Default::default()
            }
        };
    }

    let inst = init_inst!((1, 2), [("Set", "x", "1")], [(1, 1)]);
    let mut st = Status::new(3, &inst);

    let cases: Vec<(FastAcceptReply, HandlerError)> = vec![
        (frepl!(), ProtocolError::LackOf("cmn".into()).into()),
        (
            frepl!((None, None)),
            ProtocolError::LackOf("cmn.last_ballot".into()).into(),
        ),
        (
            frepl!((None, (1, 2))),
            ProtocolError::LackOf("cmn.last_ballot".into()).into(),
        ),
        (
            frepl!(((1, 2, 3), None)),
            ProtocolError::LackOf("cmn.instance_id".into()).into(),
        ),
        (
            frepl!(((1, 2, 3), (1, 2)), None),
            ProtocolError::LackOf("deps".into()).into(),
        ),
        (
            frepl!(((1, 2, 3), (1, 2)), [(1, 2), (2, 3)], vec![true]),
            ProtocolError::Incomplete("deps_committed".into(), 2, 1).into(),
        ),
    ];

    for (from_rid, (repl, want)) in cases.iter().enumerate() {
        let r = handle_fast_accept_reply(&mut st, from_rid as ReplicaID, repl);
        assert_eq!(r.err().unwrap(), *want);
    }
}

#[test]
fn test_handle_fast_accept_reply() {
    macro_rules! frepl {
        (($blt:tt, $id:tt), $deps:tt, $cmts:expr) => {
            FastAcceptReply {
                cmn: Some(replcmn!($blt, $id)),
                deps: deps!($deps),
                deps_committed: $cmts,
                ..Default::default()
            }
        };
    }

    let inst = init_inst!((1, 2), [("Set", "x", "1")], [(1, 1)]);
    let mut st = Status::new(3, &inst);

    {
        // positive reply updates the Status.
        let repl: FastAcceptReply =
            frepl!(((0, 0, 1), (1, 2)), [(1, 2), (2, 3)], vec![false, true]);
        let from_rid = 5;

        let r = handle_fast_accept_reply(&mut st, from_rid, &repl);
        assert_eq!(r.unwrap(), ());
        assert_eq!(st.fast_replied[&from_rid], true);
        assert_eq!(st.fast_deps[&1], vec![(1, 2).into()]);
        assert_eq!(st.fast_deps[&2], vec![(2, 3).into()]);
        assert!(st.fast_committed[&instid!(2, 3)]);
        assert_eq!(None, st.fast_committed.get(&instid!(1, 2)));
    }
    {
        // greater ballot should be ignored
        let repl: FastAcceptReply = frepl!(((100, 0, 1), (1, 2)), [(3, 4)], vec![true]);
        let from_rid = 4;

        let r = handle_fast_accept_reply(&mut st, from_rid, &repl);
        assert_eq!(
            r.err().unwrap(),
            HandlerError::StaleBallot((0, 0, 1).into(), (100, 0, 1).into())
        );
        assert_eq!(
            true,
            st.fast_replied.contains_key(&from_rid),
            "reply with higher ballot is still be recorded"
        );
        assert_eq!(false, st.fast_deps.contains_key(&3));
        assert_eq!(false, st.fast_committed.contains_key(&instid!(3, 4)));
    }
    {
        // duplicated message
        let repl: FastAcceptReply = frepl!(((0, 0, 1), (1, 2)), [(3, 4)], vec![true]);
        let from_rid = 4;

        let r = handle_fast_accept_reply(&mut st, from_rid, &repl);
        assert_eq!(r.err().unwrap(), HandlerError::Dup(from_rid));
        assert_eq!(true, st.fast_replied.contains_key(&from_rid));
        assert_eq!(false, st.fast_deps.contains_key(&3));
        assert_eq!(false, st.fast_committed.contains_key(&instid!(3, 4)));
    }

    {
        // reply contains `err` is ignored
        let mut repl: FastAcceptReply = frepl!(((0, 0, 1), (1, 2)), [(3, 4)], vec![true]);
        repl.err = Some(QError {
            ..Default::default()
        });
        let from_rid = 6;

        let r = handle_fast_accept_reply(&mut st, from_rid, &repl);
        assert_eq!(
            r.err().unwrap(),
            HandlerError::RemoteError(repl.err.unwrap())
        );
        assert_eq!(
            true,
            st.fast_replied.contains_key(&from_rid),
            "error reply should be recorded"
        );
        assert_eq!(false, st.fast_deps.contains_key(&3));
        assert_eq!(false, st.fast_committed.contains_key(&instid!(3, 4)));
    }
}
