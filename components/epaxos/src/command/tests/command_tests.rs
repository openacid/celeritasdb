use super::*;
use prost::Message;

#[test]
fn test_command_protobuf() {
    let cmd1 = Command::of(OpCode::NoOp, "key".as_bytes(), "value".as_bytes());

    test_enc_dec!(cmd1, Command);
}
