use crate::conf::ClusterInfo;
use crate::conf::GroupInfo;
use crate::conf::Node;
use crate::conf::NodeId;
use crate::qpaxos::ReplicaID;
use crate::replica::Replica;
use crate::snapshot::MemEngine;
use crate::snapshot::Storage;
use crate::RangeLookupError;
use std::collections::BTreeMap;
use std::sync::Arc;

/// ServerData is shared between threads or coroutine.
/// TODO: Storage does not need to be shared with Arc any more.
// #[derive(Debug)]
pub struct ServerData {
    pub cluster: ClusterInfo,
    pub node_id: NodeId,
    pub node: Node,
    pub local_replicas: BTreeMap<ReplicaID, Replica>,
    pub storage: Storage,
}

impl Default for ServerData {
    /// default creates a damn simple cluster with only one node, one group and one replica.
    fn default() -> Self {
        let yaml = "
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
";
        let ci = ClusterInfo::from_str(yaml).unwrap();
        let sto = MemEngine::new().unwrap();
        let sto = Arc::new(sto);
        let node_id = "127.0.0.1:4441";
        ServerData::new(sto, ci, node_id.into())
    }
}

impl ServerData {
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
