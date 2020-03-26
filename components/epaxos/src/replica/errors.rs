use crate::qpaxos::ReplicaID;
use crate::qpaxos::{InvalidRequest, QError, StorageFailure};
use crate::snapshot::Error as SnapError;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        EngineError(s: SnapError) {
            from(err: SnapError) -> (err)
        }

        CmdNotSupport(s: String)

        SystemError(s: String) {
            from(err: std::time::SystemTimeError) -> (format!("{:?}", err))
        }

        Existed{}

        NoSuchReplica(rid: ReplicaID, my_rid: ReplicaID) {
            from(ids: (ReplicaID, ReplicaID)) -> (ids.0, ids.1)
            display("no such replica:{}, my replica_id:{}", rid, my_rid)
        }

        LackOf(field: String) {
            display("lack of required field:{}", field)
        }
    }
}

impl Error {
    pub fn to_qerr(&self) -> QError {
        match self {
            Self::EngineError(_) => QError {
                sto: Some(StorageFailure {}),
                ..Default::default()
            },

            Self::Existed {} => QError {
                sto: Some(StorageFailure {}),
                ..Default::default()
            },

            Self::NoSuchReplica(rid, my_rid) => QError {
                req: Some(InvalidRequest {
                    field: "cmn.to_replica_id".into(),
                    problem: "NotFound".into(),
                    ctx: format!("{}; my replica_id: {}", rid, my_rid).into(),
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

            _ => QError {
                ..Default::default()
            },
        }
    }
}
