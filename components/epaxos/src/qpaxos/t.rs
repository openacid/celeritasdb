use super::*;
// Message is required to use to use method in trait Message.
use prost::Message;

use std::str;

fn new_foo_inst() -> Instance {
    let replica = 1;

    let inst_id1 = InstanceID::of(1, 10);
    let inst_id2 = InstanceID::of(2, 20);
    let inst_id3 = InstanceID::of(3, 30);
    let initial_deps = vec![inst_id1.clone(), inst_id2.clone(), inst_id3.clone()];

    let cmd1 = Command::of(OpCode::NoOp, "k1".as_bytes(), "v1".as_bytes());
    let cmd2 = Command::of(OpCode::Get, "k2".as_bytes(), "v2".as_bytes());
    let cmds = vec![cmd1, cmd2];
    let ballot = BallotNum::of(0, 0, replica);
    let ballot2 = BallotNum::of(1, 2, replica);

    let mut inst = Instance::of(&cmds[..], &ballot, &initial_deps[..]);
    // TODO move these to Instance::new_instance
    inst.instance_id = Some(inst_id1);
    inst.deps = [inst_id2].to_vec();
    inst.final_deps = [inst_id3].to_vec();
    inst.last_ballot = Some(ballot2);

    inst
}

// TODO test to_replica_id

macro_rules! test_request_common {
    ($msg:ident, $inst:ident) => {
        assert_eq!($inst.ballot, $msg.cmn.as_ref().unwrap().ballot);
        assert_eq!($inst.instance_id, $msg.cmn.as_ref().unwrap().instance_id);
    };
}

macro_rules! test_reply_common {
    ($msg:ident, $inst:ident) => {
        assert_eq!($inst.last_ballot, $msg.cmn.as_ref().unwrap().last_ballot);
        assert_eq!($inst.instance_id, $msg.cmn.as_ref().unwrap().instance_id);
    };
}

#[test]
fn test_instance_protobuf() {
    let inst_id1 = (1, 10).into();
    let inst_id2 = (2, 20).into();
    let inst_id3 = (3, 30).into();
    let initial_deps = vec![inst_id1, inst_id2, inst_id3];

    let cmd1 = Command::of(OpCode::NoOp, "k1".as_bytes(), "v1".as_bytes());
    let cmd2 = Command::of(OpCode::Get, "k2".as_bytes(), "v2".as_bytes());
    let cmds = vec![cmd1, cmd2];
    let ballot = (1, 2, 3).into();

    let inst1 = Instance::of(&cmds[..], &ballot, &initial_deps[..]);

    test_enc_dec!(inst1, Instance);
}

#[test]
fn test_instanceid_derived() {
    let inst_id1 = InstanceID::of(1, 10);
    let inst_id2 = inst_id1;

    assert_eq!(inst_id1, inst_id2);
    assert_eq!(inst_id1, (1, 10).into());
}

#[test]
fn test_ballotnum_derived() {
    let b1 = BallotNum::of(1, 10, 5);
    let b2 = b1;

    assert_eq!(b1, b2);
    assert_eq!(b1, (1, 10, 5).into());
}

#[test]
fn test_instance_id_to_key() {
    let k = InstanceID::of(1, 10).to_key();
    assert_eq!(
        "/instance/0000000000000001/000000000000000a",
        str::from_utf8(&k).unwrap()
    );

    let k = InstanceID::of(-1, -10).to_key();
    assert_eq!(
        "/instance/ffffffffffffffff/fffffffffffffff6",
        str::from_utf8(&k).unwrap()
    );
}

#[test]
fn test_cmp_instance_id() {
    let cases = vec![
        ((1, 10), (1, 10), "="),
        ((1, 10), (1, 10), "<="),
        ((1, 10), (1, 10), ">="),
        ((2, 10), (1, 10), ">"),
        ((2, 11), (1, 10), ">"),
        ((1, 10), (1, 11), "<"),
        ((1, 10), (2, 10), "<"),
        ((1, 10), (2, 12), "<"),
    ];

    for (t1, t2, op) in cases {
        let i1: InstanceID = t1.into();
        let i2: InstanceID = t2.into();
        match op {
            "=" => assert_eq!(i1 == i2, true),
            "<=" => assert_eq!(i1 <= i2, true),
            ">=" => assert_eq!(i1 >= i2, true),
            "<" => assert_eq!(i1 < i2, true),
            ">" => assert_eq!(i1 > i2, true),
            _ => {
                assert!(false, format!("Unknown op: {}", op));
            }
        };
    }
}

#[test]
fn test_instance_after() {
    let cases = vec![
        (
            Instance {
                final_deps: vec![(1, 1).into()],
                ..Default::default()
            },
            Instance {
                final_deps: vec![(1, 1).into()],
                ..Default::default()
            },
            false,
        ),
        (
            Instance {
                final_deps: vec![(1, 1).into()],
                ..Default::default()
            },
            Instance {
                final_deps: vec![(1, 0).into()],
                ..Default::default()
            },
            true,
        ),
        (
            Instance {
                final_deps: vec![(1, 1).into(), (2, 1).into()],
                ..Default::default()
            },
            Instance {
                final_deps: vec![(1, 1).into(), (2, 1).into()],
                ..Default::default()
            },
            false,
        ),
        (
            Instance {
                final_deps: vec![(1, 1).into(), (2, 1).into()],
                ..Default::default()
            },
            Instance {
                final_deps: vec![(1, 1).into(), (2, 0).into()],
                ..Default::default()
            },
            true,
        ),
        (
            Instance {
                final_deps: vec![(1, 1).into(), (2, 1).into(), (3, 1).into()],
                ..Default::default()
            },
            Instance {
                final_deps: vec![(1, 1).into(), (2, 1).into()],
                ..Default::default()
            },
            false,
        ),
        (
            Instance {
                final_deps: vec![(1, 1).into(), (2, 1).into()],
                ..Default::default()
            },
            Instance {
                final_deps: vec![(1, 1).into(), (2, 1).into(), (3, 1).into()],
                ..Default::default()
            },
            false,
        ),
        (
            Instance {
                final_deps: vec![(1, 2).into(), (2, 1).into()],
                ..Default::default()
            },
            Instance {
                final_deps: vec![(1, 1).into(), (2, 1).into(), (3, 1).into()],
                ..Default::default()
            },
            true,
        ),
    ];

    for (a, b, r) in cases {
        assert_eq!(r, a.after(&b));
    }
}

#[test]
fn test_command_protobuf() {
    let cmd1 = Command::of(OpCode::NoOp, "key".as_bytes(), "value".as_bytes());

    test_enc_dec!(cmd1, Command);
}

#[test]
fn test_request_prepare_pb() {
    let inst = new_foo_inst();

    let pp = MakeRequest::prepare(&inst);

    test_request_common!(pp, inst);
    // prepare has no other fields.

    test_enc_dec!(pp, PrepareRequest);
}

#[test]
fn test_reply_prepare_pb() {
    let inst = new_foo_inst();

    let pp = MakeReply::prepare(&inst);

    test_reply_common!(pp, inst);
    assert_eq!(inst.deps, pp.deps);
    assert_eq!(inst.final_deps, pp.final_deps);
    assert_eq!(inst.committed, pp.committed);

    test_enc_dec!(pp, PrepareReply);
}

#[test]
fn test_request_fast_accpt_pb() {
    let inst = new_foo_inst();

    let deps_committed = &[true, false];
    let pp = MakeRequest::fast_accept(&inst, deps_committed);

    test_request_common!(pp, inst);
    assert_eq!(inst.cmds, pp.cmds);
    assert_eq!(inst.initial_deps, pp.initial_deps);
    assert_eq!(deps_committed.to_vec(), pp.deps_committed);

    test_enc_dec!(pp, FastAcceptRequest);
}

#[test]
fn test_reply_fast_accept_pb() {
    let inst = new_foo_inst();

    let deps_committed = &[true, false];
    let pp = MakeReply::fast_accept(&inst, deps_committed);

    test_reply_common!(pp, inst);
    assert_eq!(inst.deps, pp.deps);
    assert_eq!(deps_committed.to_vec(), pp.deps_committed);

    test_enc_dec!(pp, FastAcceptReply);
}

#[test]
fn test_request_accept_pb() {
    let inst = new_foo_inst();

    let pp = MakeRequest::accept(&inst);

    test_request_common!(pp, inst);
    assert_eq!(inst.final_deps, pp.final_deps);

    test_enc_dec!(pp, AcceptRequest);
}

#[test]
fn test_reply_accept_pb() {
    let inst = new_foo_inst();

    let pp = MakeReply::accept(&inst);

    test_reply_common!(pp, inst);
    // no other fields.

    test_enc_dec!(pp, AcceptReply);
}

#[test]
fn test_request_commit_pb() {
    let inst = new_foo_inst();

    let pp = MakeRequest::commit(&inst);

    test_request_common!(pp, inst);
    assert_eq!(inst.cmds, pp.cmds);
    assert_eq!(inst.final_deps, pp.final_deps);

    test_enc_dec!(pp, CommitRequest);
}

#[test]
fn test_reply_commit_pb() {
    let inst = new_foo_inst();

    let pp = MakeReply::commit(&inst);

    test_reply_common!(pp, inst);
    // no other fields.

    test_enc_dec!(pp, CommitReply);
}
