use crate::qpaxos::BallotNum;
use crate::qpaxos::Command;
use crate::qpaxos::Dep;
use crate::qpaxos::Instance;
use crate::qpaxos::InstanceId;
use crate::qpaxos::InstanceIdVec;
use crate::qpaxos::MakeRequest;
use crate::qpaxos::OpCode;

use crate::instids;
use crate::qpaxos::replicate_reply;
use crate::qpaxos::AcceptReply;
use crate::qpaxos::CommitReply;
use crate::qpaxos::InvalidRequest;
use crate::qpaxos::PrepareReply;
use crate::qpaxos::QError;
use crate::qpaxos::ReplicateReply;
use crate::qpaxos::StorageFailure;
use crate::InstanceIds;

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
    assert_eq!("(2, 3)", format!("{}", BallotNum::from((2, 3))));
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
        format!("{}", InstanceIdVec::from(instidvec![(1, 2), (3, 4)]))
    );
}

#[test]
fn test_display_instance_ids() {
    assert_eq!("{1:2, 3:4}", format!("{}", instids! {(1,2),(3,4)}));
}

#[test]
fn test_display_instance() {
    let inst = inst!(
        (1, 2),
        (3, 4),
        [("Set", "a", "b"), ("Get", "c", "d")],
        [(3, 4), (4, 5)],
        (6, 7),
        false,
    );
    assert_eq!("{id:(1, 2), blt:(3, 4), ablt:(6, 7), cmds:[Set:a=b, Get:c], deps:[(3, 4, 0), (4, 5, 0)], c:false}",
    format!("{}", inst));
}

#[test]
fn test_display_replicate_request() {
    let inst = inst!(
        (1, 2),
        (3, 4),
        [("Set", "a", "b"), ("Get", "c", "d")],
        [(2, 3), (3, 4)],
        (2, 3),
        false,
    );

    let r = "to:10, blt:(3, 4), iid:(1, 2), phase";

    let prepare = "Prepare{cmds:[Set:a=b, Get:c], deps:[(2, 3, 0), (3, 4, 0)], c:[true, false]}";
    let accept = "Accept{deps:[(2, 3, 0), (3, 4, 0)]}";
    let commit = "Commit{cmds:[Set:a=b, Get:c], deps:[(2, 3, 0), (3, 4, 0)]}";

    let f = MakeRequest::prepare(10, &inst, &[true, false]);
    assert_eq!(format!("{{{}:{}}}", r, prepare), format!("{}", f));

    let a = MakeRequest::accept(10, &inst);
    assert_eq!(format!("{{{}:{}}}", r, accept), format!("{}", a));

    let c = MakeRequest::commit(10, &inst);
    assert_eq!(format!("{{{}:{}}}", r, commit), format!("{}", c));
}

#[test]
fn test_display_replicate_reply_err() {
    let cmn = "last:None, iid:None, phase";

    {
        // storage error
        let r = ReplicateReply {
            err: Some(QError {
                sto: Some(StorageFailure::default()),
                req: None,
            }),
            ..Default::default()
        };
        let e = "{sto:StorageFailure, req:None}";

        assert_eq!(
            format!("{{err:{}, {}:{}}}", e, cmn, "None"),
            format!("{}", r)
        );
    }
    {
        // request error
        let r = ReplicateReply {
            err: Some(QError {
                sto: None,
                req: Some(InvalidRequest {
                    field: "foo".into(),
                    problem: "must-have".into(),
                    ctx: "ctxbar".into(),
                }),
            }),
            ..Default::default()
        };
        let e = "{sto:None, req:{must-have: 'foo', ctx:ctxbar}}";

        assert_eq!(
            format!("{{err:{}, {}:{}}}", e, cmn, "None"),
            format!("{}", r)
        );
    }
}

#[test]
fn test_display_replicate_reply_normal() {
    let cmn = "last:(3, 4), iid:(1, 2), phase";

    let mut r = ReplicateReply {
        err: None,
        last_ballot: Some((3, 4).into()),
        instance_id: Some((1, 2).into()),
        phase: None,
    };

    {
        r.phase = Some(replicate_reply::Phase::Prepare(PrepareReply {
            deps: Some(instidvec![(1, 2), (3, 4)].into()),
            deps_committed: vec![true, false],
        }));
        let ph = "Prepare{deps[1]:[(1, 2, 0), (3, 4, 0)], c:[true, false]}";

        assert_eq!(format!("{{err:None, {}:{}}}", cmn, ph), format!("{}", r));
    }

    {
        r.phase = Some(replicate_reply::Phase::Accept(AcceptReply {}));
        let ph = "Accept{}";

        assert_eq!(format!("{{err:None, {}:{}}}", cmn, ph), format!("{}", r));
    }

    {
        r.phase = Some(replicate_reply::Phase::Commit(CommitReply {}));
        let ph = "Commit{}";

        assert_eq!(format!("{{err:None, {}:{}}}", cmn, ph), format!("{}", r));
    }
}
