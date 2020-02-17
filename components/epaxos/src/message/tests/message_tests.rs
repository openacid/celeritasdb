use super::*;
use crate::command::{Command, OpCode};
use crate::instance::{BallotNum, Instance, InstanceID, InstanceStatus};
use crate::message::{Reply, Request};
// Message is required to use to use method in trait Message.
use protobuf::RepeatedField;
use protobuf::{parse_from_bytes, Message};

fn new_foo_inst() -> Instance {
    let replica = 1;

    let inst_id1 = InstanceID::new_instance_id(1, 10);
    let inst_id2 = InstanceID::new_instance_id(2, 20);
    let inst_id3 = InstanceID::new_instance_id(3, 30);
    let initial_deps = vec![inst_id1.clone(), inst_id2.clone(), inst_id3.clone()];

    let status = InstanceStatus::NA;

    let cmd1 = Command::new_command(OpCode::NoOp, "k1", "v1");
    let cmd2 = Command::new_command(OpCode::Get, "k2", "v2");
    let cmds = vec![cmd1, cmd2];

    let ballot = BallotNum::new_ballot_num(0, 0, replica);
    let ballot2 = BallotNum::new_ballot_num(1, 2, replica);

    let mut inst = Instance::new_instance(status, &cmds[..], &ballot, &initial_deps[..]);
    // TODO move these to Instance::new_instance
    inst.set_instance_id(inst_id1);
    inst.set_deps(RepeatedField::from_slice(&[inst_id2]));
    inst.set_final_deps(RepeatedField::from_slice(&[inst_id3]));
    inst.set_last_ballot(ballot2);

    inst
}

#[test]
fn test_message_protobuf() {
    // pub fn new_message(
    //     req_type: RequestType,
    //     msg_type: MessageType,
    //     data: Vec<u8>,
    //
    //
    // pub fn new_prepare_req(
    // leader: ReplicaID,
    // replica: ReplicaID,
    // inst_id: InstanceID,
    // ballot: BallotNum,

    let leader = 0;
    let replica = 1;
    let inst_id = InstanceID::new_instance_id(replica, 0);
    let ballot = BallotNum::new_ballot_num(0, 0, replica);
    let pr1 = PrepareReq::new_prepare_req(leader, replica, inst_id, ballot);
    let pr_bytes: Vec<u8> = pr1.write_to_bytes().unwrap();

    let req_type = RequestType::Prepare;
    let msg_type = MessageType::TypeRequest;

    let msg1 = super::Message::new_message(req_type, msg_type, pr_bytes);
    let size = msg1.compute_size();
    let msg_bytes: Vec<u8> = msg1.write_to_bytes().unwrap();

    let msg2 = parse_from_bytes::<super::Message>(&msg_bytes).unwrap();

    assert_eq!(req_type, msg2.req_type);
    assert_eq!(msg_type, msg2.msg_type);

    let pr2 = parse_from_bytes::<PrepareReq>(&msg2.data).unwrap();
    assert_eq!(pr1, pr2);
}

#[test]
fn test_request_prepare_protobuf() {
    let req_type = RequestType::Prepare;
    let msg_type = MessageType::TypeRequest;

    let inst = new_foo_inst();

    let pr1 = Request::prepare(&inst);
    let pr_bytes: Vec<u8> = pr1.write_to_bytes().unwrap();
    let size = pr1.compute_size();

    assert_eq!(true, size > 0);

    let pr2 = parse_from_bytes::<Request>(&pr_bytes).unwrap();
    assert_eq!(pr1, pr2);
}
