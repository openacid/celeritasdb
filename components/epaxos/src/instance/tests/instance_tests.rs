use super::*;
use crate::command::{Command, OpCode};
use protobuf::{parse_from_bytes, Message};
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

    let inst_bytes: Vec<u8> = inst1.write_to_bytes().unwrap();

    let inst2 = parse_from_bytes::<Instance>(&inst_bytes).unwrap();

    assert_eq!(inst2.cmds.into_vec(), cmds);
    assert_eq!(*inst2.ballot.get_ref(), ballot);
    for (idx, inst_id) in inst2.initial_deps.iter().enumerate() {
        assert_eq!(*inst_id, initial_deps[idx]);
    }
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
