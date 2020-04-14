// TODO rename this file, choose a better bin name

use clap::{App, Arg};

use net2;
use redis;

use std::net::SocketAddr;
use std::str::from_utf8;
use std::sync::Arc;

// for boxed()
use futures::future::FutureExt;

use futures::Future;

use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tonic;

use epaxos::conf::ClusterInfo;
use epaxos::conf::NodeId;
use epaxos::qpaxos::Command;
use epaxos::qpaxos::OpCode;
use epaxos::qpaxos::QPaxosServer;
use epaxos::snapshot::MemEngine;
use epaxos::snapshot::Storage;
use epaxos::MyQPaxos;

use parse::Response;

use epaxos::ServerData;

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

/// ReidsApi impl redis-protocol
#[derive(Clone)]
struct RedisApi {
    server_data: Arc<ServerData>,
}

impl RedisApi {
    async fn serve_with_shutdown<F>(self, addr: SocketAddr, signal: F) -> Result<(), std::io::Error>
    where
        F: Future + Send,
    {
        // TODO config tcp backlog?
        let backlog = 1024;

        // impl Unpin
        let mut sig = signal.boxed();

        let builder = net2::TcpBuilder::new_v4().unwrap();
        builder.reuse_address(true).unwrap();
        let lis = builder.bind(addr).unwrap().listen(backlog).unwrap();

        let mut lis = TcpListener::from_std(lis).unwrap();

        println!("redis api listened: {}", addr);
        loop {
            tokio::select! {
                _v = (&mut sig) => {
                    break;
                },
                inc = lis.accept() => {
                    let (sock, _cli_addr) = inc?;
                    let slf = self.clone();
                    tokio::spawn(async move {
                        slf.handle_new_conn(sock).await;
                    });
                }
            }
        }

        println!("RedisApi stopped");
        Ok(())
    }

    async fn handle_new_conn(self, mut sock: TcpStream) {
        println!("new connection");

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
            let r = self.exec_redis_cmd(v).await;
            println!("r={:?}", r);
            println!("response bytes:{:?}", r.as_bytes());
            sock.write_all(&*r.as_bytes())
                .await
                .expect("failed to write data to socket");
        }
    }

    async fn exec_redis_cmd(&self, v: redis::Value) -> Response {
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
                return Response::Error("invalid command".to_owned());
            }
        };

        println!("instruction: {:?}", t);
        let tok0str = from_utf8(&t).unwrap();

        // execute the command

        let r = match tok0str {
            "SET" => self.cmd_set(&tokens).await,
            "FLUSHDB" => Ok(Response::Status("OK".to_owned())),
            "GET" => Ok(Response::Integer(42)),
            _ => Err(Response::Error("invalid command".to_owned())),
        };

        match r {
            Ok(rr) => rr,
            Err(rr) => rr,
        }
    }

    /// cmd_set impl redis-command set. TODO impl it.
    async fn cmd_set(&self, tokens: &[redis::Value]) -> Result<Response, Response> {
        let cmd = OpCode::Set;
        let key = match tokens[1] {
            redis::Value::Data(ref d) => d,
            _ => {
                println!("expect tokens[1] to be key but not a Data");
                return Err(Response::Error("invalid key".to_owned()));
            }
        };
        let value = match tokens[2] {
            redis::Value::Data(ref d) => d,
            _ => {
                println!("expect tokens[2] to be value but not a Data");
                return Err(Response::Error("invalid value".to_owned()));
            }
        };

        let _cmd = Command::of(cmd, key, value);

        let (_g, _r) = self.server_data.get_local_replica_for_key(key)?;

        // TODO run fast-accept etc.
        Ok(Response::Status("OK".to_owned()))
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
