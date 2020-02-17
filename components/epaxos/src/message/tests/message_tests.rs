use super::*;
use crate::command::{Command, OpCode};
use crate::instance::{BallotNum, Instance, InstanceID, InstanceStatus};
use crate::message::{Reply, Request};
use protobuf::RepeatedField;
use protobuf::{parse_from_bytes, Message};

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
