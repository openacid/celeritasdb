use crate::qpaxos::BallotNum;
use crate::qpaxos::ProtocolError;
use crate::qpaxos::QError;
use crate::qpaxos::ReplicaId;
use crate::replica::Error as ReplicaError;
use crate::replica::InstanceStatus;
use parse::Response;
use storage::StorageError;

quick_error! {
    /// RpcHandlerError is an error encountered when handle-xx-request or handle-xx-reply.
    #[derive(Debug, Eq, PartialEq)]
    pub enum RpcHandlerError {
        /// A duplicated request/reply is received.
        Dup(rid: ReplicaId) {
            from(rid: ReplicaId) -> (rid)
        }

        /// There is an error occured on remote peer.
        RemoteError(qerr: QError) {
            from(qerr: QError) -> (qerr)
        }

        /// The ballot number is too small to proceed.
        StaleBallot(stale: BallotNum, last: BallotNum) {
            from(bb: (BallotNum, BallotNum)) -> (bb.0, bb.1)
        }

        /// A malformed protocol error.
        Protocal(p: ProtocolError) {
            from(p: ProtocolError) -> (p)
        }

        /// A malformed replica error.
        Replica(r: ReplicaError) {
            from(r: ReplicaError) -> (r)
        }

        Storage(s: StorageError) {
            from(s: StorageError) -> (s)
        }

        /// A delay reply is received
        DelayedReply(inst_phase: InstanceStatus, reply_phase: InstanceStatus) {
            display("instance phase:{:?} while recv reply of phase: {:?}", inst_phase, reply_phase)
        }
    }
}

quick_error! {
    /// ReplicationError is an error encountered during replicating an instance.
    #[derive(Debug)]
    pub enum ReplicationError {
        NotEnoughQuorum(phase: InstanceStatus, want: i32, got: i32) {
            display("{:?}: want at least {} replies, but:{}", phase, want, got)
        }
        Replica(re: ReplicaError) {
            from(re: ReplicaError) -> (re)
        }
        Handler(e: RpcHandlerError) {
            from(e: RpcHandlerError) -> (e)
        }
        Storage(e: StorageError) {
            from(e: StorageError) -> (e)
        }
    }
}

impl From<ReplicationError> for Response {
    fn from(e: ReplicationError) -> Self {
        Response::Error(format!("{:?}", e))
    }
}
