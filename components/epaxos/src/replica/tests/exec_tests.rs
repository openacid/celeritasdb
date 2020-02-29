use std::net::TcpListener;

use crate::qpaxos::{Command, Instance, InstanceID, OpCode};
use crate::replica::{ExecuteResult, Replica, ReplicaConf, ReplicaStatus};
use crate::snapshot::MemEngine;

fn new_replica() -> Replica {
    return Replica {
        replica_id: 0,
        group_replica_ids: vec![],
        status: ReplicaStatus::Running,
        client_listener: TcpListener::bind("127.0.0.1:5001").unwrap(),
        listener: TcpListener::bind("127.0.0.1:6001").unwrap(),
        peers: vec![],
        conf: ReplicaConf {
            ..Default::default()
        },
        inst_idx: 0,
        latest_cp: (1, 1).into(),
        storage: Box::new(MemEngine::new().unwrap()),
        problem_inst_ids: vec![],
    };
}

#[test]
fn test_find_missing_instances() {
    let rp = new_replica();

    let cases1 = [
        (
            vec![Instance {
                instance_id: Some((1, 2).into()),
                final_deps: vec![(1, 1).into()],
                ..Default::default()
            }],
            vec![InstanceID::of(1, 1)],
        ),
        // R1               R2              R3
        // |                |               |
        // |                2(Committed)-.  10(Executed)
        // |                |             ↘ |
        // 2(Committed)--.---------------.  5
        // |              ↘ |             ↘ |
        // 1(Executed)      1(Executed)     3
        // |                |               |
        (
            vec![
                Instance {
                    instance_id: Some((1, 2).into()),
                    final_deps: vec![(1, 1).into(), (2, 1).into(), (3, 3).into()],
                    ..Default::default()
                },
                Instance {
                    instance_id: Some((2, 2).into()),
                    final_deps: vec![(1, 1).into(), (2, 1).into(), (3, 5).into()],
                    ..Default::default()
                },
            ],
            vec![InstanceID::of(1, 1), (2, 1).into(), (3, 10).into()],
        ),
    ];

    for (insts, up_to) in cases1.iter() {
        match rp.find_missing_insts(&insts, &up_to) {
            None => assert!(true),
            Some(_) => assert!(false),
        };
    }

    let case2 = [
        // R1               R2              R3
        // |                |               |
        // |                6(NotFound)    6(NotFound)
        // |              ↗ |             ↗ |
        // 2(Committed)--`---------------`  |
        // |                |               |
        // 1(Executed)      5(Executed)     5(Executed)
        // |                |               |
        //
        // need recover (2, 6) and (3, 6)
        (
            vec![Instance {
                instance_id: Some((1, 2).into()),
                final_deps: vec![(1, 1).into(), (2, 6).into(), (3, 6).into()],
                ..Default::default()
            }],
            vec![InstanceID::of(1, 1), (2, 5).into(), (3, 5).into()],
            vec![InstanceID::of(2, 6), (3, 6).into()],
        ),
        // R1               R2              R3
        // |                |               |
        // |      .---------2-------------→ 3(NotFound)
        // |    .`          |               |
        // 2(Committed)--.---------------.  2(NotFound)
        // |↙             ↘ |             ↘ |
        // 1(Executed)      1(Executed)     1(Executed)
        // |                |               |
        //
        // need recover (3, 2)
        (
            vec![
                Instance {
                    instance_id: Some((1, 2).into()),
                    final_deps: vec![(1, 1).into(), (2, 1).into(), (3, 1).into()],
                    ..Default::default()
                },
                Instance {
                    instance_id: Some((2, 2).into()),
                    final_deps: vec![(1, 1).into(), (2, 1).into(), (3, 3).into()],
                    ..Default::default()
                },
            ],
            vec![InstanceID::of(1, 1), (2, 1).into(), (3, 1).into()],
            vec![InstanceID::of(3, 2)],
        ),
    ];

    for (insts, up_to, exp) in case2.iter() {
        match rp.find_missing_insts(&insts, &up_to) {
            None => assert!(false),
            Some(s) => assert_eq!(exp, &s),
        };
    }
}

#[test]
fn test_execute_commands() {
    let mut rp = new_replica();

    match rp.storage.set_kv(vec![1], vec![11]) {
        Err(_) => assert!(false),
        Ok(_) => {}
    }
    match rp.storage.set_kv(vec![2], vec![22]) {
        Err(_) => assert!(false),
        Ok(_) => {}
    }
    let cases = [
        (
            Instance {
                instance_id: Some((2, 2).into()),
                cmds: vec![],
                ..Default::default()
            },
            Vec::<ExecuteResult>::new(),
        ),
        (
            Instance {
                instance_id: Some((2, 2).into()),
                cmds: vec![Command::of(OpCode::Get, &[96], &[])],
                ..Default::default()
            },
            vec![ExecuteResult::NotFound],
        ),
        (
            Instance {
                instance_id: Some((2, 2).into()),
                cmds: vec![Command::of(OpCode::Get, &[1], &[])],
                ..Default::default()
            },
            vec![ExecuteResult::SuccessWithVal { value: vec![11] }],
        ),
        (
            Instance {
                instance_id: Some((2, 2).into()),
                cmds: vec![
                    Command::of(OpCode::Get, &[1], &[]),
                    Command::of(OpCode::Get, &[2], &[]),
                ],
                ..Default::default()
            },
            vec![
                ExecuteResult::SuccessWithVal { value: vec![11] },
                ExecuteResult::SuccessWithVal { value: vec![22] },
            ],
        ),
        (
            Instance {
                instance_id: Some((2, 2).into()),
                cmds: vec![
                    Command::of(OpCode::NoOp, &[], &[]),
                    Command::of(OpCode::Set, &[2], &[222]),
                    Command::of(OpCode::Get, &[2], &[]),
                ],
                ..Default::default()
            },
            vec![
                ExecuteResult::Success,
                ExecuteResult::Success,
                ExecuteResult::SuccessWithVal { value: vec![222] },
            ],
        ),
    ];

    for (inst, res) in cases.iter() {
        match rp.execute_commands(&inst) {
            Ok(r) => assert_eq!(res, &r),
            Err(_) => assert!(false),
        }
    }
}

#[test]
fn test_execute_instances() {
    let mut rp = new_replica();

    // (3, 1)→(2, 1)→(1, 1)
    let min_insts = vec![
        Instance {
            instance_id: Some((1, 1).into()),
            cmds: vec![Command::of(OpCode::Set, &[2], &[222])],
            final_deps: vec![(1, 0).into(), (2, 0).into(), (3, 0).into()],
            ..Default::default()
        },
        Instance {
            instance_id: Some(InstanceID::of(2, 1)),
            cmds: vec![Command::of(OpCode::NoOp, &[], &[])],
            final_deps: vec![(1, 1).into(), (2, 0).into(), (3, 0).into()],
            ..Default::default()
        },
        Instance {
            instance_id: Some((3, 1).into()),
            cmds: vec![Command::of(OpCode::Set, &[1], &[11])],
            final_deps: vec![(1, 1).into(), (2, 1).into(), (3, 0).into()],
            ..Default::default()
        },
    ];

    match rp.execute_instances(&min_insts) {
        Ok(iids) => assert_eq!(vec![InstanceID::of(1, 1)], iids),
        Err(_) => assert!(false),
    };

    // (3, 1)~(2, 1)~(1, 1)
    let min_insts = vec![
        Instance {
            instance_id: Some((1, 1).into()),
            cmds: vec![Command::of(OpCode::Set, &[2], &[222])],
            final_deps: vec![(1, 0).into(), (2, 1).into(), (3, 0).into()],
            ..Default::default()
        },
        Instance {
            instance_id: Some(InstanceID::of(2, 1)),
            cmds: vec![Command::of(OpCode::NoOp, &[], &[])],
            final_deps: vec![(1, 1).into(), (2, 0).into(), (3, 0).into()],
            ..Default::default()
        },
        Instance {
            instance_id: Some((3, 1).into()),
            cmds: vec![Command::of(OpCode::Set, &[1], &[11])],
            final_deps: vec![(1, 1).into(), (2, 0).into(), (3, 0).into()],
            ..Default::default()
        },
    ];

    match rp.execute_instances(&min_insts) {
        Ok(iids) => assert_eq!(
            vec![InstanceID::of(1, 1), (2, 1).into(), (3, 1).into()],
            iids
        ),
        Err(_) => assert!(false),
    };

    // (3, 1) →(1, 1)
    //       ↘   ~
    //         (2, 1)
    let min_insts = vec![
        Instance {
            instance_id: Some((1, 1).into()),
            cmds: vec![Command::of(OpCode::Set, &[2], &[222])],
            final_deps: vec![(1, 0).into(), (2, 1).into(), (3, 0).into()],
            ..Default::default()
        },
        Instance {
            instance_id: Some(InstanceID::of(2, 1)),
            cmds: vec![Command::of(OpCode::NoOp, &[], &[])],
            final_deps: vec![(1, 1).into(), (2, 0).into(), (3, 0).into()],
            ..Default::default()
        },
        Instance {
            instance_id: Some((3, 1).into()),
            cmds: vec![Command::of(OpCode::Set, &[1], &[11])],
            final_deps: vec![(1, 1).into(), (2, 1).into(), (3, 0).into()],
            ..Default::default()
        },
    ];

    match rp.execute_instances(&min_insts) {
        Ok(iids) => assert_eq!(vec![InstanceID::of(1, 1), (2, 1).into()], iids),
        Err(_) => assert!(false),
    };
}
