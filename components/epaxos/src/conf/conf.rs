use std::collections::BTreeMap;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;

use super::errors::ConfError;

use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "./tests/conf_tests.rs"]
mod tests;

pub type NodeID = String;

/// a struct to represent a cluster node, not necessary a replica
#[derive(Serialize, Deserialize, Debug)]
pub struct NodeInfo {
    #[serde(default)]
    pub node_id: NodeID,
    pub api_addr: SocketAddr,
    pub api_uaddr: Option<String>,
    pub replication: SocketAddr,
    // idc: String, // TODO(lsl): need topology information of a node
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClusterInfo {
    pub nodes: BTreeMap<String, NodeInfo>,
}

impl ClusterInfo {
    pub fn new(path: &PathBuf) -> Result<ClusterInfo, ConfError> {
        let content = fs::read_to_string(path)?;
        let mut cluster: ClusterInfo = serde_yaml::from_str(content.as_str())?;

        for (_, node) in cluster.nodes.iter_mut() {
            node.node_id = ClusterInfo::make_node_id("");
        }

        return Ok(cluster);
    }

    // make a node id from key, i.e. mac address
    pub fn make_node_id(_: &str) -> NodeID {
        // TODO: make sure the way to make node id
        return String::from("");
    }
}
