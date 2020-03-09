use crate::qpaxos::{InvalidRequest, QError, StorageFailure};

quick_error! {
    /// Errors occur when set/get with snapshot
    #[derive(Debug, PartialEq)]
    pub enum Error {
        DBError{msg: String} {
            from(msg: String) -> {msg: msg}
            display("got db error:{}", msg)
        }

        NotFound{}

        NoSuchReplica{
            replica_id: i64,
            my_replica_id: i64,
        } {
            // from(msg: String) -> {msg: msg}
            display("no such replica:{}, my replica_id:{}",
                    replica_id, my_replica_id)
        }
        LackOf(field: String) {
            display("lack of required field:{}",
                    field)
        }
    }
}

impl Error {
    pub fn to_qerr(&self) -> QError {
        match self {
            Error::DBError { msg: _ } => QError {
                sto: Some(StorageFailure {}),
                ..Default::default()
            },

            Self::NotFound {} => QError {
                sto: Some(StorageFailure {}),
                ..Default::default()
            },

            Self::NoSuchReplica {
                replica_id: rid,
                my_replica_id: mrid,
            } => QError {
                req: Some(InvalidRequest {
                    field: "cmn.to_replica_id".into(),
                    problem: "NotFound".into(),
                    ctx: format!("{}; my replica_id: {}", rid, mrid).into(),
                }),
                ..Default::default()
            },

            Self::LackOf(f) => QError {
                req: Some(InvalidRequest {
                    field: f.clone(),
                    problem: "LackOf".into(),
                    ctx: "".into(),
                }),
                ..Default::default()
            },
        }
    }
}
