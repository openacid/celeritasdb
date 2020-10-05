use crate::qpaxos::*;
use prost::Message;

#[test]
fn test_command_pb() {
    let cmd1 = cmd!("NoOp", "key", "value");

    test_enc_dec!(cmd1, Command);
}

#[test]
fn test_op_code_from() {
    assert_eq!(OpCode::NoOp, OpCode::from_str("NoOp").unwrap());
    assert_eq!(OpCode::Get, OpCode::from_str("Get").unwrap());
    assert_eq!(OpCode::Set, OpCode::from_str("Set").unwrap());
    assert_eq!(OpCode::Delete, OpCode::from_str("Delete").unwrap());
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
    let k = "key".as_bytes();
    let v = "value".as_bytes();
    assert_eq!(c, (OpCode::Set, k, v).into());
}

#[test]
fn test_command_conflit() {
    let nx = Command::from(("NoOp", "x", "1"));
    let gx = Command::from(("Get", "x", "1"));
    let sx = Command::from(("Set", "x", "1"));
    let dx = Command::from(("Delete", "x", "1"));

    let ny = Command::from(("NoOp", "y", "1"));
    let gy = Command::from(("Get", "y", "1"));
    let sy = Command::from(("Set", "y", "1"));
    let dy = Command::from(("Delete", "y", "1"));

    let xs = vec![nx, gx, sx, dx];
    let ys = vec![ny, gy, sy, dy];

    // conflicts[i, j] indicates whether xs[i] and xs[j] conflict.
    let conflicts = vec![
        vec![0, 0, 0, 0],
        vec![0, 0, 1, 1],
        vec![0, 1, 1, 1],
        vec![0, 1, 1, 1],
    ];

    for (i, v) in conflicts.iter().enumerate() {
        for (j, c) in v.iter().enumerate() {
            let want = *c == 1;

            assert_eq!(
                want,
                xs[i].conflict(&xs[j]),
                "x x check conflict: {} {}",
                i,
                j
            );
            assert_eq!(
                want,
                ys[i].conflict(&ys[j]),
                "y y check conflict: {} {}",
                i,
                j
            );

            assert!(!xs[i].conflict(&ys[j]), "x y never conflict: {} {}", i, j);
            assert!(!ys[i].conflict(&xs[j]), "y x never conflict: {} {}", i, j);
        }
    }
}

#[test]
fn test_macro_cmd() {
    let cmd = cmd!("Set", "key", "value");
    assert_eq!(cmd, (OpCode::Set, "key", "value").into());

    let cmd = cmd!(key = value);
    assert_eq!(cmd, (OpCode::Set, "key", "value").into());
}

#[test]
fn test_macro_cmdvec() {
    let cmds = cmdvec![(a = b), ("Set", "key", "value"), (OpCode::Get, "a", "b")];

    assert_eq!(cmds[0], (OpCode::Set, "a", "b").into());
    assert_eq!(cmds[1], (OpCode::Set, "key", "value").into());
    assert_eq!(cmds[2], ("Get", "a", "b").into());
    assert_eq!(3, cmds.len());
}
