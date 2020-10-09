use crate::conf::GroupInfo;
use crate::qpaxos::Command;
use crate::qpaxos::MakeRequest;
use crate::replica::InstanceStatus;
use crate::replica::Replica;
use crate::replica::ReplicationStatus;
use crate::replication::bcast_msg;
use crate::replication::handle_accept_reply;
use crate::replication::handle_prepare_reply;
use crate::ReplicationError;
use crate::StorageAPI;

/// replicate runs replication algo to forward instance to other replica in leader's group.
/// An OK return value indicate the instance becomes safe, but not yet committed.
/// An Err return value means the instance could be unsafe yet.
///
/// On success it returns the status containing an instance and replication status.
pub async fn replicate(
    cmds: &[Command],
    g: &GroupInfo,
    r: &Replica,
) -> Result<ReplicationStatus, ReplicationError> {
    let grids: Vec<_> = g.replicas.keys().cloned().collect();
    println!("grids:{:?}", grids);

    let inst = r.new_instance(cmds)?;

    let n = grids.len();
    let mut st = ReplicationStatus::new(n as i32, inst);
    println!("st:{:?}", st);

    // a special path for n = 1
    let fast = st.get_fastpath_deps(&grids);
    match fast {
        Some(fdeps) => {
            st.instance.deps = Some(fdeps.into());
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

    let req = MakeRequest::prepare(0, &st.instance, &deps_committed);
    let repls = bcast_msg(&r.peers, req).await;

    println!("got {} replies", repls.len());

    for (from_rid, repl) in repls.iter() {
        println!("fast-reply from:{} {}", from_rid, repl.get_ref());
        // TODO  consume repl do not clone
        handle_prepare_reply(&mut st, *from_rid, repl.get_ref().clone())?;
        let fast = st.get_fastpath_deps(&grids);
        match fast {
            Some(fdeps) => {
                st.instance.deps = Some(fdeps.into());
                // instance is safe to commit.
                return Ok(st);
            }
            None => {
                // not enough fast replies, continue
            }
        };
    }

    let adeps = st.get_slowpath_deps(&grids);
    // println!("st.fast_deps: {:?}", st.fast_deps);
    println!("got accept deps:{:?}", adeps);

    let adeps = adeps.ok_or(ReplicationError::NotEnoughQuorum(
        InstanceStatus::Prepared,
        st.quorum,
        st.prepared[&0].replied.len() as i32,
    ))?;

    // slow path

    st.instance.deps = Some(adeps.into());
    st.start_accept();
    r.storage.set_instance(&st.instance)?;

    let req = MakeRequest::accept(0, &st.instance);
    let repls = bcast_msg(&r.peers, req).await;

    for (from_rid, repl) in repls.iter() {
        handle_accept_reply(&mut st, *from_rid, repl.get_ref().clone())?;
        if st.accepted.len() as i32 >= st.quorum {
            // instance is safe to commit.
            return Ok(st);
        }
    }

    // println!("{:?}", st.fast_deps);

    Err(ReplicationError::NotEnoughQuorum(
        InstanceStatus::Accepted,
        st.quorum,
        st.accepted.len() as i32,
    ))
}
