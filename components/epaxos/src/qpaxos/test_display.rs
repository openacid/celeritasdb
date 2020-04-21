use crate::qpaxos::BallotNum;
use crate::qpaxos::Command;
use crate::qpaxos::Instance;
use crate::qpaxos::InstanceId;
use crate::qpaxos::InstanceIdVec;
use crate::qpaxos::OpCode;

#[test]
fn test_display_instance_id() {
    assert_eq!(
        "(1, 2)",
        format!(
            "{}",
            InstanceId {
                replica_id: 1,
                idx: 2
            }
        )
    );
}

#[test]
fn test_display_ballot() {
    assert_eq!("(1, 2, 3)", format!("{}", BallotNum::from((1, 2, 3))));
}

#[test]
fn test_display_command() {
    let k: Vec<u8> = "foo".into();
    let v: Vec<u8> = "bar".into();
    assert_eq!(
        "NoOp",
        format!(
            "{}",
            Command {
                op: OpCode::NoOp as i32,
                key: k.clone(),
                value: v.clone()
            }
        )
    );
    assert_eq!(
        "Get:foo",
        format!(
            "{}",
            Command {
                op: OpCode::Get as i32,
                key: k.clone(),
                value: v.clone()
            }
        )
    );
    assert_eq!(
        "Set:foo=bar",
        format!(
            "{}",
            Command {
                op: OpCode::Set as i32,
                key: k.clone(),
                value: v.clone()
            }
        )
    );
    assert_eq!(
        "Delete:foo",
        format!(
            "{}",
            Command {
                op: OpCode::Delete as i32,
                key: k.clone(),
                value: v.clone()
            }
        )
    );
}

#[test]
fn test_display_instance_id_vec() {
    assert_eq!(
        "[(1, 2), (3, 4)]",
        format!("{}", InstanceIdVec::from(instids![(1, 2), (3, 4)]))
    );
}

#[test]
fn test_display_instance() {
    let inst = inst!(
        (1, 2),
        (2, 3, 4),
        [("Set", "a", "b"), ("Get", "c", "d")],
        [(2, 3), (3, 4)],
        [(3, 4), (4, 5)],
        [(4, 5), (5, 6)],
        false,
        true,
    );
    assert_eq!("{id:(1, 2), blt:(2, 3, 4), cmds:[Set:a=b, Get:c], deps:[(2, 3), (3, 4)][(3, 4), (4, 5)][(4, 5), (5, 6)], c/e:false/true}",
    format!("{}", inst));
}
