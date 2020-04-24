use crate::conf::ClusterInfo;
use crate::conf::GroupInfo;
use crate::conf::Node;
use crate::conf::NodeId;
use crate::qpaxos::ReplicaId;
use crate::replica::Replica;
use crate::RangeLookupError;
use crate::Storage;
use std::collections::BTreeMap;
use std::sync::Arc;
use storage::MemEngine;

/// LOCAL_NODE_ID is the only node id used in LOCAL_CLUSTERS.
static LOCAL_NODE_ID: &str = "127.0.0.1:4441";

lazy_static! {
    /// LOCAL_CLUSTERS predefines several single-node cluster for testing.
    static ref LOCAL_CLUSTERS: BTreeMap<&'static str, &'static str> = {
        let mut h = BTreeMap::new();
        h.insert("1", "
nodes:
    127.0.0.1:4441:
        api_addr: 127.0.0.1:6379
        replication: 127.0.0.1:4441
groups:
-   range:
    -   a
    -   z
    replicas:
        1: 127.0.0.1:4441
");

        h.insert("3", "
nodes:
    127.0.0.1:4441:
        api_addr: 127.0.0.1:6379
        replication: 127.0.0.1:4441
groups:
-   range:
    -   a
    -   z
    replicas:
        1: 127.0.0.1:4441
        2: 127.0.0.1:4441
        3: 127.0.0.1:4441
");

        h
    };
}

/// ServerData is shared between threads or coroutine.
/// TODO: Storage does not need to be shared with Arc any more.
// #[derive(Debug)]
pub struct ServerData {
    pub cluster: ClusterInfo,
    pub node_id: NodeId,
    pub node: Node,
    pub local_replicas: BTreeMap<ReplicaId, Replica>,
    pub storage: Storage,
}

// TODO a Default for ServerData is not appropriated.
// Remove it in future.
impl Default for ServerData {
    /// default creates a damn simple cluster with only one node, one group and one replica.
    fn default() -> Self {
        Self::mem_cluster("1")
    }
}

impl ServerData {
    /// mem_cluster creates a ServerData with cluster config specified by `name`.
    /// Such a cluster is only meant for test because it uses only an in-memory storage.
    pub fn mem_cluster(name: &str) -> ServerData {
        let yaml = LOCAL_CLUSTERS[name];

        let ci = ClusterInfo::from_str(yaml).unwrap();

        let sto = MemEngine::new().unwrap();
        let sto = Arc::new(sto);

        let node_id = LOCAL_NODE_ID;

        ServerData::new(sto, ci, node_id.into())
    }

    pub fn new(sto: Storage, cluster: ClusterInfo, node_id: NodeId) -> ServerData {
        let n = cluster.get(&node_id).unwrap().clone();

        let mut rs = BTreeMap::new();
        for (rid, rinfo) in cluster.replicas.iter() {
            if rinfo.node_id == node_id {
                let rp = Replica::new(*rid, &cluster, sto.clone()).unwrap();
                rs.insert(*rid, rp);
            }
        }

        ServerData {
            cluster,
            node_id,
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
