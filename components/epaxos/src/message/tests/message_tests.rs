use crate::command::{Command, OpCode};
use crate::instance::{BallotNum, Instance, InstanceID};
use crate::message::{Reply, Request, RequestType};
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

fn test_request_common_fields(inst: &Instance, req: &Request, t: RequestType) {
    assert_eq!(t as i32, req.req_type);
    assert_eq!(inst.ballot, req.ballot);
    assert_eq!(inst.instance_id, req.instance_id);
}

fn test_reply_common_fields(inst: &Instance, rp: &Reply, t: RequestType) {
    assert_eq!(t as i32, rp.req_type);
    assert_eq!(inst.last_ballot, rp.last_ballot);
    assert_eq!(inst.instance_id, rp.instance_id);
}

#[test]
fn test_request_prepare_pb() {
    let inst = new_foo_inst();

    let pp = Request::prepare(&inst);

    test_request_common_fields(&inst, &pp, RequestType::Prepare);
    // prepare has no other fields.

    test_enc_dec!(pp, Request);
}

#[test]
fn test_reply_prepare_pb() {
    let inst = new_foo_inst();

    let pp = Reply::prepare(&inst);

    test_reply_common_fields(&inst, &pp, RequestType::Prepare);
    assert_eq!(inst.deps, pp.deps);
    assert_eq!(inst.final_deps, pp.final_deps);
    assert_eq!(inst.committed, pp.committed);

    test_enc_dec!(pp, Reply);
}

#[test]
fn test_request_preaccpt_pb() {
    let inst = new_foo_inst();

    let deps_committed = &[true, false];
    let pp = Request::preaccept(&inst, deps_committed);

    test_request_common_fields(&inst, &pp, RequestType::PreAccept);
    assert_eq!(inst.cmds, pp.cmds);
    assert_eq!(inst.initial_deps, pp.initial_deps);
    assert_eq!(deps_committed.to_vec(), pp.deps_committed);

    test_enc_dec!(pp, Request);
}

#[test]
fn test_reply_preaccept_pb() {
    let inst = new_foo_inst();

    let deps_committed = &[true, false];
    let pp = Reply::preaccept(&inst, deps_committed);

    test_reply_common_fields(&inst, &pp, RequestType::PreAccept);
    assert_eq!(inst.deps, pp.deps);
    assert_eq!(deps_committed.to_vec(), pp.deps_committed);

    test_enc_dec!(pp, Reply);
}

#[test]
fn test_request_accpt_pb() {
    let inst = new_foo_inst();

    let pp = Request::accept(&inst);

    test_request_common_fields(&inst, &pp, RequestType::Accept);
    assert_eq!(inst.final_deps, pp.final_deps);

    test_enc_dec!(pp, Request);
}

#[test]
fn test_reply_accept_pb() {
    let inst = new_foo_inst();

    let pp = Reply::accept(&inst);

    test_reply_common_fields(&inst, &pp, RequestType::Accept);
    // no other fields.

    test_enc_dec!(pp, Reply);
}

#[test]
fn test_request_commit_pb() {
    let inst = new_foo_inst();

    let pp = Request::commit(&inst);

    test_request_common_fields(&inst, &pp, RequestType::Commit);
    assert_eq!(inst.cmds, pp.cmds);
    assert_eq!(inst.final_deps, pp.final_deps);

    test_enc_dec!(pp, Request);
}

#[test]
fn test_reply_commit_pb() {
    let inst = new_foo_inst();

    let pp = Reply::commit(&inst);

    test_reply_common_fields(&inst, &pp, RequestType::Commit);
    // no other fields.

    test_enc_dec!(pp, Reply);
}
