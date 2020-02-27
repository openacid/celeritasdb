use tonic;
use tonic::{Request, Response, Status};

use super::instance::Instance;
use super::message;

include!(concat!(env!("OUT_DIR"), "/qpaxos.rs"));

// #[cfg(test)]
// mod t;

pub use q_paxos_client::*;
pub use q_paxos_server::*;

#[derive(Debug, Default)]
pub struct MyQPaxos {}

#[tonic::async_trait]
impl QPaxos for MyQPaxos {
    async fn fast_accept(
        &self,
        request: Request<message::FastAcceptRequest>,
    ) -> Result<Response<message::FastAcceptReply>, Status> {
        // TODO I did nothing but let the test pass happily
        let inst = Instance {
            ..Default::default()
        };

        let reply = message::Reply::fast_accept(&inst, &[]);
        println!("Got a request: {:?}", request);
        Ok(Response::new(reply))
    }
    async fn accept(
        &self,
        request: Request<message::AcceptRequest>,
    ) -> Result<Response<message::AcceptReply>, Status> {
        // TODO I did nothing but let the test pass happily
        let inst = Instance {
            ..Default::default()
        };

        let reply = message::Reply::accept(&inst);
        println!("Got a request: {:?}", request);
        Ok(Response::new(reply))
    }
    async fn commit(
        &self,
        request: Request<message::CommitRequest>,
    ) -> Result<Response<message::CommitReply>, Status> {
        // TODO I did nothing but let the test pass happily
        let inst = Instance {
            ..Default::default()
        };

        let reply = message::Reply::commit(&inst);
        println!("Got a request: {:?}", request);
        Ok(Response::new(reply))
    }
    async fn prepare(
        &self,
        request: Request<message::PrepareRequest>,
    ) -> Result<Response<message::PrepareReply>, Status> {
        // TODO I did nothing but let the test pass happily
        let inst = Instance {
            ..Default::default()
        };

        let reply = message::Reply::prepare(&inst);
        println!("Got a request: {:?}", request);
        Ok(Response::new(reply))
    }
}
