use crate::conf::NodeId;
use crate::qpaxos::ReplicaID;
use std::net::AddrParseError;

quick_error! {
    #[derive(Debug)]
    pub enum ConfError {
        IOError(e: std::io::Error) {
            from(e: std::io::Error) -> (e)
        }

        BadYaml(e: serde_yaml::Error) {
            from(e: serde_yaml::Error) -> (e)
        }

        BadReplication(e: AddrParseError) {
            from(e: AddrParseError) -> (e)
        }

        OrphanReplica(rid: ReplicaID, nid: NodeId) {}

        DupReplica(rid: ReplicaID) {}

        GroupOutOfOrder(a: String, b: String) {}
    }
}

impl PartialEq<ConfError> for ConfError {
    fn eq(&self, other: &ConfError) -> bool {
        match (self, other) {
            (Self::IOError(a), Self::IOError(b)) => a.kind() == b.kind(),
            (Self::BadYaml(_), Self::BadYaml(_)) => true,
            (Self::BadReplication(a), Self::BadReplication(b)) => a == b,
            (Self::OrphanReplica(a, b), Self::OrphanReplica(x, y)) => a == x && b == y,
            (Self::DupReplica(a), Self::DupReplica(b)) => a == b,
            (Self::GroupOutOfOrder(a, b), Self::GroupOutOfOrder(x, y)) => a == x && b == y,
            _ => false,
        }
    }
}
