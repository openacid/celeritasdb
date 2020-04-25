use crate::conf;
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

impl ServerData {
    /// new_inmem creates a ServerData with predefined config specified by `name`. See
    /// ClusterInfo::new_predefined.
    ///
    /// Such a cluster is only meant for test because it use a in-memory storage.
    pub fn new_inmem(name: &str) -> ServerData {
        let ci = ClusterInfo::new_predefined(name);

        let sto = MemEngine::new().unwrap();
        let sto = Arc::new(sto);

        let node_id = conf::LOCAL_NODE_ID;

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
