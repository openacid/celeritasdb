use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::qpaxos::*;
use crate::replica::{Replica, ReplicaPeer};
use crate::MyQPaxos;
use crate::Storage;
use storage::MemEngine;

use tokio::sync::oneshot;
use tokio::time::delay_for;
use tonic::transport::Server;

/// Create an instance with command "set x=y".
/// Use this when only deps are concerned.
/// The initial_deps and deps are all set to the second arg.
/// supported pattern:
/// foo_inst!(iid, cmds, initial_deps)
/// foo_inst!(iid, key, initial_deps) // an instance with a single cmd: Set `key`
/// foo_inst!(iid, initial_deps)
/// foo_inst!(None, initial_deps)
/// foo_inst!(iid)
#[allow(unused_macros)]
macro_rules! foo_inst {
    ($id:expr,
     [$( ($op:expr, $key:expr, $val:expr)),*],
     [$(($dep_rid:expr, $dep_idx:expr)),* $(,)*]
    ) => {
        inst!($id, (0, 0, _),
              [$( ($op, $key, $val)),*],
              [$(($dep_rid, $dep_idx)),*],
              "withdeps"
        )
    };

    ($id:expr,
     $key:expr,
     [$(($dep_rid:expr, $dep_idx:expr)),* $(,)*]
    ) => {
        inst!($id, (0, 0, _),
              [("Set", $key, $key)],
              [$(($dep_rid, $dep_idx)),*],
              "withdeps"
        )
    };

    (None,
     [$(($dep_rid:expr, $dep_idx:expr)),* $(,)*]
    ) => {
        Instance {
            instance_id: None,
            ..inst!((0, 0), (0, 0, _),
                      [("Set", "x", "y")],
                      [$(($dep_rid, $dep_idx)),*],
                      "withdeps"
                     )
        }
    };

    ($id:expr,
     [$(($dep_rid:expr, $dep_idx:expr)),* $(,)*]
    ) => {
        inst!($id, (0, 0, _),
              [("Set", "x", "y")],
              [$(($dep_rid, $dep_idx)),*],
              "withdeps"
        )
    };

    ($id:expr
    ) => {
        inst!($id, (0, 0, _),
              [("Set", "x", "y")],
        )
    };
}

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

pub fn new_replica(
    rid: ReplicaId,
    group: Vec<ReplicaId>,
    peers: Vec<ReplicaPeer>,
    sto: Storage,
) -> Replica {
    Replica {
        replica_id: rid,
        group_replica_ids: group,
        peers,
        storage: sto,
        committed_timeout: 1000,
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

            let qp = MyQPaxos::default();
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
