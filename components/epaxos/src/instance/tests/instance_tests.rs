use super::*;
use crate::command::{Command, OpCode};
use prost::Message;
use std::str;

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
