use crate::RangeLookupError;
use crate::ServerData;
use epaxos::conf::ClusterInfo;
use epaxos::snapshot::MemEngine;
use std::sync::Arc;

#[test]
fn test_serverdata() {
    let yaml = "
nodes:
    127.0.0.1:4441:
        api_addr: 127.0.0.1:3331
        replication: 127.0.0.1:5551
    192.168.0.1:4442:
        api_addr: 192.168.0.1:3332
        api_uaddr: /var/run/usocket2
        replication: 192.168.0.1:4442
groups:
-   range:
    -   a
    -   p
    replicas:
        1: 192.168.0.1:4442
";
    let ci = ClusterInfo::from_str(yaml).unwrap();
    let sto = MemEngine::new().unwrap();
    let sto = Arc::new(sto);
    {
        // test lookup group
        let node_id = "192.168.0.1:4442";
        let sd = ServerData::new(sto.clone(), ci.clone(), node_id.into());

        let (g, r) = sd.get_local_replica_for_key("b".as_bytes()).unwrap();
        assert_eq!(g, &ci.groups[0]);
        assert_eq!(r.replica_id, sd.local_replicas.get(&1).unwrap().replica_id);
        let rst = sd.get_local_replica_for_key("z".as_bytes());
        assert_eq!(
            RangeLookupError::NoGroupForKey("z".into()),
            rst.err().unwrap()
        );
    }
    {
        // test no replica locally
        let node_id = "127.0.0.1:4441";
        let sd = ServerData::new(sto.clone(), ci.clone(), node_id.into());

        assert_eq!(
            RangeLookupError::NoLocalReplicaForKey("b".into()),
            sd.get_local_replica_for_key("b".as_bytes()).err().unwrap()
        );
    }
}
