use crate::qpaxos::ProtocolError;
use crate::qpaxos::QPaxos;
use crate::qpaxos::ReplicateReply;
use crate::qpaxos::ReplicateRequest;
use crate::replication::RpcHandlerError;
use crate::ServerData;
use std::sync::Arc;

use tonic;
use tonic::{Request, Response, Status};

pub struct QPaxosImpl {
    server_data: Arc<ServerData>,
}

impl QPaxosImpl {
    pub fn new(server_data: Arc<ServerData>) -> Self {
        QPaxosImpl { server_data }
    }
}

#[tonic::async_trait]
impl QPaxos for QPaxosImpl {
    async fn replicate(
        &self,
        request: Request<ReplicateRequest>,
    ) -> Result<Response<ReplicateReply>, Status> {
        let _meta = request.metadata();
        let req = request.into_inner();

        println!("Got a ReplicateRequest: {}", req);

        let reply = handle_replicate_request(self, req);
        let reply = match reply {
            Ok(v) => v,
            Err(e) => ReplicateReply {
                err: Some(e.into()),
                ..Default::default()
            },
        };
        Ok(Response::new(reply))
    }
}

pub fn handle_replicate_request(
    sv: &QPaxosImpl,
    req: ReplicateRequest,
) -> Result<ReplicateReply, RpcHandlerError> {
    // TODO test replica not found
    let rid = req.to_replica_id;
    let r = sv.server_data.local_replicas.get(&rid);
    let r = r.ok_or(ProtocolError::NoSuchReplica(rid, 0))?;

    r.handle_replicate(req)
}
