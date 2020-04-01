use super::*;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[test]
fn test_conf_serde_yaml() {
    // TODO test ipv6

    // create tmp yaml file
    let content = "
nodes:
    127.0.0.1:4441:
        api_addr: 127.0.0.1:3331
        replication: 127.0.0.1:5551
    192.168.0.1:4442:
        api_addr: 192.168.0.1:3332
        api_uaddr: /var/run/usocket2
        replication: 192.168.0.1:4442
        ";

    let pb = PathBuf::from("/tmp/conf.yaml");
    let mut writer = File::create(&pb).unwrap();
    writer.write_all(content.as_bytes()).unwrap();
    writer.sync_all().unwrap();

    let ci = ClusterInfo::from_file(&pb).unwrap();
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

    fs::remove_file(pb).unwrap();
}
