#[cfg(test)]
use pretty_assertions::assert_eq;

use epaxos::instance;
use epaxos::message;
use epaxos::qpaxos::*;
use std::time::Duration;

use crate::support::*;
use tokio::time::delay_for;

mod support;

#[test]
fn test_getset() {
    let ctx = TestContext::new();
    let mut con = ctx.connection();

    redis::cmd("SET").arg("foo").arg(42).execute(&mut con);
    assert_eq!(redis::cmd("GET").arg("foo").query(&mut con), Ok(42));

    // TODO This test not passed:
    //
    // redis::cmd("SET").arg("bar").arg("foo").execute(&mut con);
    // assert_eq!(
    //     redis::cmd("GET").arg("bar").query(&mut con),
    //     Ok(b"foo".to_vec())
    // );
}

#[test]
fn test_replication_server() {
    let ctx = TestContext::new();
    connect_repl();
    // dropping ctx cause sub process to be killed
    let _ = ctx;
}

#[tokio::main]
async fn connect_repl() {
    delay_for(Duration::from_millis(1_000)).await;
    let mut client = QPaxosClient::connect("http://127.0.0.1:6666")
        .await
        .unwrap();

    let inst = instance::Instance {
        ..Default::default()
    };

    let request = message::Request::accept(&inst);

    let response = client.accept(request).await.unwrap();

    println!("RESPONSE={:?}", response);
}
