use std::net::IpAddr;
use std::collections::HashMap;

#[cfg(test)]
#[path = "./tests/conf_tests.rs"]
mod tests;

pub type NodeID = String;

/// a struct to represent a cluster node, not necessary a replica
pub struct NodeInfo {
    pub node_id: NodeID,
    pub ip: IpAddr,
    // idc: String, // TODO(lsl): need topology information of a node
}

pub struct ClusterInfo {
    // pub nodes: Vec<NodeInfo>,
    pub nodes: HashMap<NodeID, NodeInfo>,
}
