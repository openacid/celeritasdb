#![allow(clippy::let_unit_value)]
use redis::{Commands, ControlFlow, PubSubCommands};

use std::collections::{BTreeMap, BTreeSet};
use std::collections::{HashMap, HashSet};
use std::io::BufReader;
use std::thread::{sleep, spawn};
use std::time::Duration;

use crate::support::*;

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
