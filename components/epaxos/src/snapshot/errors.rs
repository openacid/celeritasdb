use crate::qpaxos::QError;
use crate::qpaxos::StorageFailure;

quick_error! {
    /// Errors occur when set/get with snapshot
    #[derive(Debug, PartialEq, Eq)]
    pub enum Error {
        DBError(msg: String) {
            from(msg: String) -> (msg)
            display("got db error:{}", msg)
        }

        NotFound{}
    }
}

impl Into<QError> for Error {
    fn into(self) -> QError {
        match self {
            Self::DBError(_) => QError {
                sto: Some(StorageFailure {}),
                ..Default::default()
            },

            Self::NotFound {} => QError {
                sto: Some(StorageFailure {}),
                ..Default::default()
            },
        }
    }
}
