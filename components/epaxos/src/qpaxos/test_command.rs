use crate::qpaxos::*;
use prost::Message;

#[test]
fn test_command_pb() {
    let cmd1 = Command::of(OpCode::NoOp, "key".as_bytes(), "value".as_bytes());

    test_enc_dec!(cmd1, Command);
}

#[test]
fn test_command_from() {
    let c = Command{
        op: OpCode::Set as i32,
        key: "key".as_bytes().to_vec(),
        value: "value".as_bytes().to_vec(), 
    };


    assert_eq!(c, (OpCode::Set, "key", "value").into());
    assert_eq!(c, ("Set", "key", "value").into());
}

