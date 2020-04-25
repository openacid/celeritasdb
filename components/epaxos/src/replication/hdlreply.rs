use crate::qpaxos::replicate_reply::Phase;
use crate::qpaxos::Direction;
use crate::qpaxos::ReplicateReply;
use crate::qpaxos::*;
use crate::replica::*;
use crate::replication::RpcHandlerError;

pub fn check_repl_common(inst: &Instance, reply: ReplicateReply) -> Result<Phase, RpcHandlerError> {
    let iid = reply
        .instance_id
        .ok_or(ProtocolError::LackOf("instance_id".into()))?;

    if iid != inst.instance_id.unwrap() {
        let err = ProtocolError::NotMatch(
            "instance_id".into(),
            format!("{}", inst.instance_id.unwrap()),
            format!("{}", iid),
        );
        return Err(err.into());
    }

    let phase = reply.phase.ok_or(ProtocolError::LackOf("phase".into()))?;

    let last_ballot = reply.last_ballot;
    if inst.ballot < last_ballot {
        let zero = Some(BallotNum::default());
        let err = RpcHandlerError::StaleBallot(
            inst.ballot.or(zero).unwrap(),
            last_ballot.or(zero).unwrap(),
        );
        return Err(err);
    }

    Ok(phase)
}

pub fn handle_fast_accept_reply(
    st: &mut Status,
    from_rid: ReplicaId,
    repl: ReplicateReply,
) -> Result<(), RpcHandlerError> {
    // A duplicated message is received. Just ignore.
    if st.fast_replied.contains_key(&from_rid) {
        return Err(RpcHandlerError::DupRpc(
            InstanceStatus::FastAccepted,
            Direction::Reply,
            from_rid,
            st.instance.instance_id.unwrap(),
        ));
    }

    st.fast_replied.insert(from_rid, true);

    if let Some(ref e) = repl.err {
        return Err(RpcHandlerError::RemoteError(e.clone()));
    }

    let phase = check_repl_common(&st.instance, repl)?;

    let frepl: FastAcceptReply = phase
        .try_into()
        .or(Err(ProtocolError::LackOf("phase::Fast".into())))?;

    let deps = frepl
        .deps
        .as_ref()
        .ok_or(ProtocolError::LackOf("phase::Fast.deps".into()))?;

    // TODO choose the appropriate data structure to reduce needless error checking
    if frepl.deps_committed.len() < deps.len() {
        return Err(ProtocolError::Incomplete(
            "phase::Fast.deps_committed".into(),
            deps.len() as i32,
            frepl.deps_committed.len() as i32,
        )
        .into());
    }

    for (i, d) in deps.iter().enumerate() {
        let rid = d.replica_id;
        if !st.fast_deps.contains_key(&rid) {
            st.fast_deps.insert(rid, Vec::new());
        }

        st.fast_deps.get_mut(&rid).unwrap().push(*d);

        if frepl.deps_committed[i] {
            st.fast_committed.insert(*d, true);
        }
    }

    st.fast_oks.insert(from_rid, true);

    Ok(())
}

pub fn handle_accept_reply(
    st: &mut Status,
    from_rid: ReplicaId,
    repl: ReplicateReply,
) -> Result<(), RpcHandlerError> {
    // TODO test duplicated message
    // A duplicated message is received. Just ignore.
    if st.accept_replied.contains_key(&from_rid) {
        return Err(RpcHandlerError::DupRpc(
            InstanceStatus::Accepted,
            Direction::Reply,
            from_rid,
            st.instance.instance_id.unwrap(),
        ));
    }
    st.accept_replied.insert(from_rid, true);

    if let Some(ref e) = repl.err {
        return Err(RpcHandlerError::RemoteError(e.clone()));
    }

    check_repl_common(&st.instance, repl)?;
    let inst = &st.instance;

    // TODO is it necessary to check status?
    // ignore delay reply
    let status = inst.status();
    if status != InstanceStatus::Accepted {
        return Err(RpcHandlerError::DelayedReply(
            InstanceStatus::Accepted,
            status,
        ));
    }

    st.accept_oks.insert(from_rid, true);

    Ok(())
}
