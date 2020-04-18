use crate::qpaxos::Command;
use crate::qpaxos::Instance;
use crate::qpaxos::InstanceId;
use crate::replication::bcast_accept;
use crate::replication::bcast_commit;
use crate::replication::bcast_fast_accept;
use crate::replication::bcast_prepare;
use crate::testutil::TestCluster;

#[tokio::main]
async fn _bcast_fast_accept() {
    let mut tc = TestCluster::new(3);
    tc.start().await;
    let inst = foo_inst!((0, 1), "key_x", [(0, 0), (1, 0), (2, 0)]);
    let r = bcast_fast_accept(&tc.replicas[0].peers, &inst, &[true, true, true]).await;

    println!("receive fast accept replys: {:?}", r);
    // not contain self
    assert_eq!(2, r.len());
}

#[test]
fn test_bcast_fast_accept() {
    _bcast_fast_accept();
}

#[tokio::main]
async fn _bcast_accept() {
    let mut tc = TestCluster::new(3);
    tc.start().await;
    let inst = foo_inst!((0, 1), "key_x", [(0, 0), (1, 0), (2, 0)]);
    let r = bcast_accept(&tc.replicas[0].peers, &inst).await;

    println!("receive accept replys: {:?}", r);
    // not contain self
    assert_eq!(2, r.len());
}

#[test]
fn test_bcast_accept() {
    _bcast_accept();
}

#[tokio::main]
async fn _bcast_commit() {
    let mut tc = TestCluster::new(3);
    tc.start().await;
    let inst = foo_inst!((0, 1), "key_x", [(0, 0), (1, 0), (2, 0)]);
    let r = bcast_commit(&tc.replicas[0].peers, &inst).await;

    println!("receive commit replys: {:?}", r);
    // not contain self
    assert_eq!(2, r.len());
}

#[test]
fn test_bcast_commit() {
    _bcast_commit();
}

#[tokio::main]
async fn _bcast_prepare() {
    let mut tc = TestCluster::new(3);
    tc.start().await;
    let inst = foo_inst!((0, 1), "key_x", [(0, 0), (1, 0), (2, 0)]);
    let r = bcast_prepare(&tc.replicas[0].peers, &inst).await;

    println!("receive prepare replys: {:?}", r);
    // not contain self
    assert_eq!(2, r.len());
}

#[test]
fn test_bcast_prepare() {
    _bcast_prepare();
}
