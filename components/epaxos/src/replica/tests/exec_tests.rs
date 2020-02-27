use std::net::{SocketAddr, TcpListener, TcpStream};

use crate::qpaxos::Instance;
use crate::qpaxos::InstanceID;
use crate::replica::Replica;
use crate::replica::ReplicaConf;
use crate::replica::ReplicaStatus;
use crate::snapshot::MemEngine;

fn new_replica() -> Replica<MemEngine> {
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
        latest_cp: InstanceID::of(1, 1),
        engine: MemEngine::new().unwrap(),
        problem_inst_ids: vec![],
    };
}

#[test]
fn test_find_missing_instances() {
    let rp = new_replica();

    let cases1 = [
        (
            vec![Instance {
                instance_id: Some(InstanceID::of(1, 2)),
                final_deps: vec![InstanceID::of(1, 1)],
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
                    instance_id: Some(InstanceID::of(1, 2)),
                    final_deps: vec![
                        InstanceID::of(1, 1),
                        InstanceID::of(2, 1),
                        InstanceID::of(3, 3),
                    ],
                    ..Default::default()
                },
                Instance {
                    instance_id: Some(InstanceID::of(2, 2)),
                    final_deps: vec![
                        InstanceID::of(1, 1),
                        InstanceID::of(2, 1),
                        InstanceID::of(3, 5),
                    ],
                    ..Default::default()
                },
            ],
            vec![
                InstanceID::of(1, 1),
                InstanceID::of(2, 1),
                InstanceID::of(3, 10),
            ],
        ),
    ];

    for (insts, up_to) in cases1.iter() {
        match rp.find_missing_insts(&insts, &up_to) {
            None => assert!(true),
            Some(s) => assert!(false),
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
                instance_id: Some(InstanceID::of(1, 2)),
                final_deps: vec![
                    InstanceID::of(1, 1),
                    InstanceID::of(2, 6),
                    InstanceID::of(3, 6),
                ],
                ..Default::default()
            }],
            vec![
                InstanceID::of(1, 1),
                InstanceID::of(2, 5),
                InstanceID::of(3, 5),
            ],
            vec![InstanceID::of(2, 6), InstanceID::of(3, 6)],
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
                    instance_id: Some(InstanceID::of(1, 2)),
                    final_deps: vec![
                        InstanceID::of(1, 1),
                        InstanceID::of(2, 1),
                        InstanceID::of(3, 1),
                    ],
                    ..Default::default()
                },
                Instance {
                    instance_id: Some(InstanceID::of(2, 2)),
                    final_deps: vec![
                        InstanceID::of(1, 1),
                        InstanceID::of(2, 1),
                        InstanceID::of(3, 3),
                    ],
                    ..Default::default()
                },
            ],
            vec![
                InstanceID::of(1, 1),
                InstanceID::of(2, 1),
                InstanceID::of(3, 1),
            ],
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
