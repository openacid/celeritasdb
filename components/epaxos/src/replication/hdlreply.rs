use crate::qpaxos::Direction;
use crate::qpaxos::*;
use crate::replica::*;
use crate::replication::RpcHandlerError;

pub fn check_repl_common(
    cm: &Option<ReplyCommon>,
) -> Result<(BallotNum, InstanceId), ProtocolError> {
    let cm = cm.as_ref().ok_or(ProtocolError::LackOf("cmn".into()))?;
    let ballot = cm
        .last_ballot
        .ok_or(ProtocolError::LackOf("cmn.last_ballot".into()))?;
    let iid = cm
        .instance_id
        .ok_or(ProtocolError::LackOf("cmn.instance_id".into()))?;

    Ok((ballot, iid))
}

pub fn handle_fast_accept_reply(
    st: &mut Status,
    from_rid: ReplicaId,
    repl: &FastAcceptReply,
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
    let (last_ballot, _iid) = check_repl_common(&repl.cmn)?;
    let inst = &st.instance;

    let deps = repl
        .deps
        .as_ref()
        .ok_or(ProtocolError::LackOf("deps".into()))?;

    // TODO choose the appropriate data structure to reduce needless error checking
    if repl.deps_committed.len() < deps.len() {
        return Err(ProtocolError::Incomplete(
            "deps_committed".into(),
            deps.len() as i32,
            repl.deps_committed.len() as i32,
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

        if repl.deps_committed[i] {
            st.fast_committed.insert(*d, true);
        }
    }

    st.fast_oks.insert(from_rid, true);

    Ok(())
}

pub fn handle_accept_reply(
    st: &mut Status,
    from_rid: ReplicaId,
    repl: &AcceptReply,
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

    let (last_ballot, _iid) = check_repl_common(&repl.cmn)?;
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
