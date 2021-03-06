use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use crate::qpaxos::*;
use crate::replica::{Replica, ReplicaPeer};
use crate::QPaxosImpl;
use storage::MemEngine;
use storage::RawKV;
use storage::Storage;

use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tokio::time::delay_for;
use tonic::transport::Server;

#[path = "testutil_cluster.rs"]
mod testutil_cluster;

pub use testutil_cluster::*;

#[allow(unused_macros)]
macro_rules! test_enc_dec {
    // $msg is a prost Message.
    // $typ is one of message.Request or message.Reply
    ($msg:ident, $typ:ty) => {
        let size = $msg.encoded_len();
        assert!(size > 0);

        let mut byts = vec![];
        $msg.encode(&mut byts).unwrap();
        let msg2 = <$typ>::decode(byts.as_slice()).unwrap();
        assert_eq!($msg, msg2);
    };
}

pub fn new_sto(rid: ReplicaId) -> Storage {
    let eng = MemEngine::new().unwrap();
    Storage::new(rid, Arc::new(eng))
}

pub fn new_two_sto(rid1: ReplicaId, rid2: ReplicaId) -> (Storage, Storage) {
    let eng = Arc::new(MemEngine::new().unwrap());
    (
        Storage::new(rid1, eng.clone()),
        Storage::new(rid2, eng.clone()),
    )
}

pub fn new_replica(
    rid: ReplicaId,
    group: Vec<ReplicaId>,
    peers: Vec<ReplicaPeer>,
    sto: Arc<dyn RawKV>,
) -> Replica {
    Replica {
        replica_id: rid,
        group_replica_ids: group,
        peers,
        storage: Storage::new(rid, sto),
        committed_timeout: 1000,
        waiting_replies: Mutex::new(HashMap::new()),
    }
}

pub struct TestCluster {
    pub replicas: Vec<Replica>,
    pub txs: Vec<oneshot::Sender<()>>,
    pub addrs: Vec<String>,
}

impl TestCluster {
    pub fn new(replica_cnt: i32) -> Self {
        let sto = Arc::new(MemEngine::new().unwrap());

        let mut addrs = HashMap::new();
        for i in 0..replica_cnt {
            let url = format!("http://127.0.0.1:555{}", i);
            addrs.insert(i as ReplicaId, url);
        }

        let group: Vec<ReplicaId> = addrs.iter().map(|(k, _)| k.clone()).collect();

        let mut replicas = vec![];
        for (k, _) in addrs.iter() {
            let mut ps = vec![];
            for (k1, v1) in addrs.iter() {
                if k1 == k {
                    continue;
                }
                ps.push((*k1, v1.clone(), true).into());
            }

            let rp = new_replica(*k, group.clone(), ps, sto.clone());
            replicas.push(rp);
        }

        Self {
            replicas,
            txs: vec![],
            addrs: addrs.iter().map(|(_, v)| v.clone()).collect(),
        }
    }

    pub async fn start(&mut self) {
        for addr in self.addrs.iter() {
            let (tx, rx) = oneshot::channel::<()>();

            let qp = QPaxosImpl::new(Arc::new(new_inmem_server_data("az_1")));
            let s = Server::builder().add_service(QPaxosServer::new(qp));

            // remove scheme
            // TODO standard parse the addr
            let addr = addr[7..].parse().unwrap();
            tokio::spawn(async move {
                s.serve_with_shutdown(addr, async {
                    rx.await.unwrap();
                })
                .await
                .unwrap();
            });

            println!("serving addr: {:?}", addr);

            self.txs.push(tx);
        }

        // TODO check the server available with connect
        delay_for(Duration::from_millis(1_000)).await;
    }

    pub fn stop(&mut self) {
        while let Some(tx) = self.txs.pop() {
            tx.send(()).unwrap();
        }
    }
}

impl Drop for TestCluster {
    fn drop(&mut self) {
        self.stop()
    }
}

/// wait_for wait for some state becomes valid.
/// wait_for stops is `get_state` returns Ok().
/// wait_for goes on to get new state if `get_state` returns error and
/// `is_retriable_err(err)` is `true`.
pub fn wait_for<GET, T, E, ERETRY>(mut get_state: GET, is_retriable_err: ERETRY) -> T
where
    E: fmt::Debug,
    GET: FnMut() -> Result<T, E>,
    ERETRY: Fn(&E) -> bool,
{
    let millisecond = Duration::from_millis(50);
    loop {
        let r = get_state();
        match r {
            Err(err) => {
                if is_retriable_err(&err) {
                    println!(
                        "err: {:?} while waiting. retry after {:?}",
                        err, millisecond
                    );
                    sleep(millisecond);
                } else {
                    panic!("Unexpected error: {:?}", err);
                }
            }
            Ok(x) => {
                println!("waiting done");
                return x;
            }
        }
    }
}
