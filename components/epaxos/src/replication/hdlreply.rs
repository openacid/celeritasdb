use crate::qpaxos::*;
use crate::replica::*;
use crate::replication::HandlerError;

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
    from_rid: ReplicaID,
    repl: &FastAcceptReply,
) -> Result<(), HandlerError> {
    // A duplicated message is received. Just ignore.
    if st.fast_replied.contains_key(&from_rid) {
        return Err(HandlerError::Dup(from_rid));
    }

    st.fast_replied.insert(from_rid, true);

    if let Some(ref e) = repl.err {
        return Err(HandlerError::RemoteError(e.clone()));
    }

    // TODO check iid matches
    let (last_ballot, _iid) = check_repl_common(&repl.cmn)?;
    let inst = &st.instance;

    let deps = repl
        .deps
        .as_ref()
        .ok_or(ProtocolError::LackOf("deps".into()))?;
    if repl.deps_committed.len() < deps.len() {
        return Err(ProtocolError::Incomplete(
            "deps_committed".into(),
            deps.len() as i32,
            repl.deps_committed.len() as i32,
        )
        .into());
    }

    if inst.ballot < Some(last_ballot) {
        return Err(HandlerError::StaleBallot(
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

    Ok(())
}

pub async fn handle_accept_reply(
    st: &mut Status,
    from_rid: ReplicaID,
    ra: &Replica,
    repl: &AcceptReply,
) -> Result<bool, HandlerError> {
    // TODO test duplicated message
    // A duplicated message is received. Just ignore.
    if st.accept_replied.contains_key(&from_rid) {
        return Err(HandlerError::Dup(from_rid));
    }
    st.accept_replied.insert(from_rid, true);

    if let Some(ref e) = repl.err {
        return Err(HandlerError::RemoteError(e.clone()));
    }

    let (last_ballot, iid) = check_repl_common(&repl.cmn)?;
    let mut inst = ra.get_instance(iid)?;

    // ignore delay reply
    let status = inst.status();
    if status != InstanceStatus::Accepted {
        return Err(HandlerError::DelayedReply(InstanceStatus::Accepted, status));
    }

    if inst.ballot < Some(last_ballot) {
        return Err(HandlerError::StaleBallot(
            inst.ballot.or(Some((0, 0, 0).into())).unwrap(),
            last_ballot,
        ));
    }

    if st.finish() {
        inst.committed = true;
        ra.storage.set_instance(&inst)?;
        bcast_commit(&ra.peers, &inst).await;
        return Ok(true);
    }

    Ok(false)
}
