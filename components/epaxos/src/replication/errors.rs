use crate::qpaxos::BallotNum;
use crate::qpaxos::ProtocolError;
use crate::qpaxos::QError;
use crate::qpaxos::ReplicaID;
use crate::replica::Error as ReplicaError;
use crate::snapshot::Error as SnapError;

quick_error! {
    /// HandlerError is an error encountered when handle-xx-request or handle-xx-reply.
    #[derive(Debug, Eq, PartialEq)]
    pub enum HandlerError {
        /// A duplicated request/reply is received.
        Dup(rid: ReplicaID) {
            from(rid: ReplicaID) -> (rid)
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
            from(e: SnapError) -> (e.into())
        }

        /// A delay reply is received
        DelayReply(msg: String) {
            display("receive delay msg :{}", &msg)
        }
    }
}
