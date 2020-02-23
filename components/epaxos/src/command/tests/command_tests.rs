use super::*;
use prost::Message;

#[test]
fn test_command_protobuf() {
    let cmd1 = Command::of(OpCode::NoOp, "key".as_bytes(), "value".as_bytes());

    let mut cmd_bytes = vec![];
    cmd1.encode(&mut cmd_bytes).unwrap();

    let cmd2 = Command::decode(cmd_bytes.as_slice()).unwrap();

    assert_eq!(cmd1.op, cmd2.op);
    assert_eq!(cmd1.key, cmd2.key);
    assert_eq!(cmd1.value, cmd2.value);
}
