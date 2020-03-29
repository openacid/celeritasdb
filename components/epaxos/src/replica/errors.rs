use crate::qpaxos::ProtocolError;
use crate::qpaxos::{QError, StorageFailure};
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

        Protocol(e: ProtocolError) {
            from(e: ProtocolError) -> (e)
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
        }
    }
}
