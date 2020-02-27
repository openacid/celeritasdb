use crate::command::{Command, OpCode};
use crate::instance::{BallotNum, Instance, InstanceID};
use crate::message::*;
// Message is required to use to use method in trait Message.
use prost::Message;

fn new_foo_inst() -> Instance {
    let replica = 1;

    let inst_id1 = InstanceID::of(1, 10);
    let inst_id2 = InstanceID::of(2, 20);
    let inst_id3 = InstanceID::of(3, 30);
    let initial_deps = vec![inst_id1.clone(), inst_id2.clone(), inst_id3.clone()];

    let cmd1 = Command::of(OpCode::NoOp, "k1".as_bytes(), "v1".as_bytes());
    let cmd2 = Command::of(OpCode::Get, "k2".as_bytes(), "v2".as_bytes());
    let cmds = vec![cmd1, cmd2];
    let ballot = BallotNum::of(0, 0, replica);
    let ballot2 = BallotNum::of(1, 2, replica);

    let mut inst = Instance::of(&cmds[..], &ballot, &initial_deps[..]);
    // TODO move these to Instance::new_instance
    inst.instance_id = Some(inst_id1);
    inst.deps = [inst_id2].to_vec();
    inst.final_deps = [inst_id3].to_vec();
    inst.last_ballot = Some(ballot2);

    inst
}

// TODO test to_replica_id

macro_rules! test_request_common {
    ($msg:ident, $inst:ident) => {
        assert_eq!($inst.ballot, $msg.cmn.as_ref().unwrap().ballot);
        assert_eq!($inst.instance_id, $msg.cmn.as_ref().unwrap().instance_id);
    };
}

macro_rules! test_reply_common {
    ($msg:ident, $inst:ident) => {
        assert_eq!($inst.last_ballot, $msg.cmn.as_ref().unwrap().last_ballot);
        assert_eq!($inst.instance_id, $msg.cmn.as_ref().unwrap().instance_id);
    };
}

#[test]
fn test_request_prepare_pb() {
    let inst = new_foo_inst();

    let pp = Request::prepare(&inst);

    test_request_common!(pp, inst);
    // prepare has no other fields.

    test_enc_dec!(pp, PrepareRequest);
}

#[test]
fn test_reply_prepare_pb() {
    let inst = new_foo_inst();

    let pp = Reply::prepare(&inst);

    test_reply_common!(pp, inst);
    assert_eq!(inst.deps, pp.deps);
    assert_eq!(inst.final_deps, pp.final_deps);
    assert_eq!(inst.committed, pp.committed);

    test_enc_dec!(pp, PrepareReply);
}

#[test]
fn test_request_fast_accpt_pb() {
    let inst = new_foo_inst();

    let deps_committed = &[true, false];
    let pp = Request::fast_accept(&inst, deps_committed);

    test_request_common!(pp, inst);
    assert_eq!(inst.cmds, pp.cmds);
    assert_eq!(inst.initial_deps, pp.initial_deps);
    assert_eq!(deps_committed.to_vec(), pp.deps_committed);

    test_enc_dec!(pp, FastAcceptRequest);
}

#[test]
fn test_reply_fast_accept_pb() {
    let inst = new_foo_inst();

    let deps_committed = &[true, false];
    let pp = Reply::fast_accept(&inst, deps_committed);

    test_reply_common!(pp, inst);
    assert_eq!(inst.deps, pp.deps);
    assert_eq!(deps_committed.to_vec(), pp.deps_committed);

    test_enc_dec!(pp, FastAcceptReply);
}

#[test]
fn test_request_accpt_pb() {
    let inst = new_foo_inst();

    let pp = Request::accept(&inst);

    test_request_common!(pp, inst);
    assert_eq!(inst.final_deps, pp.final_deps);

    test_enc_dec!(pp, AcceptRequest);
}

#[test]
fn test_reply_accept_pb() {
    let inst = new_foo_inst();

    let pp = Reply::accept(&inst);

    test_reply_common!(pp, inst);
    // no other fields.

    test_enc_dec!(pp, AcceptReply);
}

#[test]
fn test_request_commit_pb() {
    let inst = new_foo_inst();

    let pp = Request::commit(&inst);

    test_request_common!(pp, inst);
    assert_eq!(inst.cmds, pp.cmds);
    assert_eq!(inst.final_deps, pp.final_deps);

    test_enc_dec!(pp, CommitRequest);
}

#[test]
fn test_reply_commit_pb() {
    let inst = new_foo_inst();

    let pp = Reply::commit(&inst);

    test_reply_common!(pp, inst);
    // no other fields.

    test_enc_dec!(pp, CommitReply);
}
