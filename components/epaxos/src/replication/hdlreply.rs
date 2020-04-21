use crate::qpaxos::Direction;
use crate::qpaxos::ReplicateReply;
use crate::qpaxos::*;
use crate::replica::*;
use crate::replication::RpcHandlerError;

pub fn check_repl_common(cm: &ReplicateReply) -> Result<(BallotNum, InstanceId), ProtocolError> {
    let ballot = cm
        .last_ballot
        .ok_or(ProtocolError::LackOf("last_ballot".into()))?;
    let iid = cm
        .instance_id
        .ok_or(ProtocolError::LackOf("instance_id".into()))?;

    Ok((ballot, iid))
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

    // TODO check iid matches
    let (last_ballot, _iid) = check_repl_common(&repl)?;
    let inst = &st.instance;

    let phase = repl.phase.ok_or(ProtocolError::LackOf("phase".into()))?;
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

    if inst.ballot < Some(last_ballot) {
        return Err(RpcHandlerError::StaleBallot(
            inst.ballot.or(Some((0, 0, 0).into())).unwrap(),
            last_ballot,
        ));
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
    repl: &ReplicateReply,
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

    let (last_ballot, _iid) = check_repl_common(&repl)?;
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

    if inst.ballot < Some(last_ballot) {
        return Err(RpcHandlerError::StaleBallot(
            inst.ballot.or(Some((0, 0, 0).into())).unwrap(),
            last_ballot,
        ));
    }

    st.accept_oks.insert(from_rid, true);

    Ok(())
}
