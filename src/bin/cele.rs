// TODO rename this file, choose a better bin name

use clap::{App, Arg};

use net2;
use redis;

use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::str::from_utf8;

use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tonic;

use epaxos::conf::ClusterInfo;
use epaxos::conf::Node;
use epaxos::conf::NodeId;
use epaxos::qpaxos::*;

use parse::Response;

pub struct Server {
    cluster: ClusterInfo,
    node_id: NodeId,
    node: Node,
    _join_handles: Vec<JoinHandle<()>>,
}

impl Server {
    pub fn new(cluster: ClusterInfo, node_id: NodeId) -> Server {
        let n = cluster.get(&node_id).unwrap().clone();
        Server {
            cluster: cluster,
            node_id: node_id.clone(),
            node: n,
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
    /// start_servers(10);
    /// ```
    #[tokio::main]
    async fn start_servers(&mut self, tcp_backlog: i32) -> Result<(), Box<dyn std::error::Error>> {
        let api_addr = self.node.api_addr;
        let repl_addr = self.node.replication;

        let builder = net2::TcpBuilder::new_v4().unwrap();
        builder.reuse_address(true).unwrap();
        let lis = builder.bind(api_addr).unwrap().listen(tcp_backlog).unwrap();

        let listener1 = TcpListener::from_std(lis).unwrap();

        println!("api listened: {}", api_addr);

        let j1 = tokio::spawn(async move {
            api_loop(listener1).await;
        });

        println!("serving: {}", api_addr);

        let qp = MyQPaxos::default();
        let s = tonic::transport::Server::builder().add_service(QPaxosServer::new(qp));

        let j2 = tokio::spawn(async move {
            println!("repl server spawned");
            s.serve(repl_addr).await.unwrap();
        });

        println!("serving: {}", repl_addr);

        j1.await.unwrap();
        j2.await.unwrap();

        Ok(())
    }
}

fn exec_redis_cmd(v: redis::Value) -> Option<Response> {
    // cmd is a nested array: ["set", "a", "1"] or ["set", ["b", "c"], ...]
    // A "set" or "get" redis command is serialized as non-nested array.
    //
    // Flatten one level:
    // tokens is a vec[Value].
    let tokens = match v {
        redis::Value::Bulk(tokens) => tokens,
        _ => vec![],
    };

    // the first token is instruction, e.g. "set" or "get".
    let tok0 = &tokens[0];

    let t = match tok0 {
        redis::Value::Data(d) => d,
        _ => {
            println!("tok0 is not a Data!!!");
            return Some(Response::Error("invalid command".to_owned()));
        }
    };

    println!("instruction: {:?}", t);
    let tok0str = from_utf8(&t).unwrap();

    // execute the command

    match tok0str {
        "FLUSHDB" => Some(Response::Status("OK".to_owned())),
        "SET" => Some(Response::Status("OK".to_owned())),
        "GET" => Some(Response::Integer(42)),
        _ => Some(Response::Error("invalid command".to_owned())),
    }
}

async fn api_loop(mut listener: TcpListener) {
    println!("api_loop start");
    loop {
        let (mut sock, _) = listener.accept().await.expect("accept failure");
        println!("new connection");

        tokio::spawn(async move {
            loop {
                let mut buf = vec![0u8; 1024];

                let n = sock
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                println!("read buf: len={:}, {:?}", n, buf);

                if n == 0 {
                    println!("client closed");
                    return;
                }

                let v = redis::parse_redis_value(&buf);
                let v = match v {
                    Ok(q) => {
                        println!("parsed redis value: {:?}", q);
                        q
                    }
                    Err(err) => {
                        // TODO bad protocol handling
                        println!("parse error: {:}", err);
                        panic!("bad redis protocol");
                    }
                };
                let r = exec_redis_cmd(v);
                println!("r={:?}", r);
                match r {
                    // received a response, send it to the client
                    Some(response) => {
                        println!("response bytes:{:?}", response.as_bytes());
                        sock.write_all(&*response.as_bytes())
                            .await
                            .expect("failed to write data to socket");
                    }
                    None => {
                        // TODO
                    }
                };
            }
        });
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

    let cluster = ClusterInfo::from_file(conffn).unwrap();
    let mut server = Server::new(cluster, node_id.into());
    server.start_servers(10).unwrap();
}
