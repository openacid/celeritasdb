use crate::qpaxos::{ProtocolError, ReplicaID};
use crate::qpaxos::{QError, StorageFailure};
use storage::StorageError;

quick_error! {
    #[derive(Debug, Eq, PartialEq)]
    pub enum Error {
        EngineError(s: StorageError) {
            from(err: StorageError) -> (err)
        }

        CmdNotSupport(s: String)

        SystemError(s: String) {
            from(err: std::time::SystemTimeError) -> (format!("{:?}", err))
        }

        Existed{}

        Protocol(e: ProtocolError) {
            from(e: ProtocolError) -> (e)
        }

        ReplicaNotFound(rid: ReplicaID) {
            display("replica {:?} not found in cluster", rid)
        }
    }
}

impl Into<QError> for Error {
    fn into(self) -> QError {
        match self {
            Self::EngineError(_) => QError {
                sto: Some(StorageFailure {}),
                ..Default::default()
            },

            Self::Existed {} => QError {
                sto: Some(StorageFailure {}),
                ..Default::default()
            },

            // TODO impl
            Self::CmdNotSupport(_) => QError {
                sto: Some(StorageFailure {}),
                ..Default::default()
            },

            // TODO impl
            Self::SystemError(_) => QError {
                sto: Some(StorageFailure {}),
                ..Default::default()
            },

            Self::Protocol(e) => e.into(),

            // TODO impl
            Self::ReplicaNotFound(_) => QError {
                sto: Some(StorageFailure {}),
                ..Default::default()
            },
        }
    }
}
