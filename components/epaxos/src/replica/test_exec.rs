use std::sync::Arc;

use crate::qpaxos::{Command, Instance, InstanceId};
use crate::replica::*;
use crate::snapshot::MemEngine;

#[allow(unused_macros)]
macro_rules! test_inst {
    // instance_id, final_deps
    (($rid:expr, $idx: expr),
     [$( ($fdep_rid:expr, $fdep_idx:expr) ),*]
    ) => {
        Instance {
            instance_id: Some(($rid, $idx).into()),
            final_deps: Some(
                instids![$( ($fdep_rid, $fdep_idx)),*].into()
            ),
            ..Default::default()
        }
    };

    // instance_id, cmds
    (($replica_id:expr, $idx:expr),
     [$( ($op:expr, $key:expr, $val:expr)),*]
     ) => {
        Instance {
            instance_id: Some(($replica_id, $idx).into()),
            cmds: cmds![$( ($op, $key, $val)),*].into(),
            ..Default::default()
        }
    };

    // instance_id, cmds, final_deps
    (($replica_id:expr, $idx:expr),
     [$( ($op:expr, $key:expr, $val:expr)),*],
     [$( ($fdep_rid:expr, $fdep_idx:expr) ),*]
     ) => {
        Instance {
            instance_id: Some(($replica_id, $idx).into()),
            cmds: cmds![$( ($op, $key, $val)),*].into(),
            final_deps: Some(
                instids![$( ($fdep_rid, $fdep_idx)),*].into()
            ),
            ..Default::default()
        }
    };

    // instance_id, final_deps, committed
    (($replica_id:expr, $idx:expr),
     [$( ($fdep_rid:expr, $fdep_idx:expr) ),*],
     $committed:expr
     ) => {
        Instance {
            instance_id: Some(($replica_id, $idx).into()),
            final_deps: Some(
                instids![$( ($fdep_rid, $fdep_idx)),*].into()
            ),
            committed: $committed,
            ..Default::default()
        }
    };
}

fn new_replica() -> Replica {
    test_util::new_replica(
        1,
        vec![1, 2, 3],
        vec![],
        Arc::new(MemEngine::new().unwrap()),
    )
}

#[test]
fn test_find_missing_instances() {
    let rp = new_replica();

    let cases1 = [
        (
            vec![test_inst!((1, 2), [(1, 1)])],
            vec![InstanceId::from((1, 1))],
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
                test_inst!((1, 2), [(1, 1), (2, 1), (3, 3)]),
                test_inst!((2, 2), [(1, 1), (2, 1), (3, 5)]),
            ],
            vec![InstanceId::from((1, 1)), (2, 1).into(), (3, 10).into()],
        ),
    ];

    for (insts, up_to) in cases1.iter() {
        match rp.find_missing_insts(&insts, &up_to[..].into()) {
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
            vec![test_inst!((1, 2), [(1, 1), (2, 6), (3, 6)])],
            vec![InstanceId::from((1, 1)), (2, 5).into(), (3, 5).into()],
            vec![InstanceId::from((2, 6)), (3, 6).into()],
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
                test_inst!((1, 2), [(1, 1), (2, 1), (3, 1)]),
                test_inst!((2, 2), [(1, 1), (2, 1), (3, 3)]),
            ],
            vec![InstanceId::from((1, 1)), (2, 1).into(), (3, 1).into()],
            vec![InstanceId::from((3, 2))],
        ),
    ];

    for (insts, up_to, exp) in case2.iter() {
        match rp.find_missing_insts(&insts, &up_to[..].into()) {
            None => assert!(false),
            Some(s) => assert_eq!(exp, &s.to_vec()),
        };
    }
}

#[test]
fn test_execute_commands() {
    let rp = new_replica();
    rp.storage
        .set_kv("x".as_bytes().to_vec(), vec![11])
        .unwrap();
    rp.storage
        .set_kv("y".as_bytes().to_vec(), vec![22])
        .unwrap();

    let cases = [
        (test_inst!((2, 2), []), Vec::<ExecuteResult>::new()),
        (
            test_inst!((2, 2), [("Get", "xx", "")]),
            vec![ExecuteResult::NotFound],
        ),
        (
            test_inst!((2, 2), [("Get", "x", "")]),
            vec![ExecuteResult::SuccessWithVal { value: vec![11] }],
        ),
        (
            test_inst!((2, 2), [("Get", "x", ""), ("Get", "y", "")]),
            vec![
                ExecuteResult::SuccessWithVal { value: vec![11] },
                ExecuteResult::SuccessWithVal { value: vec![22] },
            ],
        ),
        (
            test_inst!(
                (2, 2),
                [("NoOp", "", ""), ("Set", "y", "foo"), ("Get", "y", "")]
            ),
            vec![
                ExecuteResult::Success,
                ExecuteResult::Success,
                ExecuteResult::SuccessWithVal {
                    value: "foo".as_bytes().to_vec(),
                },
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
    let rp = new_replica();

    // (3, 1)→(2, 1)→(1, 1)
    let min_insts = vec![
        test_inst!((1, 1), [("Set", "x", "vx")], [(1, 0), (2, 0), (3, 0)]),
        test_inst!((2, 1), [("NoOp", "", "")], [(1, 1), (2, 0), (3, 0)]),
        test_inst!((3, 1), [("Set", "y", "vy")], [(1, 1), (2, 1), (3, 0)]),
    ];

    match rp.execute_instances(&min_insts) {
        Ok(iids) => assert_eq!(vec![InstanceId::from((1, 1))], iids),
        Err(_) => assert!(false),
    };

    // (3, 1)~(2, 1)~(1, 1)
    let min_insts = vec![
        test_inst!((1, 1), [("Set", "x", "vx")], [(1, 0), (2, 1), (3, 0)]),
        test_inst!((2, 1), [("NoOp", "", "")], [(1, 1), (2, 0), (3, 0)]),
        test_inst!((3, 1), [("Set", "y", "vy")], [(1, 1), (2, 0), (3, 0)]),
    ];

    match rp.execute_instances(&min_insts) {
        Ok(iids) => assert_eq!(
            vec![InstanceId::from((1, 1)), (2, 1).into(), (3, 1).into()],
            iids
        ),
        Err(_) => assert!(false),
    };

    // (3, 1) →(1, 1)
    //       ↘   ~
    //         (2, 1)
    let min_insts = vec![
        test_inst!((1, 1), [("Set", "x", "vx")], [(1, 0), (2, 1), (3, 0)]),
        test_inst!((2, 1), [("NoOp", "", "")], [(1, 1), (2, 0), (3, 0)]),
        test_inst!((3, 1), [("Set", "y", "vy")], [(1, 1), (2, 1), (3, 0)]),
    ];

    match rp.execute_instances(&min_insts) {
        Ok(iids) => assert_eq!(vec![InstanceId::from((1, 1)), (2, 1).into()], iids),
        Err(_) => assert!(false),
    };
}

#[test]
fn test_replica_execute() {
    let rp = new_replica();

    let cases = vec![
        // (1, 1)
        (
            vec![test_inst!((1, 1), [(1, 0), (2, 0), (3, 0)], true)],
            vec![(1, 1), (2, 0), (3, 0)],
            vec![(1, 0), (2, 0), (3, 0)],
            vec![InstanceId::from((1, 1))],
        ),
        // (3, 2)->(2, 2)->(1, 2)
        (
            vec![
                test_inst!((1, 2), [(1, 1), (2, 1), (3, 1)], true),
                test_inst!((2, 2), [(1, 2), (2, 1), (3, 1)], true),
                test_inst!((3, 2), [(1, 2), (2, 2), (3, 1)], true),
            ],
            vec![(1, 2), (2, 2), (3, 2)],
            vec![(1, 1), (2, 1), (3, 1)],
            vec![InstanceId::from((1, 2))],
        ),
        // (1, 3)~(2, 3)~(3, 3)
        (
            vec![
                test_inst!((1, 3), [(1, 2), (2, 3), (3, 2)], true),
                test_inst!((2, 3), [(1, 3), (2, 2), (3, 2)], true),
                test_inst!((3, 3), [(1, 3), (2, 2), (3, 2)], true),
            ],
            vec![(1, 3), (2, 3), (3, 3)],
            vec![(1, 2), (2, 2), (3, 2)],
            vec![InstanceId::from((1, 3)), (2, 3).into(), (3, 3).into()],
        ),
        // (1, 4)->(2, 4)~(3, 4)
        (
            vec![
                test_inst!((1, 4), [(1, 3), (2, 4), (3, 4)], true),
                test_inst!((2, 4), [(1, 3), (2, 3), (3, 4)], true),
                test_inst!((3, 4), [(1, 4), (2, 4), (3, 3)], true),
            ],
            vec![(1, 4), (2, 4), (3, 4)],
            vec![(1, 3), (2, 3), (3, 3)],
            vec![InstanceId::from((2, 4))],
        ),
        // (1, 5)[NotFound]<-(2, 5)~(3, 5)
        (
            vec![
                test_inst!((2, 5), [(1, 5), (2, 4), (3, 5)], true),
                test_inst!((3, 5), [(1, 4), (2, 5), (3, 4)], true),
            ],
            vec![(1, 4), (2, 5), (3, 5)],
            vec![(1, 4), (2, 4), (3, 4)],
            Vec::<InstanceId>::new(),
        ),
    ];

    for (insts, max_ref, exec_ref, rst) in cases.iter() {
        insts.iter().for_each(|inst| {
            rp.storage.set_instance(&inst).unwrap();
        });

        for (rid, idx) in max_ref.iter() {
            rp.storage
                .set_ref("max", *rid as i64, (*rid as i64, *idx as i64).into())
                .unwrap();
        }
        for (rid, idx) in exec_ref.iter() {
            rp.storage
                .set_ref("exec", *rid as i64, (*rid as i64, *idx as i64).into())
                .unwrap();
        }

        match rp.execute() {
            Ok(r) => {
                assert_eq!(rst, &r);
                for iid in r.iter() {
                    assert_eq!(*iid, rp.storage.get_ref("exec", iid.replica_id).unwrap());
                    assert_eq!(
                        true,
                        rp.storage.get_instance(*iid).unwrap().unwrap().executed
                    );
                }
            }
            Err(_) => {
                assert!(false);
            }
        }
    }
}
