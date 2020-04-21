use crate::qpaxos::ProtocolError;
use crate::qpaxos::QPaxos;
use crate::qpaxos::ReplicateReply;
use crate::qpaxos::ReplicateRequest;
use crate::replication::RpcHandlerError;
use crate::ServerData;
use std::sync::Arc;

use tonic;
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct MyQPaxos {
    server_data: Arc<ServerData>,
}

impl MyQPaxos {
    pub fn new(server_data: Arc<ServerData>) -> Self {
        MyQPaxos { server_data }
    }
}

#[tonic::async_trait]
impl QPaxos for MyQPaxos {
    async fn replicate(
        &self,
        request: Request<ReplicateRequest>,
    ) -> Result<Response<ReplicateReply>, Status> {
        println!("Got a request: {:?}", request);

        let req = request.into_inner();

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
    sv: &MyQPaxos,
    req: ReplicateRequest,
) -> Result<ReplicateReply, RpcHandlerError> {
    // TODO test replica not found
    let rid = req.to_replica_id;
    let r = sv.server_data.local_replicas.get(&rid);
    let r = r.ok_or(ProtocolError::NoSuchReplica(rid, 0))?;

    r.handle_replicate(req)
}
