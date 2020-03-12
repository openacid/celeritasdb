use crate::qpaxos::*;
use crate::replica::*;
use crate::snapshot::MemEngine;

fn new_foo_inst(leader_id: i64) -> Instance {
    let iid1 = InstanceId::from((1, 10));
    let iid2 = InstanceId::from((2, 20));
    let iid3 = InstanceId::from((3, 30));
    let initial_deps = vec![iid1, iid2, iid3];

    let cmd1 = ("NoOp", "k1", "v1").into();
    let cmd2 = ("Get", "k2", "v2").into();
    let cmds = vec![cmd1, cmd2];
    let ballot = (2, 2, leader_id).into();
    let ballot2 = (1, 2, leader_id).into();

    let mut inst = Instance::of(&cmds[..], ballot, &initial_deps[..]);
    // TODO move these to Instance::new_instance
    inst.instance_id = Some((leader_id, 1).into());
    inst.deps = [iid2].to_vec();
    inst.final_deps = [iid3].to_vec();
    inst.last_ballot = Some(ballot2);

    inst
}

fn new_foo_replica(replica_id: i64) -> Replica {
    Replica {
        replica_id,
        group_replica_ids: vec![1, 2, 3],
        status: ReplicaStatus::Running,
        peers: vec![],
        conf: ReplicaConf {
            ..Default::default()
        },
        inst_idx: 0,
        latest_cp: (1, 1).into(),
        storage: Box::new(MemEngine::new().unwrap()),
        problem_inst_ids: vec![],
    }
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
fn test_handle_xxx_request_invalid() {
    let replica_id = 2;
    let mut replica = new_foo_replica(replica_id);

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
fn test_handle_fast_accept_request() {
    let replica_id = 1;
    let mut replica = new_foo_replica(replica_id);
    replica.group_replica_ids = vec![0, 1, 2];

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
        _test_get_inst(&replica, iid, blt, blt, inst.cmds, vec![]);
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
        // instx
        let x_iid = (0, 0).into();
        let cmd1 = ("Set", "key_x", "val_x").into();
        let cmds = vec![cmd1];
        let ballot = (0, 0, 0).into();
        let initial_deps = vec![];

        let mut instx = Instance::of(&cmds[..], ballot, &initial_deps[..]);
        instx.deps = vec![(0, 0).into(), (0, 0).into(), (0, 0).into()];
        instx.instance_id = Some(x_iid);
        replica.storage.set_instance(&instx).unwrap();

        // insty
        let y_iid = (1, 0).into();
        let cmd1 = ("Get", "key_y", "val_y").into();
        let cmds = vec![cmd1];
        let ballot = (0, 0, 1).into();
        let initial_deps = vec![];

        let mut insty = Instance::of(&cmds[..], ballot, &initial_deps[..]);
        insty.deps = vec![(0, 0).into(), (0, 0).into(), (0, 0).into()];
        insty.instance_id = Some(y_iid);
        replica.storage.set_instance(&insty).unwrap();

        // instz
        let z_iid = (2, 0).into();
        let cmd1 = ("Get", "key_z", "val_z").into();
        let cmds = vec![cmd1];
        let ballot = (0, 0, 2).into();
        let initial_deps = vec![];

        let mut instz = Instance::of(&cmds[..], ballot, &initial_deps[..]);
        instz.deps = vec![(0, 0).into(), (0, 0).into(), (0, 0).into()];
        instz.instance_id = Some(z_iid);
        replica.storage.set_instance(&instz).unwrap();

        // instb
        let b_iid = (1, 1).into();
        let cmd1 = ("Get", "key_b", "val_b").into();
        let cmds = vec![cmd1];
        let ballot = (0, 0, 1).into();
        let initial_deps = vec![];

        let mut instb = Instance::of(&cmds[..], ballot, &initial_deps[..]);
        instb.deps = vec![x_iid, y_iid, z_iid];
        instb.instance_id = Some(b_iid);
        instb.committed = true;
        replica.storage.set_instance(&instb).unwrap();

        // insta
        let a_iid = (0, 1).into();
        let cmd1 = ("Get", "key_a", "val_a").into();
        let cmds = vec![cmd1];
        let ballot = (0, 0, 0).into();
        let initial_deps = vec![x_iid, y_iid, z_iid];

        let mut insta = Instance::of(&cmds[..], ballot, &initial_deps[..]);
        insta.deps = vec![x_iid, y_iid, z_iid];
        insta.instance_id = Some(a_iid);

        // instd
        let d_iid = (0, 2).into();
        let cmd1 = ("Get", "key_d", "val_d").into();
        let cmds = vec![cmd1];
        let ballot = (0, 0, 0).into();
        let initial_deps = vec![];

        let mut instd = Instance::of(&cmds[..], ballot, &initial_deps[..]);
        instd.deps = vec![a_iid, b_iid, z_iid];
        instd.instance_id = Some(d_iid);
        replica.storage.set_instance(&instd).unwrap();

        // instc
        let c_iid = (2, 3).into();
        let cmd1 = ("Get", "key_z", "val_z").into();
        let cmds = vec![cmd1];
        let ballot = (0, 0, 2).into();
        let initial_deps = vec![];

        let mut instc = Instance::of(&cmds[..], ballot, &initial_deps[..]);
        instc.deps = vec![d_iid, b_iid, z_iid];
        instc.instance_id = Some(c_iid);
        replica.storage.set_instance(&instc).unwrap();

        let deps_committed = vec![false, true, false];
        let req = MakeRequest::fast_accept(replica_id, &insta, &deps_committed);
        let repl = replica.handle_fast_accept(&req);

        insta.deps = vec![x_iid, b_iid, z_iid];

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
            vec![],
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

    let mut replica = new_foo_replica(replica_id);
    let none = replica.storage.get_instance(iid).unwrap();
    assert_eq!(None, none);

    {
        // ok reply with none instance.
        let req = MakeRequest::accept(replica_id, &inst);
        let repl = replica.handle_accept(&req);
        assert_eq!(None, repl.err);
        _test_repl_cmn_ok(&repl.cmn.unwrap(), iid, None);

        // get the written instance.
        _test_get_inst(&replica, iid, blt, None, vec![], fdeps.clone());
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
        _test_get_inst(&replica, iid, blt, blt, vec![], fdeps.clone());
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
        curr.final_deps = vec![];
        replica.storage.set_instance(&curr).unwrap();

        let curr = replica.storage.get_instance(iid).unwrap().unwrap();
        assert!(curr.ballot > blt);

        // accept wont update this instance.
        let repl = replica.handle_accept(&req);
        assert_eq!(None, repl.err);
        _test_repl_cmn_ok(&repl.cmn.unwrap(), iid, bigger);

        // get the intact instance.
        _test_get_inst(&replica, iid, bigger, blt, vec![], vec![]);
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

    let mut replica = new_foo_replica(replica_id);
    let none = replica.storage.get_instance(iid).unwrap();
    assert_eq!(None, none);

    let req = MakeRequest::commit(replica_id, &inst);

    {
        // ok reply with none instance.
        let repl = replica.handle_commit(&req);
        assert_eq!(None, repl.err);
        _test_repl_cmn_ok(&repl.cmn.unwrap(), iid, None);

        // get the committed instance.
        _test_get_inst(&replica, iid, blt, None, cmds.clone(), fdeps.clone());
    }

    {
        // ok reply when replacing instance.
        let repl = replica.handle_commit(&req);
        assert_eq!(None, repl.err);
        _test_repl_cmn_ok(&repl.cmn.unwrap(), iid, blt);

        // get the committed instance.
        _test_get_inst(&replica, iid, blt, blt, cmds.clone(), fdeps.clone());
    }

    // TODO test storage error
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
    final_deps: Vec<InstanceId>,
) {
    let got = replica.storage.get_instance(iid).unwrap().unwrap();
    assert_eq!(iid, got.instance_id.unwrap());
    assert_eq!(blt, got.ballot);
    assert_eq!(last, got.last_ballot);
    assert_eq!(cmds, got.cmds);
    assert_eq!(final_deps, got.final_deps);
}
