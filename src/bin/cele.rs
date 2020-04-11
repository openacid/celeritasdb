// TODO rename this file, choose a better bin name

use clap::{App, Arg};

use net2;
use redis;
use std::i64;

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::str::from_utf8;
use std::sync::Arc;

use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tonic;

use epaxos::conf::ClusterInfo;
use epaxos::conf::GroupInfo;
use epaxos::conf::Node;
use epaxos::conf::NodeId;
use epaxos::qpaxos::Command;
use epaxos::qpaxos::Instance;
use epaxos::qpaxos::InstanceId;
use epaxos::qpaxos::MyQPaxos;
use epaxos::qpaxos::OpCode;
use epaxos::qpaxos::QPaxosServer;
use epaxos::qpaxos::ReplicaID;
use epaxos::replica::bcast_accept;
use epaxos::replica::bcast_fast_accept;
use epaxos::replica::Replica;
use epaxos::replica::ReplicaConf;
use epaxos::replica::ReplicaPeer;
use epaxos::replica::Status;
use epaxos::replication::handle_accept_reply;
use epaxos::replication::handle_fast_accept_reply;
use epaxos::snapshot::MemEngine;
use epaxos::snapshot::Storage;

use parse::Response;

/// RangeLookupError defines all error occurs at server level.
/// It also wraps lower level errors.
pub enum RangeLookupError {
    NoGroupForKey(String),
    NoLocalReplicaForKey(String),
}

impl From<RangeLookupError> for Response {
    fn from(e: RangeLookupError) -> Response {
        match e {
            RangeLookupError::NoGroupForKey(k) => Response::Error(format!("No gruop serves: {}", k)),
            RangeLookupError::NoLocalReplicaForKey(k) => {
                Response::Error(format!("No replica serve: {}", k))
            }
        }
    }
}

/// ServerData is shared between threads or coroutine.
/// TODO: Storage does not need to be shared with Arc any more.
pub struct ServerData {
    cluster: ClusterInfo,
    node_id: NodeId,
    node: Node,
    local_replicas: BTreeMap<ReplicaID, Replica>,
    storage: Storage,
}

impl ServerData {
    pub fn new(sto: Storage, cluster: ClusterInfo, node_id: NodeId) -> ServerData {
        let n = cluster.get(&node_id).unwrap().clone();

        let mut rs = BTreeMap::new();
        for (rid, rinfo) in cluster.replicas.iter() {
            if rinfo.node_id == node_id {
                let gidx = rinfo.group_idx;
                let g = &cluster.groups[gidx];
                rs.insert(
                    *rid,
                    Replica {
                        replica_id: *rid,
                        group_replica_ids: g.replicas.keys().cloned().collect(),
                        peers: vec![], // TODO
                        conf: ReplicaConf {
                            dreply: false,
                            inst_committed_timeout: 100000,
                        },
                        storage: sto.clone(),
                    },
                );
            }
        }

        ServerData {
            cluster: cluster,
            node_id: node_id.clone(),
            node: n,
            local_replicas: rs,
            storage: sto,
        }
    }

    pub fn get_local_replica_for_key(
        &self,
        key: &[u8],
    ) -> Result<(&GroupInfo, &Replica), RangeLookupError> {
        let k = String::from_utf8(key.to_vec()).unwrap();

        let g = self
            .cluster
            .get_group_for_key(&k)
            .ok_or(RangeLookupError::NoGroupForKey(k.clone()))?;

        for (rid, _) in g.replicas.iter() {
            let replica = self.local_replicas.get(rid);
            if let Some(v) = replica {
                return Ok((g, v));
            }
        }

        Err(RangeLookupError::NoLocalReplicaForKey(k.clone()))
    }
}

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
    /// start_servers(10);
    /// ```
    #[tokio::main]
    async fn start_servers(&mut self, tcp_backlog: i32) -> Result<(), Box<dyn std::error::Error>> {
        let sd = &self.server_data;
        let api_addr = sd.node.api_addr;
        let repl_addr = sd.node.replication;

        let builder = net2::TcpBuilder::new_v4().unwrap();
        builder.reuse_address(true).unwrap();
        let lis = builder.bind(api_addr).unwrap().listen(tcp_backlog).unwrap();

        let listener1 = TcpListener::from_std(lis).unwrap();

        println!("api listened: {}", api_addr);

        let redisapi = RedisApi {
            server_data: sd.clone(),
        };

        let j1 = tokio::spawn(async move {
            redisapi.api_loop(listener1).await;
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

/// ReidsApi impl redis-protocol
#[derive(Clone)]
struct RedisApi {
    server_data: Arc<ServerData>,
}

impl RedisApi {
    async fn api_loop(self, mut listener: TcpListener) {
        println!("api_loop start");
        loop {
            let (mut sock, _) = listener.accept().await.expect("accept failure");
            println!("new connection");

            let slf = self.clone();

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
                    let r = slf.exec_redis_cmd(v).await;
                    println!("r={:?}", r);
                    println!("response bytes:{:?}", r.as_bytes());
                    sock.write_all(&*r.as_bytes())
                        .await
                        .expect("failed to write data to socket");
                }
            });
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

        let cmd = Command::of(cmd, key, value);

        let (g, r) = self.server_data.get_local_replica_for_key(key)?;

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

    server.start_servers(10).unwrap();
}
