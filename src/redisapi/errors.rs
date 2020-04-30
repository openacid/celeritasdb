use epaxos::replica::ReplicaError;
use epaxos::RangeLookupError;
use epaxos::ReplicationError;
use storage::StorageError;
use tokio::sync::oneshot::error::RecvError;

quick_error! {
    #[derive(Debug, PartialEq)]
    pub enum RedisApiError{
        ExecCommandError(msg: String) {
            from(err: RangeLookupError) -> (format!("{:?}", err))
            from(err: ReplicaError) -> (format!("{:?}", err))
            from(err: ReplicationError) -> (format!("{:?}", err))
            from(err: StorageError) -> (format!("{:?}", err))
            from(err: RecvError) -> (format!("{:?}", err))
        }
    }
}
