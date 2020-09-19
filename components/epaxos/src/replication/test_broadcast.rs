use crate::qpaxos::Command;
use crate::qpaxos::Dep;
use crate::qpaxos::Instance;
use crate::qpaxos::InstanceId;
use crate::qpaxos::MakeRequest;
use crate::replication::bcast_msg;
use crate::testutil::TestCluster;

#[tokio::test(threaded_scheduler)]
async fn test_bcast_replicate_request() {
    let mut tc = TestCluster::new(3);
    tc.start().await;
    let inst = foo_inst!((0, 1), "key_x", [(0, 0), (1, 0), (2, 0)]);
    let req = MakeRequest::prepare(0, &inst, &[true, true, true]);

    let r = bcast_msg(&tc.replicas[0].peers, req).await;

    println!("receive prepare replys: {:?}", r);
    // not contain self
    assert_eq!(2, r.len());
}
