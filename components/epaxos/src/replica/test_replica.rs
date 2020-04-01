use std::sync::Arc;

use crate::qpaxos::*;
use crate::replica::*;
use crate::snapshot::MemEngine;
use crate::snapshot::Storage;

use pretty_assertions::assert_eq;

/// Create an instance with command "set x=y".
/// Use this when only deps are concerned.
/// The initial_deps and deps are all set to the second arg.
/// supported pattern:
/// foo_inst!(iid, cmds, initial_deps)
/// foo_inst!(iid, key, initial_deps) // an instance with a single cmd: Set `key`
/// foo_inst!(iid, initial_deps)
/// foo_inst!(None, initial_deps)
/// foo_inst!(iid)
macro_rules! foo_inst {
    ($id:expr,
     [$( ($op:expr, $key:expr, $val:expr)),*],
     [$(($dep_rid:expr, $dep_idx:expr)),* $(,)*]
    ) => {
        inst!($id, (0, 0, _),
              [$( ($op, $key, $val)),*],
              [$(($dep_rid, $dep_idx)),*],
              "withdeps"
        )
    };

    ($id:expr,
     $key:expr,
     [$(($dep_rid:expr, $dep_idx:expr)),* $(,)*]
    ) => {
        inst!($id, (0, 0, _),
              [("Set", $key, $key)],
              [$(($dep_rid, $dep_idx)),*],
              "withdeps"
        )
    };

    (None,
     [$(($dep_rid:expr, $dep_idx:expr)),* $(,)*]
    ) => {
        Instance {
            instance_id: None,
            ..inst!((0, 0), (0, 0, _),
                      [("Set", "x", "y")],
                      [$(($dep_rid, $dep_idx)),*],
                      "withdeps"
                     )
        }
    };

    ($id:expr,
     [$(($dep_rid:expr, $dep_idx:expr)),* $(,)*]
    ) => {
        inst!($id, (0, 0, _),
              [("Set", "x", "y")],
              [$(($dep_rid, $dep_idx)),*],
              "withdeps"
        )
    };

    ($id:expr
    ) => {
        inst!($id, (0, 0, _),
              [("Set", "x", "y")],
        )
    };
}

fn new_foo_inst(leader_id: i64) -> Instance {
    let mut ii = inst!(
        (leader_id, 1),
        (2, 2, _),
        [("NoOp", "k1", "v1"), ("Get", "k2", "v2")],
        [(1, 10), (2, 20), (3, 30)],
        [(2, 20)]
    );
    ii.last_ballot = Some((1, 2, leader_id).into());
    ii.final_deps = Some(instids![(3, 30)].into());

    ii
}

fn new_mem_sto() -> Storage {
    Arc::new(MemEngine::new().unwrap())
}

/// Create a stupid replica with some instances stored.
fn new_foo_replica(
    replica_id: i64,
    storage: Storage,
    insts: &[((i64, i64), &Instance)],
) -> Replica {
    let r = Replica {
        replica_id,
        group_replica_ids: vec![0, 1, 2],
        peers: vec![],
        conf: ReplicaConf {
            ..Default::default()
        },
        storage,
    };

    for (iid, inst) in insts.iter() {
        r.storage.set_obj((*iid).into(), inst).unwrap();
    }

    r
}

macro_rules! test_invalid_req {
    ($replica:expr, $req_t:ident, $handle:path, $cases: expr) => {
        for (cmn, etuple) in $cases.clone() {
            let req = $req_t {
                cmn,
                ..Default::default()
            };
            let repl = $handle($replica, &req);
            let err = repl.err.unwrap();
            assert_eq!(
                QError {
                    req: Some(etuple.into()),
                    ..Default::default()
                },
                err
            );
        }
    };
}

#[test]
fn test_new_instance() {
    let rid1 = 1;
    let rid2 = 2;

    let cmds = cmds![("Set", "x", "1")];
    let sto = new_mem_sto();

    let r1 = new_foo_replica(rid1, sto.clone(), &[]);
    let r2 = new_foo_replica(rid2, sto.clone(), &[]);

    // (1, 0) -> []
    let i10 = r1.new_instance(cmds.clone()).unwrap();
    assert_eq!(i10, init_inst!((rid1, 0), [("Set", "x", "1")], []));

    // (2, 0) -> [(1, 0)]
    let i20 = r2.new_instance(cmds.clone()).unwrap();
    assert_eq!(i20, init_inst!((rid2, 0), [("Set", "x", "1")], [(rid1, 0)]));

    // (2, 1) -> [(1, 0), (2, 0)]
    assert_eq!(
        r2.new_instance(cmds.clone()).unwrap(),
        init_inst!((rid2, 1), [("Set", "x", "1")], [(rid1, 0), (rid2, 0)])
    );
}

#[test]
fn test_quorums() {
    let cases: Vec<(i32, i32, i32)> = vec![
        (0, 1, 0),
        (1, 1, 0),
        (2, 2, 1),
        (3, 2, 2),
        (4, 3, 2),
        (5, 3, 3),
        (6, 4, 4),
        (7, 4, 5),
        (8, 5, 5),
        (9, 5, 6),
    ];

    for (n_replicas, q, fastq) in cases {
        let mut r = new_foo_replica(1, new_mem_sto(), &[]);
        r.group_replica_ids = vec![];

        for i in 0..n_replicas {
            r.group_replica_ids.push(i as ReplicaID);
        }

        assert_eq!(q, quorum(n_replicas), "quorum n={}", n_replicas);
        assert_eq!(
            fastq,
            fast_quorum(n_replicas),
            "fast-quorum n={}",
            n_replicas
        );
    }
}

#[test]
fn test_handle_xxx_request_invalid() {
    let replica_id = 2;
    let mut replica = new_foo_replica(replica_id, new_mem_sto(), &vec![]);

    let cases: Vec<(Option<RequestCommon>, (&str, &str, &str))> = vec![
        (None, ("cmn", "LackOf", "")),
        (
            Some(RequestCommon {
                to_replica_id: 0,
                ballot: None,
                instance_id: None,
            }),
            ("cmn.to_replica_id", "NotFound", "0; my replica_id: 2"),
        ),
        (
            Some(RequestCommon {
                to_replica_id: replica_id,
                ballot: None,
                instance_id: None,
            }),
            ("cmn.ballot", "LackOf", ""),
        ),
        (
            Some(RequestCommon {
                to_replica_id: replica_id,
                ballot: Some((0, 0, 1).into()),
                instance_id: None,
            }),
            ("cmn.instance_id", "LackOf", ""),
        ),
    ];

    test_invalid_req!(&mut replica, AcceptRequest, Replica::handle_accept, cases);
    test_invalid_req!(&mut replica, CommitRequest, Replica::handle_commit, cases);
}

#[test]
#[should_panic(expected = "local_inst.deps is unexpected to be None")]
fn test_handle_fast_accept_request_panic_local_deps_none() {
    let inst = foo_inst!((0, 0));
    let req_inst = foo_inst!((1, 0), [(0, 0)]);

    _handle_fast_accept_request((0, 0), inst, req_inst);
}

#[test]
#[should_panic(expected = "local_inst.instance_id is unexpected to be None")]
fn test_handle_fast_accept_request_panic_local_instance_id_none() {
    let inst = foo_inst!(None, [(2, 0)]);
    let req_inst = foo_inst!((1, 0), [(0, 0)]);

    _handle_fast_accept_request((0, 0), inst, req_inst);
}

fn _handle_fast_accept_request(iid: (i64, i64), inst: Instance, req_inst: Instance) {
    let replica = new_foo_replica(1, new_mem_sto(), &[(iid, &inst)]);

    let req = MakeRequest::fast_accept(1, &req_inst, &vec![false]);
    replica.handle_fast_accept(&req);
}

#[test]
fn test_handle_fast_accept_request() {
    let replica_id = 1;
    let replica = new_foo_replica(replica_id, new_mem_sto(), &vec![]);

    {
        let mut inst = new_foo_inst(replica_id);
        let iid = inst.instance_id.unwrap();
        let blt = inst.ballot;

        let none = replica.storage.get_instance(iid).unwrap();
        assert_eq!(None, none);

        let deps_committed = vec![false, false, false];
        let req = MakeRequest::fast_accept(replica_id, &inst, &deps_committed);
        let repl = replica.handle_fast_accept(&req);

        inst.deps = inst.initial_deps.clone();

        assert_eq!(None, repl.err);
        assert_eq!(deps_committed, repl.deps_committed);
        assert_eq!(inst.deps, repl.deps);

        _test_repl_cmn_ok(&repl.cmn.unwrap(), iid, blt);

        // get the written instance.
        _test_get_inst(&replica, iid, blt, blt, inst.cmds, None, false, false);
    }

    {
        // instance space layout, then a is replicated to R1
        //               .c
        //             /  |
        // d          d   |
        // |          |\ /
        // a          a-b            c
        // x y z      x y z      x y z
        // -----      -----      -----
        // R0         R1         R2

        // below code that new instance is encapsulated in a func

        let x_iid = instid!(0, 0);
        let instx = foo_inst!(x_iid, "key_x", [(0, 0), (0, 0), (0, 0)]);

        let y_iid = instid!(1, 0);
        let insty = foo_inst!(y_iid, "key_y", [(0, 0), (0, 0), (0, 0)]);

        let z_iid = instid!(2, 0);
        let instz = foo_inst!(z_iid, "key_z", [(0, 0), (0, 0), (0, 0)]);

        // instb -> {x, y, z} committed
        let b_iid = instid!(1, 1);
        let mut instb = foo_inst!(b_iid, "key_b", [(0, 0), (1, 0), (2, 0)]);
        instb.committed = true;

        // insta -> {x, y, z}
        let a_iid = instid!(0, 1);
        let mut insta = foo_inst!(a_iid, "key_a", [(0, 0), (1, 0), (2, 0)]);

        // instd -> {a, b, z}
        let d_iid = instid!(0, 2);
        let instd = foo_inst!(d_iid, "key_d", [(0, 1), (1, 1), (2, 0)]);

        // instc -> {d, b, z}
        let c_iid = instid!(2, 3);
        let instc = foo_inst!(c_iid, "key_z", [(0, 2), (1, 1), (2, 0)]);

        let replica = new_foo_replica(
            replica_id,
            new_mem_sto(),
            &vec![
                ((0, 0), &instx),
                ((0, 2), &instd),
                ((1, 0), &insty),
                ((1, 1), &instb),
                ((2, 0), &instz),
                ((2, 3), &instc),
            ],
        );

        let deps_committed = vec![false, true, false];
        let req = MakeRequest::fast_accept(replica_id, &insta, &deps_committed);
        let repl = replica.handle_fast_accept(&req);

        insta.deps = Some(vec![x_iid, b_iid, z_iid].into());

        assert_eq!(None, repl.err);
        assert_eq!(deps_committed, repl.deps_committed);
        assert_eq!(insta.deps, repl.deps);

        _test_repl_cmn_ok(&repl.cmn.unwrap(), insta.instance_id.unwrap(), insta.ballot);

        // get the written instance.
        _test_get_inst(
            &replica,
            insta.instance_id.unwrap(),
            insta.ballot,
            insta.ballot,
            insta.cmds,
            None,
            false,
            false,
        );
    }
}

#[test]
fn test_handle_accept_request() {
    let replica_id = 2;
    let inst = new_foo_inst(replica_id);
    let iid = inst.instance_id.unwrap();
    let blt = inst.ballot;
    let fdeps = inst.final_deps.clone();

    let replica = new_foo_replica(replica_id, new_mem_sto(), &vec![]);
    let none = replica.storage.get_instance(iid).unwrap();
    assert_eq!(None, none);

    {
        // ok reply with none instance.
        let req = MakeRequest::accept(replica_id, &inst);
        let repl = replica.handle_accept(&req);
        assert_eq!(None, repl.err);
        _test_repl_cmn_ok(&repl.cmn.unwrap(), iid, None);

        // get the written instance.
        _test_get_inst(
            &replica,
            iid,
            blt,
            None,
            vec![],
            fdeps.clone(),
            false,
            false,
        );
    }

    {
        // ok reply when replacing instance. same ballot.
        let req = MakeRequest::accept(replica_id, &inst);
        assert_eq!(
            req.cmn.clone().unwrap().ballot,
            replica.storage.get_instance(iid).unwrap().unwrap().ballot
        );

        let repl = replica.handle_accept(&req);
        assert_eq!(None, repl.err);
        _test_repl_cmn_ok(&repl.cmn.unwrap(), iid, blt);

        // get the accepted instance.
        _test_get_inst(&replica, iid, blt, blt, vec![], fdeps.clone(), false, false);
    }

    {
        // ok reply but not written because of a higher ballot.
        let req = MakeRequest::accept(replica_id, &inst);

        // make an instance with bigger ballot.
        let mut curr = replica.storage.get_instance(iid).unwrap().unwrap();
        let mut bigger = blt.unwrap();
        bigger.num += 1;
        let bigger = Some(bigger);

        curr.ballot = bigger;
        curr.final_deps = Some(vec![].into());
        replica.storage.set_instance(&curr).unwrap();

        let curr = replica.storage.get_instance(iid).unwrap().unwrap();
        assert!(curr.ballot > blt);

        // accept wont update this instance.
        let repl = replica.handle_accept(&req);
        assert_eq!(None, repl.err);
        _test_repl_cmn_ok(&repl.cmn.unwrap(), iid, bigger);

        // get the intact instance.
        _test_get_inst(
            &replica,
            iid,
            bigger,
            blt,
            vec![],
            Some(vec![].into()),
            false,
            false,
        );
    }

    // TODO test storage error
}

#[test]
fn test_handle_commit_request() {
    let replica_id = 2;
    let inst = new_foo_inst(replica_id);
    let iid = inst.instance_id.unwrap();
    let blt = inst.ballot;
    let cmds = inst.cmds.clone();
    let fdeps = inst.final_deps.clone();

    let replica = new_foo_replica(replica_id, new_mem_sto(), &vec![]);
    let none = replica.storage.get_instance(iid).unwrap();
    assert_eq!(None, none);

    let req = MakeRequest::commit(replica_id, &inst);

    {
        // ok reply with none instance.
        let repl = replica.handle_commit(&req);
        assert_eq!(None, repl.err);
        _test_repl_cmn_ok(&repl.cmn.unwrap(), iid, None);

        // get the committed instance.
        _test_get_inst(
            &replica,
            iid,
            blt,
            None,
            cmds.clone(),
            fdeps.clone(),
            true,
            false,
        );
    }

    {
        // ok reply when replacing instance.
        let repl = replica.handle_commit(&req);
        assert_eq!(None, repl.err);
        _test_repl_cmn_ok(&repl.cmn.unwrap(), iid, blt);

        // get the committed instance.
        _test_get_inst(
            &replica,
            iid,
            blt,
            blt,
            cmds.clone(),
            fdeps.clone(),
            true,
            false,
        );
    }

    // TODO test storage error
}

#[tokio::main]
async fn _bcast_fast_accept() {
    let mut tc = test_util::TestCluster::new(3);
    tc.start().await;
    let inst = foo_inst!((0, 1), "key_x", [(0, 0), (1, 0), (2, 0)]);
    let r = bcast_fast_accept(&tc.replicas[0].peers, &inst, &[true, true, true])
        .await
        .unwrap();

    println!("receive fast accept replys: {:?}", r);
    // not contain self
    assert_eq!(2, r.len());
}

#[test]
fn test_bcast_fast_accept() {
    _bcast_fast_accept();
}

#[tokio::main]
async fn _bcast_accept() {
    let mut tc = test_util::TestCluster::new(3);
    tc.start().await;
    let inst = foo_inst!((0, 1), "key_x", [(0, 0), (1, 0), (2, 0)]);
    let r = bcast_accept(&tc.replicas[0].peers, &inst).await.unwrap();

    println!("receive accept replys: {:?}", r);
    // not contain self
    assert_eq!(2, r.len());
}

#[test]
fn test_bcast_accept() {
    _bcast_accept();
}

#[tokio::main]
async fn _bcast_commit() {
    let mut tc = test_util::TestCluster::new(3);
    tc.start().await;
    let inst = foo_inst!((0, 1), "key_x", [(0, 0), (1, 0), (2, 0)]);
    let r = bcast_commit(&tc.replicas[0].peers, &inst).await.unwrap();

    println!("receive commit replys: {:?}", r);
    // not contain self
    assert_eq!(2, r.len());
}

#[test]
fn test_bcast_commit() {
    _bcast_commit();
}

#[tokio::main]
async fn _bcast_prepare() {
    let mut tc = test_util::TestCluster::new(3);
    tc.start().await;
    let inst = foo_inst!((0, 1), "key_x", [(0, 0), (1, 0), (2, 0)]);
    let r = bcast_prepare(&tc.replicas[0].peers, &inst).await.unwrap();

    println!("receive prepare replys: {:?}", r);
    // not contain self
    assert_eq!(2, r.len());
}

#[test]
fn test_bcast_prepare() {
    _bcast_prepare();
}

fn _test_repl_cmn_ok(cmn: &ReplyCommon, iid: InstanceId, last: Option<BallotNum>) {
    assert_eq!(iid, cmn.instance_id.unwrap());
    assert_eq!(last, cmn.last_ballot);
}

fn _test_get_inst(
    replica: &Replica,
    iid: InstanceId,
    blt: Option<BallotNum>,
    last: Option<BallotNum>,
    cmds: Vec<Command>,
    final_deps: Option<InstanceIdVec>,
    committed: bool,
    executed: bool,
) {
    let got = replica.storage.get_instance(iid).unwrap().unwrap();
    assert_eq!(iid, got.instance_id.unwrap());
    assert_eq!(blt, got.ballot);
    assert_eq!(last, got.last_ballot);
    assert_eq!(cmds, got.cmds);
    assert_eq!(final_deps, got.final_deps);
    assert_eq!(committed, got.committed);
    assert_eq!(executed, got.executed);
}
