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

use epaxos::qpaxos::Command;
use epaxos::qpaxos::OpCode;
use epaxos::replicate;

use parse::Response;

use epaxos::ServerData;

/// ReidsApi impl redis-protocol
#[derive(Clone)]
pub struct RedisApi {
    pub server_data: Arc<ServerData>,
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
            println!("exec_redis_cmd r={:?}", r);
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

        let cmd = Command::of(cmd, key, value);
        let cmds = vec![cmd];

        let (g, r) = self.server_data.get_local_replica_for_key(key)?;

        let mut st = replicate(&cmds, g, r).await?;
        let inst = &mut st.instance;
        inst.committed = true;
        let rst = r.storage.set_instance(inst);

        match rst {
            Ok(_v) => {}
            Err(_e) => {
                return Ok(Response::Error("local commit error".into()));
            }
        }

        // TODO bcast commit

        Ok(Response::Status("OK".to_owned()))
    }
}
