use crate::qpaxos::errors::ProtocolError;
use crate::qpaxos::{InvalidRequest, QError};

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn test_protocol_error() {
    // NoSuchReplica
    let e = ProtocolError::NoSuchReplica(1, 2);
    assert_eq!("no such replica:1, my replica_id:2", format!("{}", e));

    let e = ProtocolError::from((1, 2));
    assert_eq!("no such replica:1, my replica_id:2", format!("{}", e));

    let q: QError = e.into();
    assert_eq!(
        q,
        QError {
            req: Some(InvalidRequest {
                field: "cmn.to_replica_id".into(),
                problem: "NotFound".into(),
                ctx: format!("{}; my replica_id: {}", 1, 2).into(),
            }),
            ..Default::default()
        }
    );

    // LackOf
    let e = ProtocolError::LackOf("a.b".into());
    assert_eq!("lack of required field:a.b", format!("{}", e));

    let q: QError = e.into();
    assert_eq!(
        q,
        QError {
            req: Some(InvalidRequest {
                field: "a.b".into(),
                problem: "LackOf".into(),
                ctx: "".into(),
            }),
            ..Default::default()
        }
    );
}
