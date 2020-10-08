use crate::ClusterInfo;
use crate::NodeId;
use crate::ServerData;
use std::collections::BTreeMap;
use std::sync::Arc;
use storage::{MemEngine, Storage};

lazy_static! {
    /// LOCAL_CLUSTERS predefines several single-node cluster for testing.
    static ref LOCAL_CLUSTERS: BTreeMap<&'static str, &'static str> = {
        let mut h = BTreeMap::new();
        h.insert("az_1", "
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

        h.insert("az_3", "
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

/// new_cluster creates a ClusterInfo with predefined config specified by `name`.
/// Such a cluster is only meant for test.
/// Available names are:
/// az_1: to create a cluster with 1 group of replica 1 covers key from `[a, z)`.
/// az_3: to create a cluster with 1 group of replica 1, 2, 3 covers key from `[a, z)`.
pub fn new_cluster(name: &str) -> ClusterInfo {
    let yaml = LOCAL_CLUSTERS[name];
    ClusterInfo::from_str(yaml).unwrap()
}

/// new_inmem_server_data creates a ServerData with predefined config specified by `name`. See
/// ClusterInfo::new_predefined.
///
/// Such a cluster is only meant for test because it use a in-memory storage.
pub fn new_inmem_server_data(name: &str) -> ServerData {
    let ci = new_cluster(name);

    let sto = MemEngine::new().unwrap();
    let sto = Arc::new(sto);
    let sto = Storage::new(sto);

    let node_ids: Vec<NodeId> = ci.nodes.keys().cloned().collect();
    let node_id: NodeId = node_ids[0].clone();

    ServerData::new(sto, ci, node_id)
}
