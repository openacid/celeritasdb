use super::*;
use std::io::Write;
use tempfile;

fn load_conf(cont: &str) -> Result<(tempfile::NamedTempFile, ClusterInfo), ConfError> {
    let mut f = tempfile::NamedTempFile::new()?;
    f.write_all(cont.as_bytes()).unwrap();
    f.as_file().sync_all().unwrap();

    let ci = ClusterInfo::from_file(f.path())?;
    Ok((f, ci))
}

#[test]
fn test_conf_serde_yaml() {
    // TODO test ipv6

    // create tmp yaml file
    let cont = "
nodes:
    127.0.0.1:4441:
        api_addr: 127.0.0.1:3331
        replication: 127.0.0.1:5551
    192.168.0.1:4442:
        api_addr: 192.168.0.1:3332
        api_uaddr: /var/run/usocket2
        replication: 192.168.0.1:4442
replicas: {}
";

    let (_tmpf, ci) = load_conf(cont).unwrap();
    assert_eq!(2, ci.nodes.len());

    {
        // test `get`
        let nid1 = "127.0.0.1:4441";
        let nid2 = "192.168.0.1:4442";

        let n1 = ci.get(nid1).unwrap();
        assert_eq!(nid1, n1.node_id);
        assert_eq!(n1.api_addr, "127.0.0.1:3331".parse().unwrap());
        assert_eq!(n1.replication, nid1.parse().unwrap());
        assert_eq!(true, n1.api_uaddr.is_none());

        let n2 = ci.get(nid2).unwrap();
        assert_eq!(nid2, n2.node_id);
        assert_eq!(n2.api_addr, "192.168.0.1:3332".parse().unwrap());
        assert_eq!(n2.replication, nid2.parse().unwrap());
        assert_eq!("/var/run/usocket2", n2.api_uaddr.as_ref().unwrap());
    }
}

#[test]
fn test_conf_orphan_replica() {
    let cont = "
nodes:
    127.0.0.1:4441:
        api_addr: 127.0.0.1:3331
        replication: 127.0.0.1:5551
    192.168.0.1:4442:
        api_addr: 192.168.0.1:3332
        replication: 192.168.0.1:4442
replicas:
    1: 192.168.0.1:4442
    2: 192.168.0.1:4442
    3: 192.168.0.1:9999
";

    let rst = load_conf(cont);
    assert_eq!(
        rst.err().unwrap(),
        ConfError::OrphanReplica(3, "192.168.0.1:9999".into())
    );
}

#[test]
fn test_conf_replica() {
    let cont = "
nodes:
    127.0.0.1:4441:
        api_addr: 127.0.0.1:3331
        replication: 127.0.0.1:5551
    192.168.0.1:4442:
        api_addr: 192.168.0.1:3332
        replication: 192.168.0.1:4442
replicas:
    1: 192.168.0.1:4442
    2: 192.168.0.1:4442
";

    let (_f, ci) = load_conf(cont).unwrap();
    assert!(ci.get_replica_node(100).is_none());
    assert_eq!(
        ci.get_replica_node(2).unwrap(),
        &Node {
            node_id: "192.168.0.1:4442".into(),
            api_addr: "192.168.0.1:3332".parse().unwrap(),
            api_uaddr: None,
            replication: "192.168.0.1:4442".parse().unwrap(),
        }
    );
}
