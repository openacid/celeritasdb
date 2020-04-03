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
groups: []
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
groups:
-   range:
    -   a
    -   b
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
groups:
-   range:
    -   a
    -   b
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

#[test]
fn test_conf_dup_replica() {
    let cont = "
nodes:
    127.0.0.1:4441:
        api_addr: 127.0.0.1:3331
        replication: 127.0.0.1:5551
    192.168.0.1:4442:
        api_addr: 192.168.0.1:3332
        replication: 192.168.0.1:4442
groups:
-   range:
    -   a
    -   b
    replicas:
        1: 192.168.0.1:4442
        2: 192.168.0.1:4442
-   range:
    -   b
    -   c
    replicas:
        1: 127.0.0.1:4441
";

    let r = load_conf(cont);
    assert_eq!(ConfError::DupReplica(1), r.err().unwrap());
}

#[test]
fn test_conf_groups() {
    let cont = "
nodes:
    127.0.0.1:4441:
        api_addr: 127.0.0.1:3331
        replication: 127.0.0.1:5551
    192.168.0.1:4442:
        api_addr: 192.168.0.1:3332
        replication: 192.168.0.1:4442
groups:
-   range:
    -   a
    -   b
    replicas:
        1: 192.168.0.1:4442
        2: 192.168.0.1:4442
-   range:
    -   b
    -   c
    replicas:
        3: 127.0.0.1:4441
-   range:
    -   f
    -   g
    replicas:
        4: 192.168.0.1:4442
";

    let (_f, ci) = load_conf(cont).unwrap();
    assert_eq!(
        ci.get_replica(1),
        Some(&ReplicaInfo {
            group_idx: 0,
            node_id: "192.168.0.1:4442".into(),
        })
    );
    assert_eq!(
        ci.get_replica(2),
        Some(&ReplicaInfo {
            group_idx: 0,
            node_id: "192.168.0.1:4442".into(),
        })
    );
    assert_eq!(
        ci.get_replica(3),
        Some(&ReplicaInfo {
            group_idx: 1,
            node_id: "127.0.0.1:4441".into(),
        })
    );
    assert_eq!(
        ci.get_replica(4),
        Some(&ReplicaInfo {
            group_idx: 2,
            node_id: "192.168.0.1:4442".into(),
        })
    );
}

#[test]
fn test_conf_groups_out_of_order() {
    let cont = "
nodes:
    127.0.0.1:4441:
        api_addr: 127.0.0.1:3331
        replication: 127.0.0.1:5551
    192.168.0.1:4442:
        api_addr: 192.168.0.1:3332
        replication: 192.168.0.1:4442
groups:
-   range:
    -   a
    -   d
    replicas:
        1: 192.168.0.1:4442
-   range:
    -   b
    -   c
    replicas:
        2: 127.0.0.1:4441
";

    let r = load_conf(cont);
    assert_eq!(
        ConfError::GroupOutOfOrder("d".into(), "b".into()),
        r.err().unwrap()
    );
}

#[test]
fn test_conf_groups_range_start_gt_end() {
    let cont = "
nodes:
    127.0.0.1:4441:
        api_addr: 127.0.0.1:3331
        replication: 127.0.0.1:5551
groups:
-   range:
    -   b
    -   a
    replicas:
        1: 127.0.0.1:4441
";

    let r = load_conf(cont);
    assert_eq!(
        ConfError::GroupOutOfOrder("b".into(), "a".into()),
        r.err().unwrap()
    );
}

#[test]
fn test_conf_groups_get_by_key_range() {
    let cont = "
nodes:
    127.0.0.1:4441:
        api_addr: 127.0.0.1:3331
        replication: 127.0.0.1:5551
    192.168.0.1:4442:
        api_addr: 192.168.0.1:3332
        replication: 192.168.0.1:4442
groups:
-   range:
    -   a
    -   d
    replicas:
        1: 192.168.0.1:4442
-   range:
    -   g
    -   h
    replicas:
        2: 127.0.0.1:4441
";
    let (_f, ci) = load_conf(cont).unwrap();

    let g = ci.get_group_for_key("a");
    assert_eq!(&ci.groups[0], g.unwrap());

    let g = ci.get_group_for_key("d".into());
    assert!(g.is_none());

    let g = ci.get_group_for_key("g");
    assert_eq!(&ci.groups[1], g.unwrap());

    let g = ci.get_group_for_key("h");
    assert!(g.is_none());
}
