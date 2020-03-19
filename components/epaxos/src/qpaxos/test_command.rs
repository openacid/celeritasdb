#[macro_use]
use crate::qpaxos::*;
use prost::Message;

#[test]
fn test_command_pb() {
    let cmd1 = Command::of(OpCode::NoOp, "key".as_bytes(), "value".as_bytes());

    test_enc_dec!(cmd1, Command);
}

#[test]
fn test_command_from() {
    let c = Command {
        op: OpCode::Set as i32,
        key: "key".as_bytes().to_vec(),
        value: "value".as_bytes().to_vec(),
    };

    assert_eq!(c, (OpCode::Set, "key", "value").into());
    assert_eq!(c, ("Set", "key", "value").into());
}

#[test]
fn test_command_conflit() {
    let nx = Command::from(("NoOp", "x", "1"));
    let gx = Command::from(("Get", "x", "1"));
    let sx = Command::from(("Set", "x", "1"));

    let ny = Command::from(("NoOp", "y", "1"));
    let gy = Command::from(("Get", "y", "1"));
    let sy = Command::from(("Set", "y", "1"));

    assert!(!nx.conflict(&nx));
    assert!(!nx.conflict(&gx));
    assert!(!nx.conflict(&sx));

    assert!(!gx.conflict(&nx));
    assert!(!gx.conflict(&gx));
    assert!(gx.conflict(&sx));

    assert!(!sx.conflict(&nx));
    assert!(sx.conflict(&gx));
    assert!(sx.conflict(&sx));

    assert!(!sx.conflict(&ny));
    assert!(!sx.conflict(&gy));
    assert!(!sx.conflict(&sy));

    assert!(!sy.conflict(&nx));
    assert!(!sy.conflict(&gx));
    assert!(!sy.conflict(&sx));
}

#[test]
fn test_macro_cmds() {
    let cmds = cmds![("Set", "key", "value"), (OpCode::Get, "a", "b")];

    assert_eq!(cmds[0], (OpCode::Set, "key", "value").into());
    assert_eq!(cmds[1], ("Get", "a", "b").into());
    assert_eq!(2, cmds.len());
}
