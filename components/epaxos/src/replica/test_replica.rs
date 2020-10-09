use std::sync::Arc;

use crate::inst;

use crate::conf::ClusterInfo;
use crate::qpaxos::*;
use crate::replica::*;
use crate::testutil;
use crate::StorageAPI;
use storage::ToKey;
use storage::{DBColumnFamily, Storage};
use storage::{MemEngine, RawKV};

use pretty_assertions::assert_eq;
use prost::Message;

/// Create an instance with command "set x=y".
/// Use this when only deps are concerned.
/// The deps and deps are all set to the second arg.
/// supported pattern:
/// foo_inst!(iid, key, deps) // an instance with a single cmd: Set `key`
/// foo_inst!(iid, deps)
/// foo_inst!(None, deps)
#[allow(unused_macros)]
macro_rules! foo_inst {

    ($id:expr,
     $key:expr,
     [$(($dep_rid:expr, $dep_idx:expr)),* $(,)*]
    ) => {
        inst!($id, (0, _),
              [("Set", $key, $key)],
              [$(($dep_rid, $dep_idx)),*]
        )
    };

    (None,
     [$(($dep_rid:expr, $dep_idx:expr)),* $(,)*]
    ) => {
        Instance {
            instance_id: None,
            ..inst!((0, 0), (0, _),
                      [(x=y)],
                      [$(($dep_rid, $dep_idx)),*]
                     )
        }
    };

    ($id:expr,
     [$(($dep_rid:expr, $dep_idx:expr)),* $(,)*]
    ) => {
        inst!($id, (0, _),
              [(x=y)],
              [$(($dep_rid, $dep_idx)),*]
        )
    };
}

fn new_foo_inst(leader_id: i64) -> Instance {
    inst!(
        (leader_id, 1),
        (2, _),
        [(), (k2)],
        [(1, 10), (2, 20), (3, 30)],
    )
}

fn new_mem_sto() -> Storage {
    Storage::new(Arc::new(MemEngine::new().unwrap()))
}

/// Create a stupid replica with some instances stored.
fn new_foo_replica(
    replica_id: i64,
    storage: Storage,
    insts: &[((i64, i64), &Instance)],
) -> Replica {
    let r = testutil::new_replica(replica_id, vec![0, 1, 2], vec![], storage);

    for (iid, inst) in insts.iter() {
        let mut value = vec![];
        inst.encode(&mut value).unwrap();

        let iid = InstanceId::from(iid);
        r.storage
            .set_raw(DBColumnFamily::Instance, &iid.to_key(), &value)
            .unwrap();
    }

    r
}

#[test]
fn test_new_instance() {
    let rid1 = 1;
    let rid2 = 2;

    let cmds = cmdvec![("Set", "x", "1")];
    let sto = new_mem_sto();

    let r1 = new_foo_replica(rid1, sto.clone(), &[]);
    let r2 = new_foo_replica(rid2, sto.clone(), &[]);

    // (1, 0) -> []
    let i10 = r1.new_instance(&cmds).unwrap();
    assert_eq!(
        i10,
        inst!((rid1, 0), (0, _), [(x = "1")], (0, [-1, -1, -1]))
    );
    assert_eq!(
        i10,
        r1.storage.get_instance((rid1, 0).into()).unwrap().unwrap()
    );

    // (2, 0) -> [(1, 0)]
    let i20 = r2.new_instance(&cmds).unwrap();
    assert_eq!(i20, inst!((rid2, 0), (0, _), [(x = "1")], (0, [-1, 0, -1])));
    assert_eq!(
        i20,
        r1.storage.get_instance((rid2, 0).into()).unwrap().unwrap()
    );

    // (2, 1) -> [(1, 0), (2, 0)]
    let i21 = r2.new_instance(&cmds).unwrap();
    assert_eq!(i21, inst!((rid2, 1), (0, _), [(x = "1")], (0, [-1, 0, 0])));
    assert_eq!(
        i21,
        r1.storage.get_instance((rid2, 1).into()).unwrap().unwrap()
    );
}

#[test]
fn test_get_max_instance_ids() {
    let (i12, i13, i34) = (inst!((1, 2)), inst!((1, 3)), inst!((3, 4)));

    let insts = vec![((1, 2), &i12), ((1, 3), &i13), ((3, 4), &i34)];

    let r = new_foo_replica(3, new_mem_sto(), &insts);
    let maxs = r.get_max_instance_ids(&[1, 3, 5]);
    assert_eq!(
        maxs,
        InstanceIdVec::from(instidvec![(1, 3), (3, 4), (5, -1)])
    );
}

#[test]
fn test_handle_replicate_request_invalid() {
    let replica_id = 2;
    let replica = new_foo_replica(replica_id, new_mem_sto(), &vec![]);

    let cases: Vec<(ReplicateRequest, &str)> = vec![
        (
            ReplicateRequest {
                to_replica_id: replica_id,
                ballot: None,
                instance_id: None,
                ..Default::default()
            },
            "ballot",
        ),
        (
            ReplicateRequest {
                to_replica_id: replica_id,
                ballot: Some((0, 1).into()),
                instance_id: None,
                ..Default::default()
            },
            "instance_id",
        ),
        (
            ReplicateRequest {
                to_replica_id: replica_id,
                ballot: Some((0, 1).into()),
                instance_id: Some((1, 2).into()),
                ..Default::default()
            },
            "phase",
        ),
    ];

    for (req, estr) in cases.clone() {
        let repl = replica.handle_replicate(req);
        let err = repl.err().unwrap();
        assert_eq!(err, ProtocolError::LackOf(estr.into()).into());
    }
}

#[test]
fn test_handle_replicate_ballot_check() {
    let replica_id = 2;
    let replica = new_foo_replica(replica_id, new_mem_sto(), &vec![]);

    let local_inst = inst!((3, 4), (2, _), [("Set", "x", "0")]);
    replica.storage.set_instance(&local_inst).unwrap();

    let inst = inst!((3, 4), (1, _), [("Set", "x", "1")],);

    let reqs = vec![
        MakeRequest::prepare(0, &inst, &[]),
        MakeRequest::accept(0, &inst),
    ];

    for req in reqs {
        let repl = replica.handle_replicate(req);
        assert!(repl.is_ok());

        let repl = repl.unwrap();
        assert!(repl.err.is_none());
        assert_eq!(repl.last_ballot.unwrap(), BallotNum::from((2, 3)));
        assert_eq!(repl.instance_id.unwrap(), InstanceId::from((3, 4)));

        let notupdated = replica.get_instance((3, 4).into()).unwrap();
        assert_eq!(local_inst, notupdated, "not updated");
    }

    {
        // commit does not check ballot
        let req = MakeRequest::commit(0, &inst);

        let repl = replica.handle_replicate(req);
        assert!(repl.is_ok());

        let repl = repl.unwrap();
        assert!(repl.err.is_none());

        let updated = replica.get_instance((3, 4).into()).unwrap();
        assert!(updated.committed);
    }
}

#[test]
#[should_panic(expected = "inst.deps is unexpected to be None")]
fn test_handle_prepare_request_panic_local_deps_none() {
    let inst = inst!((0, 0));
    let req_inst = foo_inst!((1, 0), [(0, 0)]);

    _handle_prepare_request((0, 0), inst, req_inst);
}

#[test]
#[should_panic(expected = "inst.instance_id is unexpected to be None")]
fn test_handle_prepare_request_panic_local_instance_id_none() {
    let inst = foo_inst!(None, [(2, 0)]);
    let req_inst = foo_inst!((1, 0), [(0, 0)]);

    _handle_prepare_request((0, 0), inst, req_inst);
}

fn _handle_prepare_request(iid: (i64, i64), mut inst: Instance, req_inst: Instance) {
    let replica = new_foo_replica(1, new_mem_sto(), &[(iid, &inst)]);

    let req = MakeRequest::prepare(1, &req_inst, &vec![false]);
    let req: PrepareRequest = req.phase.unwrap().try_into().unwrap();
    let _ = replica.handle_prepare(&req, &mut inst);
}

#[test]
fn test_handle_prepare_normal() {
    let replica_id = 1;
    let replica = new_foo_replica(replica_id, new_mem_sto(), &vec![]);

    {
        let inst = new_foo_inst(replica_id);
        let iid = inst.instance_id.unwrap();
        let _blt = inst.ballot;

        let deps_committed = vec![false, false, false];
        let req = MakeRequest::prepare(replica_id, &inst, &deps_committed);
        let req: PrepareRequest = req.phase.unwrap().try_into().unwrap();
        let mut local_inst = Instance {
            instance_id: Some(iid),
            ..Default::default()
        };
        let repl = replica.handle_prepare(&req, &mut local_inst);
        let repl = repl.unwrap();

        // assert_eq!(None, repl.err);
        assert_eq!(deps_committed, repl.deps_committed);
        assert_eq!(inst.deps, repl.deps);

        assert_eq!(inst.deps, local_inst.deps);
        _test_updated_inst(&local_inst, inst.cmds, false, false);
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
        let insta = foo_inst!(a_iid, "key_a", [(0, 0), (1, 0), (2, 0)]);

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
        let req = MakeRequest::prepare(replica_id, &insta, &deps_committed);
        let req: PrepareRequest = req.phase.unwrap().try_into().unwrap();

        let mut local_inst = Instance {
            instance_id: Some(a_iid),
            ..Default::default()
        };

        let repl = replica.handle_prepare(&req, &mut local_inst);
        let repl = repl.unwrap();

        let wantdeps: Deps = vec![x_iid, b_iid, z_iid].into();
        let wantdeps = Some(wantdeps);

        // TODO test updated deps_committed

        assert_eq!(deps_committed, repl.deps_committed);
        assert_eq!(wantdeps.clone(), repl.deps);

        assert_eq!(wantdeps.clone(), local_inst.deps);

        _test_updated_inst(&local_inst, insta.cmds, false, false);
    }
}

#[test]
fn test_handle_accept_request() {
    let replica_id = 2;
    let inst = new_foo_inst(replica_id);
    let iid = inst.instance_id.unwrap();
    let blt = inst.ballot;
    let fdeps = inst.deps.clone();

    let replica = new_foo_replica(replica_id, new_mem_sto(), &vec![]);
    let none = replica.storage.get_instance(iid).unwrap();
    assert_eq!(None, none);

    {
        // ok reply with none instance.
        let req = MakeRequest::accept(replica_id, &inst);
        let req: AcceptRequest = req.phase.unwrap().try_into().unwrap();

        let mut local_inst = Instance {
            instance_id: Some(iid),
            ..Default::default()
        };

        let repl = replica.handle_accept(&req, &mut local_inst);
        assert!(repl.is_ok());

        println!("inst:{}", inst);
        println!("local_inst:{}", local_inst);

        assert_eq!(fdeps, local_inst.deps, "deps");
        _test_updated_inst(&local_inst, vec![], false, false);
    }

    {
        // ok reply when replacing instance. same ballot.
        let req = MakeRequest::accept(replica_id, &inst);
        let req: AcceptRequest = req.phase.unwrap().try_into().unwrap();

        let mut local_inst = Instance {
            instance_id: Some(iid),
            ballot: blt,
            ..Default::default()
        };

        let repl = replica.handle_accept(&req, &mut local_inst);
        assert!(repl.is_ok());

        assert_eq!(fdeps, local_inst.deps, "deps");
        _test_updated_inst(&local_inst, vec![], false, false);
    }

    // TODO test higher ballot

    // TODO test storage error
}

#[test]
fn test_handle_commit_request() {
    let replica_id = 2;
    let inst = new_foo_inst(replica_id);
    let iid = inst.instance_id.unwrap();
    let cmds = inst.cmds.clone();
    let fdeps = inst.deps.clone();

    let replica = new_foo_replica(replica_id, new_mem_sto(), &vec![]);

    let req = MakeRequest::commit(replica_id, &inst);
    let req: CommitRequest = req.phase.unwrap().try_into().unwrap();

    // ok reply when replacing instance.
    let mut inst = replica.get_instance(iid).unwrap();
    let repl = replica.handle_commit(&req, &mut inst);
    assert!(repl.is_ok());

    assert_eq!(fdeps, inst.deps, "deps");
    _test_updated_inst(&inst, cmds.clone(), true, false);
}

fn _test_updated_inst(got: &Instance, cmds: Vec<Command>, committed: bool, executed: bool) {
    assert_eq!(cmds, got.cmds, "cmds");
    assert_eq!(committed, got.committed, "committed");
    // TODO: check executed status of instance by RepicaStatus::Exec
}

#[test]
fn test_new_replica() {
    let cont = "
nodes:
    127.0.0.1:4441:
        api_addr: 127.0.0.1:3331
        replication: 127.0.0.1:5551
    192.168.0.1:4442:
        api_addr: 192.168.0.1:3332
        replication: 192.168.0.1:4442
groups:
-   range:
    -   a
    -   b
    replicas:
        1: 127.0.0.1:4441
        2: 192.168.0.1:4442
        3: 192.168.0.1:4442
";

    let ci = ClusterInfo::from_str(cont).unwrap();

    let mut rp = Replica::new(1, &ci, new_mem_sto()).unwrap();
    assert_eq!(1, rp.replica_id);

    rp.group_replica_ids.sort();
    assert_eq!(rp.group_replica_ids, [1, 2, 3]);

    rp.peers.sort_by(|x, y| x.replica_id.cmp(&y.replica_id));
    assert_eq!(2, rp.peers.len());
    assert_eq!(
        ReplicaPeer {
            replica_id: 2,
            addr: "http://192.168.0.1:4442".to_string(),
            alive: true
        },
        rp.peers[0]
    );
    assert_eq!(
        ReplicaPeer {
            replica_id: 3,
            addr: "http://192.168.0.1:4442".to_string(),
            alive: true
        },
        rp.peers[1]
    );

    let rp = Replica::new(4, &ci, new_mem_sto());
    assert!(rp.is_err());
}
