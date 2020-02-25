use yaml_rust;

use std::collections::HashMap;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use yaml_rust::{yaml::Array, yaml::Yaml, YamlLoader};

#[cfg(test)]
#[path = "./tests/conf_tests.rs"]
mod tests;

pub type NodeID = String;

/// a struct to represent a cluster node, not necessary a replica
pub struct NodeInfo {
    pub node_id: NodeID,
    pub req_addr: SocketAddr,
    pub req_uaddr: Option<String>,
    pub dup_addr: SocketAddr,
    // idc: String, // TODO(lsl): need topology information of a node
}

pub struct ClusterInfo {
    pub nodes: HashMap<String, NodeInfo>,
}

impl ClusterInfo {
    pub fn new(path: &PathBuf) -> Result<ClusterInfo, String> {
        // read from configuration file to load conf
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => return Err(e.to_string()),
        };

        let yamls = match YamlLoader::load_from_str(&content) {
            Ok(y) => y,
            Err(e) => return Err(e.to_string()),
        };

        return Ok(ClusterInfo {
            nodes: match ClusterInfo::parse_nodes(&yamls[0]) {
                Ok(n) => n,
                Err(err_msg) => return Err(err_msg),
            },
        });
    }

    fn parse_nodes(raw: &Yaml) -> Result<HashMap<String, NodeInfo>, String> {
        let mut nodes = HashMap::new();
        let nodes_vec = match raw["nodes"].as_vec() {
            None => return Err(String::from("no nodes info found")),
            Some(v) => v,
        };

        for node in nodes_vec {
            let node_hash = match node.as_hash() {
                None => return Err(String::from("invalid node hash")),
                Some(h) => h,
            };

            let req_addr = match node_hash.get(&Yaml::String("req".to_string())) {
                None => return Err(String::from("req address is required")),
                Some(r) => match r.as_vec() {
                    None => return Err(String::from("invalid req socket address")),
                    Some(v) => v,
                },
            };

            let req_sa = match ClusterInfo::parse_socket_addr(req_addr) {
                Ok(v) => v,
                Err(e) => return Err(e.to_string()),
            };

            let req_ua = match node_hash.get(&Yaml::String("req_unix".to_string())) {
                None => None,
                Some(v) => match v.as_str() {
                    None => return Err(format!("invalid req unix socket: {:?}", v)),
                    Some(v) => Some(String::from(v)),
                },
            };

            let dup_addr = match node_hash.get(&Yaml::String("dup".to_string())) {
                None => return Err(String::from("dup socket address is required")),
                Some(r) => match r.as_vec() {
                    None => return Err(String::from("invalid dup socket address")),
                    Some(v) => v,
                },
            };

            let dup_sa = match ClusterInfo::parse_socket_addr(dup_addr) {
                Ok(v) => v,
                Err(e) => return Err(e.to_string()),
            };

            let key = req_sa.to_string();
            let info = NodeInfo {
                node_id: ClusterInfo::make_node_id(""),
                req_addr: req_sa,
                req_uaddr: req_ua,
                dup_addr: dup_sa,
            };

            nodes.insert(key, info);
        }

        return Ok(nodes);
    }

    fn parse_socket_addr(addr: &Array) -> Result<SocketAddr, String> {
        let ip = match addr[0].as_str() {
            None => return Err(format!("invalid ip addr: {:?}", addr)),
            Some(ip) => ip,
        };
        let port = match addr[1].as_i64() {
            None => return Err(format!("invalid port: {:?}", addr)),
            Some(port) => port as u16,
        };

        return Ok(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::from_str(ip).unwrap()),
            port,
        ));
    }

    // make a node id from key, i.e. mac address
    pub fn make_node_id(_: &str) -> NodeID {
        // TODO: make sure the way to make node id
        String::from("")
    }

    pub fn get_node(&self, ip: &str, port: u16) -> Option<&NodeInfo> {
        self.nodes.get(&format!("{}:{}", ip, port))
    }
}
