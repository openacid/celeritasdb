use tonic::Response;

use crate::qpaxos::QPaxosClient;
use crate::qpaxos::ReplicaId;
use crate::qpaxos::ReplicateReply;
use crate::qpaxos::ReplicateRequest;
use crate::replica::ReplicaPeer;

pub async fn bcast_msg(
    peers: &[ReplicaPeer],
    req: ReplicateRequest,
) -> Vec<(ReplicaId, Response<ReplicateReply>)> {
    let mut rst = Vec::with_capacity(peers.len());
    for p in peers.iter() {
        let mut client = match QPaxosClient::connect(p.addr.clone()).await {
            Ok(c) => c,
            // TODO just ignore the err
            Err(e) => {
                println!("{:?} while connect to {:?}", e, &p.addr);
                continue;
            }
        };

        let mut r = req.clone();
        r.to_replica_id = p.replica_id;
        let repl = match client.replicate(r).await {
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
}
