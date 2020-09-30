#[macro_use]
extern crate epaxos;

#[cfg(test)]
use pretty_assertions::assert_eq;

use epaxos::qpaxos::*;
use std::time::Duration;

use crate::support::*;
use tokio::time::delay_for;

mod support;

#[tokio::test(threaded_scheduler)]
async fn test_replica_exec_thread() {
    let ctx = InProcContext::new("az_1");

    let cases = [
        inst!(
            (1, 0),
            (4, 2),
            [("Set", "x", "y")],
            [(1, 0)],
            (2, 3),
            true,
            false,
        ),
        inst!(
            (1, 1),
            (4, 2),
            [("Set", "z", "a")],
            [(1, 0)],
            (2, 3),
            true,
            false,
        ),
    ];

    // there is only replica

    for inst in cases.iter() {
        let sto = &ctx.get_replica(1).storage;
        sto.set_instance(&inst).unwrap();

        loop {
            let inst1 = sto
                .get_instance(inst.instance_id.unwrap())
                .unwrap()
                .unwrap();
            if inst1.executed {
                break;
            }

            delay_for(Duration::from_millis(10)).await;
        }

        for cmd in inst.cmds.iter() {
            let v = sto.get_kv(&cmd.key).unwrap().unwrap();
            assert_eq!(v, cmd.value);
        }
    }
}
