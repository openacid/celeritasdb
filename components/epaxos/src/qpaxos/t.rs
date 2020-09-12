use super::*;
// Message is required to use to use method in trait Message.
use prost::Message;

use crate::qpaxos::ReplicateReply;
use crate::qpaxos::ReplicateRequest;

use std::str;

fn new_foo_inst() -> Instance {
    let replica = 1;

    let inst_id1 = InstanceId::from((1, 10));
    let inst_id2 = InstanceId::from((2, 20));

    let cmds = cmdvec![("NoOp", "k1", "v1"), ("Get", "k2", "v2")];
    let ballot = (0, 0, replica).into();

    let mut inst = Instance::of(&cmds[..], ballot, &[]);
    // TODO move these to Instance::new_instance
    inst.instance_id = Some(inst_id1);
    inst.deps = Some(vec![inst_id2].into());

    inst
}

// TODO test to_replica_id

macro_rules! test_request_common {
    ($msg:ident, $inst:ident, $to_rid:expr) => {
        assert_eq!($inst.ballot, $msg.ballot);
        assert_eq!($inst.instance_id, $msg.instance_id);
        assert_eq!($to_rid, $msg.to_replica_id);
    };
}

#[test]
fn test_instance_protobuf() {
    let inst_id1 = (1, 10).into();
    let inst_id2 = (2, 20).into();
    let inst_id3 = (3, 30).into();
    let deps = vec![inst_id1, inst_id2, inst_id3];

    let cmds = cmdvec![("NoOp", "k1", "v1"), ("Get", "k2", "v2")];
    let ballot = (1, 2, 3).into();

    let inst1 = Instance::of(&cmds[..], ballot, &deps[..]);

    test_enc_dec!(inst1, Instance);
}

#[test]
fn test_instanceid_derived() {
    let inst_id1 = InstanceId {
        replica_id: 1,
        idx: 10,
    };
    let inst_id2 = inst_id1;

    assert_eq!(inst_id1, inst_id2);
    assert_eq!(inst_id1, (1, 10).into());
    assert_eq!(inst_id1, InstanceId::from((1, 10)));
}

#[test]
fn test_ballotnum_derived() {
    let b1 = BallotNum {
        epoch: 1,
        num: 10,
        replica_id: 5,
    };
    let b2 = b1;

    assert_eq!(b1, b2);
    assert_eq!(b1, (1, 10, 5).into());
    assert_eq!(b1, BallotNum::from((1, 10, 5)));
}

#[test]
fn test_instance_id_to_key() {
    let k = InstanceId::from((1, 10)).to_key();
    assert_eq!(
        "/instance/0000000000000001/000000000000000a",
        str::from_utf8(&k).unwrap()
    );

    let k = InstanceId::from((-1, 0)).to_key();
    assert_eq!(
        "/instance/ffffffffffffffff/0000000000000000",
        str::from_utf8(&k).unwrap()
    );
}

#[test]
#[should_panic(expected = "idx can not be less than 0:-1")]
fn test_instance_id_to_key_negative() {
    InstanceId::from((1, -1)).to_key();
}

#[test]
fn test_instance_id_from_key() {
    assert_eq!(
        InstanceId::from_key("/instance/0000000000000001/000000000000000a").unwrap(),
        (1, 10).into()
    );

    assert_eq!(
        InstanceId::from_key("/instance/ffffffffffffffff/fffffffffffffff6").unwrap(),
        (-1, -10).into()
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
        let i1: InstanceId = t1.into();
        let i2: InstanceId = t2.into();
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
                deps: Some([(1, 1)].into()),
                ..Default::default()
            },
            Instance {
                deps: Some([(1, 1)].into()),
                ..Default::default()
            },
            false,
        ),
        (
            Instance {
                deps: Some([(1, 1)].into()),
                ..Default::default()
            },
            Instance {
                deps: Some([(1, 0)].into()),
                ..Default::default()
            },
            true,
        ),
        (
            Instance {
                deps: Some([(1, 1), (2, 1)].into()),
                ..Default::default()
            },
            Instance {
                deps: Some([(1, 1), (2, 1)].into()),
                ..Default::default()
            },
            false,
        ),
        (
            Instance {
                deps: Some([(1, 1), (2, 1)].into()),
                ..Default::default()
            },
            Instance {
                deps: Some([(1, 1), (2, 0)].into()),
                ..Default::default()
            },
            true,
        ),
        (
            Instance {
                deps: Some([(1, 1), (2, 1), (3, 1)].into()),
                ..Default::default()
            },
            Instance {
                deps: Some([(1, 1), (2, 1)].into()),
                ..Default::default()
            },
            false,
        ),
        (
            Instance {
                deps: Some([(1, 1), (2, 1)].into()),
                ..Default::default()
            },
            Instance {
                deps: Some([(1, 1), (2, 1), (3, 1)].into()),
                ..Default::default()
            },
            false,
        ),
        (
            Instance {
                deps: Some([(1, 2), (2, 1)].into()),
                ..Default::default()
            },
            Instance {
                deps: Some([(1, 1), (2, 1), (3, 1)].into()),
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
fn test_request_prepare_pb() {
    let inst = new_foo_inst();

    let pp = MakeRequest::prepare(100, &inst);
    test_enc_dec!(pp, ReplicateRequest);

    test_request_common!(pp, inst, 100);
    // prepare has no other fields.
}

#[test]
fn test_request_fast_accpt_pb() {
    let inst = new_foo_inst();

    let deps_committed = &[true, false];
    let pp = MakeRequest::fast_accept(100, &inst, deps_committed);
    test_enc_dec!(pp, ReplicateRequest);

    let req: FastAcceptRequest = pp.phase.unwrap().try_into().unwrap();

    test_request_common!(pp, inst, 100);
    assert_eq!(inst.cmds, req.cmds);
    assert_eq!(inst.deps, req.deps);
    assert_eq!(deps_committed.to_vec(), req.deps_committed);
}

#[test]
fn test_request_accept_pb() {
    let inst = new_foo_inst();

    let pp = MakeRequest::accept(100, &inst);
    test_enc_dec!(pp, ReplicateRequest);

    let req: AcceptRequest = pp.phase.unwrap().try_into().unwrap();

    test_request_common!(pp, inst, 100);
    assert_eq!(inst.deps, req.deps);
}

#[test]
fn test_request_commit_pb() {
    let inst = new_foo_inst();

    let pp = MakeRequest::commit(100, &inst);
    test_enc_dec!(pp, ReplicateRequest);

    let req: CommitRequest = pp.phase.unwrap().try_into().unwrap();

    test_request_common!(pp, inst, 100);
    assert_eq!(inst.cmds, req.cmds);
    assert_eq!(inst.deps, req.deps);
}

#[test]
fn test_replicate_reply_pb() {
    let reply = ReplicateReply {
        err: None,
        last_ballot: Some((1, 2, 3).into()),
        instance_id: Some(instid!(1, 2)),
        phase: Some(
            FastAcceptReply {
                deps: Some(instidvec![(1, 2), (3, 4)].into()),
                deps_committed: vec![true],
            }
            .into(),
        ),
    };

    test_enc_dec!(reply, ReplicateReply);
}
