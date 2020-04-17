#[cfg(test)]
use pretty_assertions::assert_eq;

use epaxos::qpaxos::*;

use std::time::Duration;

use crate::support::*;
use tokio::time::delay_for;

mod support;

macro_rules! cmds {
    [$(($op:expr, $key:expr, $val:expr)),*] => {
        vec![$(Command::from(($op, $key, $val))),*]
    }
}

macro_rules! instids {
    [$(($replica_id:expr, $idx:expr)),*] => {
        vec![$(InstanceId::from(($replica_id, $idx))),*]
    }
}

#[test]
fn test_set() {
    _test_set();
}

#[tokio::main]
async fn _test_set() {
    let ctx = InProcContext::new();
    let mut con = ctx.client.get_connection().unwrap();
    redis::cmd("SET").arg("foo").arg(42).execute(&mut con);

    let sto = ctx.storage;
    let inst = sto.get_instance((1, 0).into());
    println!("read inst: {:?}", inst);

    assert!(inst.is_ok());
    let inst = inst.unwrap();

    assert!(inst.is_some());
    let inst = inst.unwrap();

    assert_eq!(cmds![("Set", "foo", "42")], inst.cmds);
    assert_eq!(
        inst.final_deps.unwrap(),
        InstanceIdVec::from(instids![(1, -1)])
    );
    assert_eq!(inst.deps.unwrap(), InstanceIdVec::from(instids![(1, -1)]));
    assert_eq!(
        inst.initial_deps.unwrap(),
        InstanceIdVec::from(instids![(1, -1)])
    );
    assert!(inst.committed);
}

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

    let inst = Instance {
        ..Default::default()
    };

    let request = MakeRequest::accept(0, &inst);

    let response = client.accept(request).await.unwrap();

    println!("RESPONSE={:?}", response);
}
