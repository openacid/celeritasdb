use super::*;
use protobuf::{parse_from_bytes, Message};

#[test]
fn test_command_protobuf() {
    let cmd1 = Command::new_command(OpCode::NoOp, "key", "value");

    let cmd_bytes: Vec<u8> = cmd1.write_to_bytes().unwrap();

    let cmd2 = parse_from_bytes::<Command>(&cmd_bytes).unwrap();

    assert_eq!(cmd1.op, cmd2.op);
    assert_eq!(cmd1.key, cmd2.key);
    assert_eq!(cmd1.value, cmd2.value);
}
