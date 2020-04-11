use crate::RangeLookupError;
use crate::conf::ClusterInfo;
use crate::conf::GroupInfo;
use crate::conf::Node;
use crate::conf::NodeId;
use crate::qpaxos::ReplicaID;
use crate::replica::Replica;
use crate::replica::ReplicaConf;
use crate::snapshot::Storage;
use std::collections::BTreeMap;

/// ServerData is shared between threads or coroutine.
/// TODO: Storage does not need to be shared with Arc any more.
pub struct ServerData {
    pub cluster: ClusterInfo,
    pub node_id: NodeId,
    pub node: Node,
    pub local_replicas: BTreeMap<ReplicaID, Replica>,
    pub storage: Storage,
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
