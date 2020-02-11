use super::*;
use crate::command::{Command, OpCode};
use protobuf::{parse_from_bytes, Message};

#[test]
fn test_instance_protobuf() {
    let inst_id1 = InstanceID::new_instance_id(1, 10);
    let inst_id2 = InstanceID::new_instance_id(2, 20);
    let inst_id3 = InstanceID::new_instance_id(3, 30);
    let initial_deps = vec![inst_id1, inst_id2, inst_id3];

    let status = InstanceStatus::NA;

    let cmd1 = Command::new_command(OpCode::NoOp, "k1", "v1");
    let cmd2 = Command::new_command(OpCode::Get, "k2", "v2");
    let cmds = vec![cmd1, cmd2];

    let ballot = BallotNum::new_ballot_num(1, 2, 3);

    let inst1 = Instance::new_instance(status, &cmds[..], &ballot, &initial_deps[..]);

    let inst_bytes: Vec<u8> = inst1.write_to_bytes().unwrap();

    let inst2 = parse_from_bytes::<Instance>(&inst_bytes).unwrap();

    assert_eq!(inst2.status, status);
    assert_eq!(inst2.cmds.into_vec(), cmds);
    assert_eq!(*inst2.ballot.get_ref(), ballot);
    for (idx, inst_id) in inst2.initial_deps.get_ref().ids.iter().enumerate() {
        assert_eq!(*inst_id, initial_deps[idx]);
    }
}
