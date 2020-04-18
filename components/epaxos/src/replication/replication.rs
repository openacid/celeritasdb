use crate::conf::GroupInfo;
use crate::qpaxos::Command;
use crate::replica::InstanceStatus;
use crate::replica::Replica;
use crate::replica::Status;
use crate::replication::bcast_accept;
use crate::replication::bcast_fast_accept;
use crate::replication::handle_accept_reply;
use crate::replication::handle_fast_accept_reply;
use crate::ReplicationError;

/// replicate runs replication algo to forward instance to other replica in leader's group.
/// An OK return value indicate the instance becomes safe, but not yet committed.
/// An Err return value means the instance could be unsafe yet.
///
/// On success it returns the status containing an instance and replication status.
pub async fn replicate(
    cmds: &[Command],
    g: &GroupInfo,
    r: &Replica,
) -> Result<Status, ReplicationError> {
    let grids: Vec<_> = g.replicas.keys().cloned().collect();
    println!("grids:{:?}", grids);

    let inst = r.new_instance(cmds)?;

    let n = grids.len();
    let mut st = Status::new(n as i32, inst);
    println!("st:{:?}", st);

    // a special path for n = 1
    let fast = st.get_fast_commit_deps(&grids);
    match fast {
        Some(fdeps) => {
            st.instance.final_deps = Some(fdeps.into());
            // instance is safe to commit.
            return Ok(st);
        }
        None => {
            // not enough fast replies, continue
        }
    };

    // TODO not impl yet.
    let mut deps_committed = vec![];
    for _ in 0..g.replicas.len() {
        deps_committed.push(false);
    }

    let repls = bcast_fast_accept(&r.peers, &st.instance, &deps_committed).await;

    println!("fast-replies:{:?}", repls);

    for (from_rid, repl) in repls.iter() {
        handle_fast_accept_reply(&mut st, *from_rid, repl.get_ref())?;
        let fast = st.get_fast_commit_deps(&grids);
        match fast {
            Some(fdeps) => {
                st.instance.final_deps = Some(fdeps.into());
                // instance is safe to commit.
                return Ok(st);
            }
            None => {
                // not enough fast replies, continue
            }
        };
    }

    let adeps = st.get_accept_deps(&grids);
    println!("st.fast_deps: {:?}", st.fast_deps);
    println!("got accept deps:{:?}", adeps);

    let adeps = adeps.ok_or(ReplicationError::NotEnoughQuorum(
        InstanceStatus::FastAccepted,
        st.quorum,
        st.fast_oks.len() as i32,
    ))?;

    // slow path

    st.instance.final_deps = Some(adeps.into());
    st.start_accept();
    r.storage.set_instance(&st.instance)?;

    let repls = bcast_accept(&r.peers, &st.instance).await;

    for (from_rid, repl) in repls.iter() {
        handle_accept_reply(&mut st, *from_rid, &r, repl.get_ref())?;
        if st.accept_oks.len() as i32 >= st.quorum {
            // instance is safe to commit.
            return Ok(st);
        }
    }

    println!("{:?}", st.fast_deps);

    Err(ReplicationError::NotEnoughQuorum(
        InstanceStatus::Accepted,
        st.quorum,
        st.accept_oks.len() as i32,
    ))
}
