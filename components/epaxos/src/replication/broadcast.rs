use tonic::Response;

use crate::qpaxos::AcceptReply;
use crate::qpaxos::CommitReply;
use crate::qpaxos::FastAcceptReply;
use crate::qpaxos::Instance;
use crate::qpaxos::MakeRequest;
use crate::qpaxos::PrepareReply;
use crate::qpaxos::QPaxosClient;
use crate::qpaxos::ReplicaId;
use crate::replica::ReplicaPeer;

macro_rules! bcast_msg {
    ($peers:expr, $make_req:expr, $func:ident) => {{
        let mut rst = Vec::with_capacity($peers.len());
        for p in $peers.iter() {
            let mut client = match QPaxosClient::connect(p.addr.clone()).await {
                Ok(c) => c,
                // TODO just ignore the err
                Err(e) => {
                    println!("{:?} while connect to {:?}", e, &p.addr);
                    continue;
                }
            };

            let req = $make_req(p.replica_id);
            let repl = match client.$func(req).await {
                Ok(r) => r,
                // TODO just ignore the err
                Err(e) => {
                    println!("{:?} while request to {:?}", e, &p.addr);
                    continue;
                }
            };

            rst.push((p.replica_id, repl));
        }

        return rst;
    }};
}

pub async fn bcast_fast_accept(
    peers: &Vec<ReplicaPeer>,
    inst: &Instance,
    deps_committed: &[bool],
) -> Vec<(ReplicaId, Response<FastAcceptReply>)> {
    bcast_msg!(
        peers,
        |rid| MakeRequest::fast_accept(rid, inst, deps_committed),
        fast_accept
    );
}

pub async fn bcast_accept(
    peers: &Vec<ReplicaPeer>,
    inst: &Instance,
) -> Vec<(ReplicaId, Response<AcceptReply>)> {
    bcast_msg!(peers, |rid| MakeRequest::accept(rid, inst), accept);
}

pub async fn bcast_commit(
    peers: &Vec<ReplicaPeer>,
    inst: &Instance,
) -> Vec<(ReplicaId, Response<CommitReply>)> {
    bcast_msg!(peers, |rid| MakeRequest::commit(rid, inst), commit);
}

pub async fn bcast_prepare(
    peers: &Vec<ReplicaPeer>,
    inst: &Instance,
) -> Vec<(ReplicaId, Response<PrepareReply>)> {
    bcast_msg!(peers, |rid| MakeRequest::prepare(rid, inst), prepare);
}
