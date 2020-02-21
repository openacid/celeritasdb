use super::*;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[test]
fn test_conf() {
    // create tmp yaml file
    let content = "
nodes:
    - {req: [\"127.0.0.1\", 3331], dup: [\"127.0.0.1\", 4441]}
    - {req: [\"192.168.0.1\", 3332], dup: [\"192.168.0.1\", 4442], req_unix: /var/run/usocket2}
";

    let pb = PathBuf::from("/tmp/conf.yaml");
    let mut file = File::create(&pb).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file.sync_all().unwrap();

    // test `new`
    let ci = ClusterInfo::new(&pb).unwrap();
    assert_eq!(2, ci.nodes.len());

    // test `get`
    let node = ci.get_node("127.0.0.1", 3331).unwrap();
    assert_eq!("", node.node_id);
    assert_eq!(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), node.req_addr.ip());
    assert_eq!(3331, node.req_addr.port());
    assert_eq!(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), node.dup_addr.ip());
    assert_eq!(4441, node.dup_addr.port());
    assert_eq!(true, node.req_uaddr.is_none());

    let node = ci.get_node("192.168.0.1", 3332).unwrap();
    assert_eq!("", node.node_id);
    assert_eq!(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)), node.req_addr.ip());
    assert_eq!(3332, node.req_addr.port());
    assert_eq!(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)), node.dup_addr.ip());
    assert_eq!(4442, node.dup_addr.port());
    assert_eq!("/var/run/usocket2", node.req_uaddr.as_ref().unwrap());

    let node = ci.get_node("192.168.0.1", 4442);
    assert!(node.is_none());
    let node = ci.get_node("127.0.0.1", 4441);
    assert!(node.is_none());

    fs::remove_file(pb).unwrap();
}
