use super::*;
use crate::command::{Command, OpCode};
use prost::Message;
use std::str;

#[test]
fn test_instance_protobuf() {
    let inst_id1 = InstanceID::of(1, 10);
    let inst_id2 = InstanceID::of(2, 20);
    let inst_id3 = InstanceID::of(3, 30);
    let initial_deps = vec![inst_id1, inst_id2, inst_id3];

    let cmd1 = Command::of(OpCode::NoOp, "k1".as_bytes(), "v1".as_bytes());
    let cmd2 = Command::of(OpCode::Get, "k2".as_bytes(), "v2".as_bytes());
    let cmds = vec![cmd1, cmd2];
    let ballot = BallotNum::of(1, 2, 3);

    let inst1 = Instance::of(&cmds[..], &ballot, &initial_deps[..]);

    test_enc_dec!(inst1, Instance);
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
        (InstanceID::of(1, 10), InstanceID::of(1, 10), "="),
        (InstanceID::of(1, 10), InstanceID::of(1, 10), "<="),
        (InstanceID::of(1, 10), InstanceID::of(1, 10), ">="),
        (InstanceID::of(2, 10), InstanceID::of(1, 10), ">"),
        (InstanceID::of(2, 11), InstanceID::of(1, 10), ">"),
        (InstanceID::of(1, 10), InstanceID::of(1, 11), "<"),
        (InstanceID::of(1, 10), InstanceID::of(2, 10), "<"),
        (InstanceID::of(1, 10), InstanceID::of(2, 12), "<"),
    ];

    for (i1, i2, op) in cases {
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
                final_deps: vec![InstanceID::of(1, 1)],
                ..Default::default()
            },
            Instance {
                final_deps: vec![InstanceID::of(1, 1)],
                ..Default::default()
            },
            false,
        ),
        (
            Instance {
                final_deps: vec![InstanceID::of(1, 1)],
                ..Default::default()
            },
            Instance {
                final_deps: vec![InstanceID::of(1, 0)],
                ..Default::default()
            },
            true,
        ),
        (
            Instance {
                final_deps: vec![InstanceID::of(1, 1), InstanceID::of(2, 1)],
                ..Default::default()
            },
            Instance {
                final_deps: vec![InstanceID::of(1, 1), InstanceID::of(2, 1)],
                ..Default::default()
            },
            false,
        ),
        (
            Instance {
                final_deps: vec![InstanceID::of(1, 1), InstanceID::of(2, 1)],
                ..Default::default()
            },
            Instance {
                final_deps: vec![InstanceID::of(1, 1), InstanceID::of(2, 0)],
                ..Default::default()
            },
            true,
        ),
        (
            Instance {
                final_deps: vec![
                    InstanceID::of(1, 1),
                    InstanceID::of(2, 1),
                    InstanceID::of(3, 1),
                ],
                ..Default::default()
            },
            Instance {
                final_deps: vec![InstanceID::of(1, 1), InstanceID::of(2, 1)],
                ..Default::default()
            },
            false,
        ),
        (
            Instance {
                final_deps: vec![InstanceID::of(1, 1), InstanceID::of(2, 1)],
                ..Default::default()
            },
            Instance {
                final_deps: vec![
                    InstanceID::of(1, 1),
                    InstanceID::of(2, 1),
                    InstanceID::of(3, 1),
                ],
                ..Default::default()
            },
            false,
        ),
        (
            Instance {
                final_deps: vec![InstanceID::of(1, 2), InstanceID::of(2, 1)],
                ..Default::default()
            },
            Instance {
                final_deps: vec![
                    InstanceID::of(1, 1),
                    InstanceID::of(2, 1),
                    InstanceID::of(3, 1),
                ],
                ..Default::default()
            },
            true,
        ),
    ];

    for (a, b, r) in cases {
        assert_eq!(r, a.after(&b));
    }
}
