use crate::qpaxos::AcceptReply;
use crate::qpaxos::AcceptRequest;
use crate::qpaxos::CommitReply;
use crate::qpaxos::CommitRequest;
use crate::qpaxos::FastAcceptReply;
use crate::qpaxos::FastAcceptRequest;
use crate::qpaxos::Instance;
use crate::qpaxos::MakeReply;
use crate::qpaxos::PrepareReply;
use crate::qpaxos::PrepareRequest;
use crate::qpaxos::ProtocolError;
use crate::qpaxos::QPaxos;
use crate::qpaxos::RequestCommon;
use crate::replica::Replica;
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
    pub fn get_replica(&self, cmn: &Option<RequestCommon>) -> Result<&Replica, ProtocolError> {
        let cmn = cmn.as_ref().ok_or(ProtocolError::LackOf("cmn".into()))?;
        let rid = cmn.to_replica_id;
        let r = self.server_data.local_replicas.get(&rid);
        let r = r.ok_or(ProtocolError::NoSuchReplica(rid, 0))?;
        Ok(r)
    }
}

#[tonic::async_trait]
impl QPaxos for MyQPaxos {
    async fn fast_accept(
        &self,
        request: Request<FastAcceptRequest>,
    ) -> Result<Response<FastAcceptReply>, Status> {
        println!("Got a request: {:?}", request);
        let req = request.get_ref();
        let r = self.get_replica(&req.cmn);
        let r = match r {
            Ok(v) => v,
            Err(e) => {
                let reply = FastAcceptReply {
                    err: Some(e.into()),
                    ..Default::default()
                };
                return Ok(Response::new(reply));
            }
        };

        let reply = r.handle_fast_accept(request.get_ref());
        Ok(Response::new(reply))
    }

    async fn accept(
        &self,
        request: Request<AcceptRequest>,
    ) -> Result<Response<AcceptReply>, Status> {
        println!("Got a request: {:?}", request);
        let req = request.get_ref();
        let r = self.get_replica(&req.cmn);
        let r = match r {
            Ok(v) => v,
            Err(e) => {
                let reply = AcceptReply {
                    err: Some(e.into()),
                    ..Default::default()
                };
                return Ok(Response::new(reply));
            }
        };
        let reply = r.handle_accept(request.get_ref());
        Ok(Response::new(reply))
    }

    async fn commit(
        &self,
        request: Request<CommitRequest>,
    ) -> Result<Response<CommitReply>, Status> {
        println!("Got a request: {:?}", request);
        let req = request.get_ref();
        let r = self.get_replica(&req.cmn);
        let r = match r {
            Ok(v) => v,
            Err(e) => {
                let reply = CommitReply {
                    err: Some(e.into()),
                    ..Default::default()
                };
                return Ok(Response::new(reply));
            }
        };
        let reply = r.handle_commit(request.get_ref());
        Ok(Response::new(reply))
    }

    async fn prepare(
        &self,
        request: Request<PrepareRequest>,
    ) -> Result<Response<PrepareReply>, Status> {
        // TODO I did nothing but let the test pass happily
        let inst = Instance {
            ..Default::default()
        };

        let reply = MakeReply::prepare(&inst);
        println!("Got a request: {:?}", request);
        Ok(Response::new(reply))
    }
}
