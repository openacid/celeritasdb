use std::collections::BTreeMap;
use std::fs;
use std::net::AddrParseError;
use std::net::SocketAddr;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use super::errors::ConfError;
use crate::qpaxos::ReplicaID;

use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "./tests/conf_tests.rs"]
mod tests;

/// NodeID is the global identity of a service.
/// A physical server could have several node on it.
/// A node has one or more Replica it serves for.
pub type NodeID = String;

/// Node is a struct to represent a cluster node, not necessary a replica.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Node {
    #[serde(default)]
    pub node_id: NodeID,
    pub api_addr: SocketAddr,
    pub api_uaddr: Option<String>,
    pub replication: SocketAddr,
    // idc: String, // TODO(lsl): need topology information of a node
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClusterInfo {
    /// The key is NodeID and should be unique globally.
    /// And when parsing cluster conf yaml, it tries to convert the key:NodeID to replication addr.
    /// In this case a user does not need to fill in `replication`.
    //
    // TODO: graceful handling replication addr in `key`: e.g. only when replication is None, or
    // make `replication` an vector.
    pub nodes: BTreeMap<String, Node>,

    pub replicas: BTreeMap<ReplicaID, NodeID>,
}

// let user to use c.get() just like c.nodes.get()
impl Deref for ClusterInfo {
    type Target = BTreeMap<String, Node>;
    fn deref(&self) -> &Self::Target {
        &self.nodes
    }
}

// let user to use c.get() just like c.nodes.get()
impl DerefMut for ClusterInfo {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.nodes
    }
}

impl ClusterInfo {
    /// from_file read cluster conf yaml from a local file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<ClusterInfo, ConfError> {
        let content = fs::read_to_string(path)?;
        let mut cluster: ClusterInfo = serde_yaml::from_str(content.as_str())?;

        for (nid, node) in cluster.nodes.iter_mut() {
            ClusterInfo::norm_node(nid, node)?;
        }

        cluster.check_replicas()?;

        return Ok(cluster);
    }

    /// get_replica_node returns the Node where the specified replica is.
    pub fn get_replica_node(&self, rid: ReplicaID) -> Option<&Node> {
        let nid = self.replicas.get(&rid)?;
        self.nodes.get(nid)
    }

    // TODO test bad node id as replication addr
    // make a node id from key, i.e. mac address
    pub fn norm_node(nid: &str, node: &mut Node) -> Result<(), AddrParseError> {
        node.node_id = String::from(nid);
        node.replication = nid.parse()?;
        Ok(())
    }

    /// check_replicas checks whether there is a replica on a unknown node.
    pub fn check_replicas(&self) -> Result<(), ConfError> {
        for (rid, nid) in self.replicas.iter() {
            if !self.nodes.contains_key(nid) {
                return Err(ConfError::OrphanReplica(*rid, nid.clone()));
            }
        }
        Ok(())
    }
}
