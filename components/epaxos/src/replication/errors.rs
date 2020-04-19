use crate::qpaxos::BallotNum;
use crate::qpaxos::Direction;
use crate::qpaxos::InstanceId;
use crate::qpaxos::ProtocolError;
use crate::qpaxos::QError;
use crate::qpaxos::ReplicaId;
use crate::qpaxos::StorageFailure;
use crate::replica::InstanceStatus;
use crate::replica::ReplicaError;
use parse::Response;
use storage::StorageError;

quick_error! {
    /// RpcHandlerError is an error encountered when handle-xx-request or handle-xx-reply.
    #[derive(Debug, Eq, PartialEq)]
    pub enum RpcHandlerError {
        /// A duplicated request/reply is received.
        DupRpc(phase: InstanceStatus, dir: Direction, from_rid: ReplicaId, inst_id: InstanceId) {
            from(f:(InstanceStatus, &str, ReplicaId, InstanceId)) -> (f.0, f.1.parse().unwrap(), f.2, f.3)
            display("dup {:?} {:?} from replica:{}, instance-id:{}", phase, dir, from_rid, inst_id)
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
        Protocol(p: ProtocolError) {
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

impl Into<QError> for RpcHandlerError {
    fn into(self) -> QError {
        match self {
            // TODO impl
            Self::DupRpc(_, _, _, _) => QError {
                sto: Some(StorageFailure {}),
                ..Default::default()
            },

            Self::RemoteError(qerr) => qerr.clone(),

            // TODO impl
            Self::StaleBallot(_stale, _last) => QError {
                sto: Some(StorageFailure {}),
                ..Default::default()
            },

            Self::Protocol(e) => e.into(),

            Self::Replica(e) => e.into(),

            Self::Storage(_) => QError {
                sto: Some(StorageFailure {}),
                ..Default::default()
            },

            // TODO impl
            Self::DelayedReply(_ip, _rp) => QError {
                sto: Some(StorageFailure {}),
                ..Default::default()
            },
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
