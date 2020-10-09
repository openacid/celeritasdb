use std::sync::Arc;

use crate::inst;
use crate::instids;
use crate::instidvec;

use crate::qpaxos::Dep;
use crate::qpaxos::{Command, Instance, InstanceId};
use crate::replica::*;
use crate::testutil;
use crate::InstanceIds;
use crate::ReplicaId;
use crate::ReplicaStatus;
use crate::StorageAPI;
use std::collections::HashMap;
use storage::MemEngine;
use storage::Storage;
use tokio::sync::oneshot;

fn new_replica() -> Replica {
    testutil::new_replica(
        1,
        vec![1, 2, 3],
        vec![],
        Storage::new(Arc::new(MemEngine::new().unwrap())),
    )
}

#[test]
fn test_find_missing_instances() {
    let rp = new_replica();

    let cases1 = [
        (vec![inst!((1, 2), deps:[(1, 1)])], instidvec![(1, 1)]),
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
                inst!((1, 2), deps:(1, [1, 1, 3])),
                inst!((2, 2), deps:(1, [1, 1, 5])),
            ],
            instidvec![(1, 1), (2, 1), (3, 10)],
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
            vec![inst!((1, 2), deps:(1, [1, 6, 6]))],
            instidvec![(1, 1), (2, 5), (3, 5)],
            instidvec![(2, 6), (3, 6)],
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
                inst!((1, 2), deps:(1, [1, 1, 1])),
                inst!((2, 2), deps:(1, [1, 1, 3])),
            ],
            instidvec![(1, 1), (2, 1), (3, 1)],
            instidvec![(3, 2)],
        ),
    ];

    for (insts, up_to, exp) in case2.iter() {
        match rp.find_missing_insts(&insts, &up_to[..].into()) {
            None => assert!(false),
            Some(s) => assert_eq!(exp, &s.to_vec()),
        };
    }
}

#[tokio::test(threaded_scheduler)]
async fn test_execute_commands() {
    let rp = new_replica();
    rp.storage
        .set_kv(&"x".as_bytes().to_vec(), &vec![11])
        .unwrap();
    rp.storage
        .set_kv(&"y".as_bytes().to_vec(), &vec![22])
        .unwrap();

    let cases: &[Instance] = &[
        inst!((2, 2), []),
        inst!((2, 2), [(xx)]),
        inst!((2, 2), [(x)]),
        inst!((2, 2), [(x), (y)]),
        inst!((2, 2), [(), (y = foo), (y)]),
    ];

    for inst in cases.iter() {
        match rp.execute_commands(vec![inst.clone()], instids![]).await {
            Ok(r) => assert_eq!(vec![inst.instance_id.unwrap()], r),
            Err(_) => assert!(false),
        }
    }
}

#[tokio::test(threaded_scheduler)]
async fn test_execute_instances() {
    let rp = new_replica();

    // (3, 1)→(2, 1)→(1, 1)
    let min_insts = vec![
        inst!((1, 1), [(x = vx)], (1, [0, 0, 0])),
        inst!((2, 1), [()], (1, [1, 0, 0])),
        inst!((3, 1), [(y = vy)], (1, [1, 1, 0])),
    ];

    let executed = instids![(1, 0), (2, 0), (3, 0)];

    match rp.execute_instances(min_insts, executed).await {
        Ok(iids) => assert_eq!(vec![InstanceId::from((1, 1))], iids),
        Err(_) => assert!(false),
    };

    // (3, 1)~(2, 1)~(1, 1)
    let min_insts = vec![
        inst!((1, 1), [(x = vx)], (1, [0, 1, 0])),
        inst!((2, 1), [()], (1, [1, 0, 0])),
        inst!((3, 1), [(y = vy)], (1, [1, 0, 0])),
    ];

    let executed = instids![(1, 0), (2, 0), (3, 0)];
    match rp.execute_instances(min_insts, executed).await {
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
        inst!((1, 1), [(x = vx)], (1, [0, 1, 0])),
        inst!((2, 1), [()], (1, [1, 0, 0])),
        inst!((3, 1), [(y = vy)], (1, [1, 1, 0])),
    ];

    let executed = instids![(1, 0), (2, 0), (3, 0)];
    match rp.execute_instances(min_insts, executed).await {
        Ok(iids) => assert_eq!(instidvec![(1, 1), (2, 1)], iids),
        Err(_) => assert!(false),
    };
}

#[tokio::test(threaded_scheduler)]
async fn test_replica_execute() {
    let rp = new_replica();

    let cases: Vec<(Vec<Instance>, Vec<(ReplicaId, i64)>, Vec<InstanceId>)> = vec![
        // (1, 1)
        (
            vec![inst!((1, 1), deps:(1, [0, 0, 0]), committed:true)],
            vec![(1, 0), (2, 0), (3, 0)],
            instidvec![(1, 1)],
        ),
        // (3, 2)->(2, 2)->(1, 2)
        (
            vec![
                inst!((1, 2), deps:(1, [1, 1, 1]), committed:true),
                inst!((2, 2), deps:(1, [2, 1, 1]), committed:true),
                inst!((3, 2), deps:(1, [2, 2, 1]), committed:true),
            ],
            vec![(1, 1), (2, 1), (3, 1)],
            instidvec![(1, 2)],
        ),
        // (1, 3)~(2, 3)~(3, 3)
        (
            vec![
                inst!((1, 3), deps:(1, [2, 3, 2]), committed:true),
                inst!((2, 3), deps:(1, [3, 2, 2]), committed:true),
                inst!((3, 3), deps:(1, [3, 2, 2]), committed:true),
            ],
            vec![(1, 2), (2, 2), (3, 2)],
            instidvec![(1, 3), (2, 3), (3, 3)],
        ),
        // (1, 4)->(2, 4)~(3, 4)
        (
            vec![
                inst!((1, 4), deps:(1, [3, 4, 4]), committed:true),
                inst!((2, 4), deps:(1, [3, 3, 4]), committed:true),
                inst!((3, 4), deps:(1, [4, 4, 3]), committed:true),
            ],
            vec![(1, 3), (2, 3), (3, 3)],
            instidvec![(2, 4)],
        ),
        // (1, 5)[NotFound]<-(2, 5)~(3, 5)
        (
            vec![
                inst!((2, 5), deps:(1, [5, 4, 5]), committed:true),
                inst!((3, 5), deps:(1, [4, 5, 4]), committed:true),
            ],
            vec![(1, 4), (2, 4), (3, 4)],
            instidvec![],
        ),
    ];

    for (insts, exec_ref, rst) in cases.iter() {
        insts.iter().for_each(|inst| {
            rp.storage.set_instance(&inst).unwrap();
        });

        let mut executed = InstanceIds {
            ..Default::default()
        };
        for (rid, idx) in exec_ref.iter() {
            executed.insert(*rid, *idx);
        }
        rp.storage
            .set_status(&ReplicaStatus::Exec, &executed)
            .unwrap();

        match rp.execute().await {
            Ok(r) => {
                assert_eq!(rst, &r);
                for iid in r.iter() {
                    assert_eq!(
                        iid.idx,
                        rp.storage
                            .get_status(&ReplicaStatus::Exec)
                            .unwrap()
                            .unwrap()[&iid.replica_id]
                    );
                }
            }
            Err(_) => {
                assert!(false);
            }
        }
    }
}

#[tokio::test(threaded_scheduler)]
async fn test_send_reply() {
    let rp = new_replica();
    let (tx, rx) = oneshot::channel();
    // (1, 1) ~ (2, 1)
    let min_insts = vec![
        inst!((1, 1), [(x = vx)], (1, [0, 1, 0])),
        inst!((2, 1), [(x)], (1, [1, 0, 0])),
    ];

    rp.insert_tx((2, 1).into(), tx).await;
    let insts = min_insts.clone();
    let executed = instids![(1, 0), (2, 0)];
    tokio::spawn(async move { rp.execute_instances(insts, executed).await });

    match rx.await {
        Ok(v) => {
            println!("got = {:?}", v);
            match &v[0] {
                Some(r) => {
                    assert_eq!("vx".as_bytes().to_vec(), &r[..]);
                }
                _ => assert!(false, "invalid reply"),
            }
        }
        Err(_) => assert!(false, "the sender dropped"),
    }

    // test no panic when rx has been dropped
    let rp = new_replica();
    let (tx, rx) = oneshot::channel();
    rp.insert_tx((2, 1).into(), tx).await;
    let insts = min_insts.clone();
    let executed = instids![(1, 0), (2, 0)];
    drop(rx);
    rp.execute_instances(insts.clone(), executed).await.unwrap();
}
