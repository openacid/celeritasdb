use super::*;
use std::fs::File;
use std::io::Write;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;

#[test]
fn test_conf_serde_yaml() {
    // create tmp yaml file
    let content = "
nodes:
    127.0.0.1:3331:
        api_addr: 127.0.0.1:3331
        replication: 127.0.0.1:4441
    192.168.0.1:3332:
        api_addr: 192.168.0.1:3332
        api_uaddr: /var/run/usocket2
        replication: 192.168.0.1:4442
        ";

    let pb = PathBuf::from("/tmp/conf.yaml");
    let mut writer = File::create(&pb).unwrap();
    writer.write_all(content.as_bytes()).unwrap();
    writer.sync_all().unwrap();

    // test `new`
    let ci = ClusterInfo::new(&pb).unwrap();
    assert_eq!(2, ci.nodes.len());

    let n1_key = "127.0.0.1:3331";
    let n2_key = "192.168.0.1:3332";

    // test `get`
    let n1 = ci.nodes.get(n1_key).unwrap();
    assert_eq!("", n1.node_id);
    assert_eq!(SocketAddr::from_str(n1_key).unwrap(), n1.api_addr);
    assert_eq!(
        SocketAddr::from_str("127.0.0.1:4441").unwrap(),
        n1.replication
    );
    assert_eq!(true, n1.api_uaddr.is_none());

    let n2 = ci.nodes.get(n2_key).unwrap();
    assert_eq!("", n2.node_id);
    assert_eq!(SocketAddr::from_str(n2_key).unwrap(), n2.api_addr);
    assert_eq!(
        SocketAddr::from_str("192.168.0.1:4442").unwrap(),
        n2.replication
    );
    assert_eq!("/var/run/usocket2", n2.api_uaddr.as_ref().unwrap());

    fs::remove_file(pb).unwrap();
}
