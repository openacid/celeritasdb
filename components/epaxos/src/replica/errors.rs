use crate::qpaxos::ReplicaId;
use crate::qpaxos::{QError, StorageFailure};
use storage::StorageError;

quick_error! {
    #[derive(Debug, Eq, PartialEq)]
    pub enum ReplicaError {
        Storage(s: StorageError) {
            from(err: StorageError) -> (err)
        }

        Existed{}

        ReplicaNotFound(rid: ReplicaId) {
            display("replica {:?} not found in cluster", rid)
        }
    }
}

impl Into<QError> for ReplicaError {
    fn into(self) -> QError {
        match self {
            Self::Storage(_) => QError {
                sto: Some(StorageFailure {}),
                ..Default::default()
            },

            Self::Existed {} => QError {
                sto: Some(StorageFailure {}),
                ..Default::default()
            },

            // TODO impl
            Self::ReplicaNotFound(_) => QError {
                sto: Some(StorageFailure {}),
                ..Default::default()
            },
        }
    }
}
