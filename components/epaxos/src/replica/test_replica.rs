use crate::qpaxos::*;
use crate::replica::*;
use crate::snapshot::MemEngine;

fn new_foo_inst(leader_id: i64) -> Instance {
    let iid1 = InstanceID::from((1, 10));
    let iid2 = InstanceID::from((2, 20));
    let iid3 = InstanceID::from((3, 30));
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

    // InvalidRequest from accept
    for (cmn, etuple) in cases.clone() {
        let req = AcceptRequest {
            cmn,
            ..Default::default()
        };
        let repl = replica.handle_accept(&req);
        let err = repl.err.unwrap();
        assert_eq!(
            QError {
                req: Some(etuple.into()),
                ..Default::default()
            },
            err
        );
    }

    // InvalidRequest from commit
    for (cmn, etuple) in cases.clone() {
        let req = CommitRequest {
            cmn,
            ..Default::default()
        };
        let repl = replica.handle_commit(&req);
        let err = repl.err.unwrap();
        assert_eq!(
            QError {
                req: Some(etuple.into()),
                ..Default::default()
            },
            err
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

fn _test_repl_cmn_ok(cmn: &ReplyCommon, iid: InstanceID, last: Option<BallotNum>) {
    assert_eq!(iid, cmn.instance_id.unwrap());
    assert_eq!(last, cmn.last_ballot);
}

fn _test_get_inst(
    replica: &Replica,
    iid: InstanceID,
    blt: Option<BallotNum>,
    last: Option<BallotNum>,
    cmds: Vec<Command>,
    final_deps: Vec<InstanceID>,
) {
    let got = replica.storage.get_instance(iid).unwrap().unwrap();
    assert_eq!(iid, got.instance_id.unwrap());
    assert_eq!(blt, got.ballot);
    assert_eq!(last, got.last_ballot);
    assert_eq!(cmds, got.cmds);
    assert_eq!(final_deps, got.final_deps);
}
