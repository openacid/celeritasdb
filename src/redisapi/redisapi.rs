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
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use epaxos::qpaxos::Command;
use epaxos::qpaxos::Instance;
use epaxos::qpaxos::OpCode;
use epaxos::replica::ReplicaPeer;
use epaxos::replicate;
use epaxos::ServerData;
use storage::AccessInstance;

use crate::RedisApiError;
use parse::Response;

/// ReidsApi impl redis-protocol
#[derive(Clone)]
pub struct RedisApi {
    pub server_data: Arc<ServerData>,
    pub commit_sender: mpsc::Sender<(Vec<ReplicaPeer>, Instance)>,
}

impl RedisApi {
    pub async fn serve_with_shutdown<F>(
        self,
        addr: SocketAddr,
        signal: F,
    ) -> Result<(), std::io::Error>
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

        info!("redis api listened: {}", addr);
        loop {
            tokio::select! {
                _v = (&mut sig) => {
                    drop(self.commit_sender);
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

        info!("RedisApi stopped");
        Ok(())
    }

    async fn handle_new_conn(mut self, mut sock: TcpStream) {
        info!("new connection");

        loop {
            let mut buf = vec![0u8; 1024];

            let n = sock
                .read(&mut buf)
                .await
                .expect("failed to read data from socket");

            info!("read buf: len={:}, {:?}", n, buf);

            if n == 0 {
                warn!("client closed");
                return;
            }

            let v = redis::parse_redis_value(&buf);
            let v = match v {
                Ok(q) => {
                    info!("parsed redis value: {:?}", q);
                    q
                }
                Err(err) => {
                    // TODO bad protocol handling
                    error!("redis parse error: {:}", err);
                    panic!("bad redis protocol");
                }
            };
            let r = match self.exec_redis_cmd(v).await {
                Ok(r) => r,
                Err(e) => Response::Error(format!("exec redis cmd error: {:?}", e)),
            };
            info!("exec_redis_cmd r={:?}", &r);
            let r = r.to_vec();
            info!("response bytes:{:?}", &r);
            sock.write_all(&r)
                .await
                .expect("failed to write data to socket");
        }
    }

    async fn exec_redis_cmd(&mut self, v: redis::Value) -> Result<Response, RedisApiError> {
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
                error!("tok0 is not a Data!!!");
                return Ok(Response::Error("invalid command".to_owned()));
            }
        };

        info!("instruction: {:?}", t);
        let tok0str = from_utf8(&t).unwrap();

        // execute the command

        let r = match &tok0str.to_uppercase()[..] {
            "SET" => self.cmd_set(&tokens).await,
            "FLUSHDB" => Ok(Response::Status("OK".to_owned())),
            "GET" => self.cmd_get(&tokens).await,
            _ => Ok(Response::Error("invalid command".to_owned())),
        };

        r
    }

    /// cmd_set impl redis-command set. TODO impl it.
    async fn cmd_set(&mut self, tokens: &[redis::Value]) -> Result<Response, RedisApiError> {
        let cmd = OpCode::Set;
        let key = match tokens[1] {
            redis::Value::Data(ref d) => d,
            _ => {
                error!("expect tokens[1] to be key but not a Data");
                return Ok(Response::Error("invalid key".to_owned()));
            }
        };
        let value = match tokens[2] {
            redis::Value::Data(ref d) => d,
            _ => {
                error!("expect tokens[2] to be value but not a Data");
                return Ok(Response::Error("invalid value".to_owned()));
            }
        };

        let cmd = Command::from((cmd, key as &[u8], value as &[u8]));
        let cmds = vec![cmd];

        let (g, r) = self.server_data.get_local_replica_for_key(key)?;
        let mut st = replicate(&cmds, g, r).await?;

        let inst = &mut st.instance;
        inst.committed = true;
        let _ = r.storage.set_instance(inst)?;

        if let Err(err) = self
            .commit_sender
            .send((r.peers.clone(), st.instance))
            .await
        {
            error!("send commit msg error: {:}", err);
        }

        Ok(Response::Status("OK".to_owned()))
    }

    /// cmd_set impl redis-command set. TODO impl it.
    async fn cmd_get(&mut self, tokens: &[redis::Value]) -> Result<Response, RedisApiError> {
        let cmd = OpCode::Get;
        let key = match tokens[1] {
            redis::Value::Data(ref d) => d,
            _ => {
                error!("expect tokens[1] to be key but not a Data");
                return Ok(Response::Error("invalid key".to_owned()));
            }
        };

        let cmd = Command::from((cmd, key as &[u8], &vec![][..]));
        let cmds = vec![cmd];

        let (g, r) = self.server_data.get_local_replica_for_key(key)?;
        let mut st = replicate(&cmds, g, r).await?;

        let inst = &mut st.instance;
        inst.committed = true;
        let (tx, rx) = oneshot::channel();
        r.insert_tx(inst.instance_id.unwrap(), tx).await;
        r.storage.set_instance(inst)?;

        if let Err(err) = self
            .commit_sender
            .send((r.peers.clone(), st.instance))
            .await
        {
            error!("send commit msg error: {:}", err);
        }

        let repl = rx.await?.pop().unwrap_or(None);
        if let Some(v) = repl {
            return Ok(Response::Data(v));
        }

        Ok(Response::Nil)
    }
}
