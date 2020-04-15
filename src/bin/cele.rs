// TODO rename this file, choose a better bin name

#[macro_use]
extern crate quick_error;

use clap::{App, Arg};

use std::sync::Arc;

use futures::Future;

use tokio;
use tokio::task::JoinHandle;
use tonic;

use epaxos::conf::ClusterInfo;
use epaxos::conf::NodeId;
use epaxos::qpaxos::QPaxosServer;
use epaxos::snapshot::MemEngine;
use epaxos::snapshot::Storage;
use epaxos::MyQPaxos;


use epaxos::ServerData;
use cele::redisapi::RedisApi;

/// Server impl some user protocol such as redis protocol and a replication service.
pub struct Server {
    server_data: Arc<ServerData>,
    _join_handles: Vec<JoinHandle<()>>,
}

impl Server {
    pub fn new(sto: Storage, cluster: ClusterInfo, node_id: NodeId) -> Server {
        Server {
            server_data: Arc::new(ServerData::new(sto, cluster, node_id)),
            _join_handles: Vec::new(),
        }
    }

    /// Starts service:
    ///
    /// # Arguments
    ///
    /// - `tcp_backlog`: tcp backlog.
    ///
    /// # Examples
    ///
    /// ```norun
    /// start_servers();
    /// ```
    #[tokio::main]
    async fn start_servers(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let sd = &self.server_data;
        let api_addr = sd.node.api_addr;
        let repl_addr = sd.node.replication;

        let redisapi = RedisApi {
            server_data: sd.clone(),
        };

        // TODO send signal to shutdown with _tx
        let (_tx, rx) = tokio::sync::oneshot::channel::<()>();

        let j1 = tokio::spawn(async move {
            let rst = redisapi.serve_with_shutdown(api_addr, rx).await;
            println!("RedisApi rst={:?}", rst);
        });

        println!("serving: {}", api_addr);

        let qp = MyQPaxos::default();
        let s = tonic::transport::Server::builder().add_service(QPaxosServer::new(qp));

        // TODO send signal to shutdown with _tx
        let (_tx, rx) = tokio::sync::oneshot::channel::<()>();

        let j2 = tokio::spawn(async move {
            println!("repl server spawned");
            let rst = s
                .serve_with_shutdown(repl_addr, async {
                    rx.await.ok();
                })
                .await;
            println!("replication server rst={:?}", rst);
        });

        println!("serving: {}", repl_addr);

        j1.await.unwrap();
        j2.await.unwrap();

        Ok(())
    }
}


fn main() {
    // TODO standalone version file.
    // TODO add test of command line argument.
    let matches = App::new("cele")
        .version("0.0.1")
        .author("openacid")
        .about("distributed redis")
        .arg(
            Arg::with_name("cluster")
                .long("cluster")
                .takes_value(true)
                .help("cluster config in yaml"),
        )
        .arg(
            Arg::with_name("id")
                .long("id")
                .takes_value(true)
                .help("node id for this server. It must be one key of clusterconf.nodes"),
        )
        .get_matches();

    let conffn = matches.value_of("cluster").unwrap();
    let node_id = matches.value_of("id").unwrap();

    let sto = MemEngine::new().unwrap();

    let cluster = ClusterInfo::from_file(conffn).unwrap();
    let mut server = Server::new(Arc::new(sto), cluster, node_id.into());

    server.start_servers().unwrap();
}
