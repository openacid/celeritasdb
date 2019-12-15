use std::net::SocketAddr;

struct NodeInfo {
    replica: ReplicaID, // -1 if not a epaxos replica
    addr: SocketAddr,
    idc: String,
}

struct ClusterInfo {
    Replicas: Vec<NodeInfo>,
}
