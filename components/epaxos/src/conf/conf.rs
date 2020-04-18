use std::collections::BTreeMap;
use std::fs;
use std::net::AddrParseError;
use std::net::SocketAddr;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use crate::conf::ConfError;
use crate::qpaxos::ReplicaId;

use serde::{Deserialize, Serialize};

/// NodeId is the global identity of a service.
/// A physical server could have several node on it.
/// A node has one or more Replica it serves for.
pub type NodeId = String;

/// Node is a struct to represent a cluster node, not necessary a replica.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Node {
    #[serde(default)]
    pub node_id: NodeId,
    pub api_addr: SocketAddr,
    pub api_uaddr: Option<String>,
    pub replication: SocketAddr,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ReplicaInfo {
    pub group_idx: usize,
    pub node_id: NodeId,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct GroupInfo {
    /// range defines the starting and ending key this group serves.
    /// It is a left-close right-open range.
    pub range: (String, String),
    pub replicas: BTreeMap<ReplicaId, NodeId>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ClusterInfo {
    /// The key is NodeId and should be unique globally.
    /// And when parsing cluster conf yaml, it tries to convert the key:NodeId to replication addr.
    /// In this case a user does not need to fill in `replication`.
    //
    // TODO: graceful handling replication addr in `key`: e.g. only when replication is None, or
    // make `replication` an vector.
    pub nodes: BTreeMap<String, Node>,

    /// groups defines the replication-groups in this cluster.
    /// Every group has about 3 replicas, and every replica is assigned to one node.
    /// No two groups have the same replica id.
    pub groups: Vec<GroupInfo>,

    #[serde(skip)]
    pub replicas: BTreeMap<ReplicaId, ReplicaInfo>,
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
        Self::from_str(content.as_str())
    }

    pub fn from_str(yaml: &str) -> Result<ClusterInfo, ConfError> {
        let mut cluster: ClusterInfo = serde_yaml::from_str(yaml)?;

        for (nid, node) in cluster.nodes.iter_mut() {
            ClusterInfo::norm_node(nid, node)?;
        }

        cluster.check_group()?;

        cluster.populate_replicas()?;

        cluster.check_replicas()?;

        return Ok(cluster);
    }

    /// get_replica returns the ReplicaInfo by specified replica-id.
    pub fn get_replica(&self, rid: ReplicaId) -> Option<&ReplicaInfo> {
        let rinfo = self.replicas.get(&rid)?;
        Some(rinfo)
    }

    /// get_replica_node returns the Node where the specified replica is.
    pub fn get_replica_node(&self, rid: ReplicaId) -> Option<&Node> {
        let rinfo = self.replicas.get(&rid)?;
        let nid = &rinfo.node_id;
        self.nodes.get(nid)
    }

    /// get_group_for_key returns the GroupInfo of which the range covers the specified key.
    pub fn get_group_for_key(&self, key: &str) -> Option<&GroupInfo> {
        for g in self.groups.iter() {
            if g.range.0.as_str() <= key && g.range.1.as_str() > key {
                return Some(g);
            }
        }

        None
    }

    /// get_group returns the GroupInfo where the specified replica in.
    pub fn get_group(&self, rid: ReplicaId) -> Option<&GroupInfo> {
        let rinfo = self.replicas.get(&rid)?;
        Some(&self.groups[rinfo.group_idx])
    }

    // TODO test bad node id as replication addr
    // make a node id from key, i.e. mac address
    pub fn norm_node(nid: &str, node: &mut Node) -> Result<(), AddrParseError> {
        node.node_id = String::from(nid);
        node.replication = nid.parse()?;
        Ok(())
    }

    pub fn check_group(&self) -> Result<(), ConfError> {
        if self.groups.len() == 0 {
            return Ok(());
        }
        for g in self.groups.iter() {
            let a = &g.range.0;
            let b = &g.range.1;
            if a >= b {
                return Err(ConfError::GroupOutOfOrder(a.clone(), b.clone()));
            }
        }

        for i in 0..self.groups.len() - 1 {
            let x = &self.groups[i];
            let y = &self.groups[i + 1];

            let a = &x.range.1;
            let b = &y.range.0;
            if a > b {
                return Err(ConfError::GroupOutOfOrder(a.clone(), b.clone()));
            }
        }

        Ok(())
    }

    pub fn populate_replicas(&mut self) -> Result<(), ConfError> {
        self.replicas = BTreeMap::new();

        for (gidx, g) in self.groups.iter().enumerate() {
            for (rid, _) in g.replicas.iter() {
                if self.replicas.contains_key(rid) {
                    return Err(ConfError::DupReplica(*rid));
                }
            }

            for (rid, nid) in g.replicas.iter() {
                self.replicas.insert(
                    *rid,
                    ReplicaInfo {
                        group_idx: gidx,
                        node_id: nid.clone(),
                    },
                );
            }
        }

        Ok(())
    }

    /// check_replicas checks whether there is a replica on a unknown node.
    pub fn check_replicas(&self) -> Result<(), ConfError> {
        for (rid, rinfo) in self.replicas.iter() {
            let nid = &rinfo.node_id;
            if !self.nodes.contains_key(nid) {
                return Err(ConfError::OrphanReplica(*rid, nid.clone()));
            }
        }
        Ok(())
    }
}
