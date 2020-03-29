use std::i64;
use tonic::Response;

use crate::conf::ClusterInfo;
use crate::qpaxos::*;
use crate::replica::AcceptStatus;
use crate::replica::Error as ReplicaError;
use crate::replica::InstanceStatus;
use crate::snapshot::{Error as SnapError, Storage};

/// ref_or_bug extracts a immutable ref from an Option.
/// If the Option is None a bug handler is triggered.
/// Without specifying bug handler, it just calls a panic!()
macro_rules! ref_or_bug {
    ($fld:expr, $bug_handler:expr) => {
        if $fld == None {
            $bug_handler;
        } else {
            $fld.as_ref().unwrap()
        }
    };

    ($fld:expr) => {
        ref_or_bug!(
            $fld,
            panic!("{} is unexpected to be None", stringify!($fld))
        )
    };
}

/// information of communication peer
pub struct ReplicaPeer {
    pub replica_id: ReplicaID,
    pub addr: String, // ip: port pairs of each replica
    pub alive: bool,  // if peer is alive or not
}

impl ReplicaPeer {
    pub fn new(rid: ReplicaID, addr: String, alive: bool) -> Self {
        Self {
            replica_id: rid,
            addr,
            alive,
        }
    }
}

impl From<(ReplicaID, &str, bool)> for ReplicaPeer {
    fn from(t: (ReplicaID, &str, bool)) -> ReplicaPeer {
        ReplicaPeer::new(t.0, t.1.to_string(), t.2)
    }
}

impl From<(ReplicaID, String, bool)> for ReplicaPeer {
    fn from(t: (ReplicaID, String, bool)) -> ReplicaPeer {
        ReplicaPeer::new(t.0, t.1, t.2)
    }
}

/// misc configuration info
#[derive(Default)]
pub struct ReplicaConf {
    pub dreply: bool, // delay replying to client after command has been executed or not
    pub inst_committed_timeout: i32, // instance committed timeout
}

/// structure to represent a replica
pub struct Replica {
    pub replica_id: ReplicaID,
    pub group_replica_ids: Vec<ReplicaID>,
    pub peers: Vec<ReplicaPeer>,
    pub conf: ReplicaConf,
    pub storage: Storage,
}

impl Replica {
    /// create a new Replica
    /// do all the initialization and start all necessary threads here,
    /// so after this call, replica is fully functional.
    pub fn new(
        _replica_id: ReplicaID,
        _cluster: &ClusterInfo,
        _thrifty: bool,
        _exec: bool,
    ) -> Result<Replica, String> {
        Err("not implemented".to_string())
    }

    /// start exec thread
    fn _start_exec_thread(&self) {}

    pub fn new_instance(&self, cmds: Vec<Command>) -> Result<Instance, ReplicaError> {
        // TODO locking
        // TODO do not need to store max instance id, store it in replica and when starting, scan
        // backward to find the max
        // TODO test storage error

        let n = self.group_replica_ids.len();
        let mut deps = Vec::with_capacity(n);
        // TODO ensure replica_ids are sorted
        for rid in self.group_replica_ids.iter() {
            let max = self.storage.get_ref("max", *rid);
            match max {
                Ok(v) => deps.push(v),
                Err(e) => match e {
                    SnapError::NotFound => {}
                    _ => {
                        return Err(e.into());
                    }
                },
            }
        }

        let iid = self.storage.next_instance_id(self.replica_id)?;
        let inst = Instance {
            last_ballot: None,
            // TODO need to use time stamp as epoch?
            ballot: Some((0, 0, self.replica_id).into()),
            instance_id: Some(iid),
            cmds,
            initial_deps: Some(deps.clone().into()),
            deps: Some(deps.into()),
            final_deps: None,
            committed: false,
            executed: false,
        };

        Ok(inst)
    }

    fn _handle_prepare(&self, _req: &PrepareRequest) -> Result<PrepareReply, String> {
        Err("not implemented".to_string())
    }

    pub fn handle_fast_accept(&self, req: &FastAcceptRequest) -> FastAcceptReply {
        match self._fast_accept(req) {
            Ok((inst, deps_committed)) => MakeReply::fast_accept(&inst, &deps_committed),
            Err(e) => FastAcceptReply {
                err: Some(e.into()),
                ..Default::default()
            },
        }
    }

    fn _fast_accept(&self, req: &FastAcceptRequest) -> Result<(Instance, Vec<bool>), ReplicaError> {
        let (ballot, iid) = check_req_common(self.replica_id, &req.cmn)?;

        let mut inst = match self.storage.get_instance(iid)? {
            Some(v) => {
                if v.ballot.is_none() || v.ballot.unwrap().num != 0 {
                    return Err(ReplicaError::Existed {});
                }
                v
            }
            None => self._empty_instance(Some(iid)),
        };

        inst.ballot = Some(ballot);
        inst.last_ballot = inst.ballot;

        let req_deps = ref_or_bug!(req.initial_deps);

        inst.cmds = req.cmds.clone();
        inst.initial_deps = req.initial_deps.clone();
        inst.deps = req.initial_deps.clone();

        // TODO update local commited status by deps_committed[i] is true
        let mut deps_committed = req.deps_committed.clone();

        for rid in self.group_replica_ids.iter() {
            let start_iid = (*rid, i64::MAX).into();

            for local_inst in self.storage.get_instance_iter(start_iid, true, true) {
                let local_deps = ref_or_bug!(local_inst.deps);
                let local_iid = ref_or_bug!(local_inst.instance_id);

                if local_deps >= &iid {
                    continue;
                }

                // only update deps if seen a conflict instance, or it is a committed instance.
                if inst.conflict(&local_inst) || local_inst.committed {
                    // go on to update deps
                } else {
                    continue;
                }

                // TODO: test: fast-accept adding a new dep

                if req_deps > &local_iid {
                    // the incoming instance already depends on this local instance, which implies
                    // it depends on any lower instances.
                    break;
                }

                let req_deps = inst.deps.as_mut().unwrap();

                let (ith, _) = req_deps.set(*local_iid);
                if ith == deps_committed.len() {
                    deps_committed.push(local_inst.committed);
                } else {
                    deps_committed[ith] = deps_committed[ith] || local_inst.committed;
                }

                // Here it stops iteration as the highest interfering inst has been seen.
                break;
            }
        }

        self.storage.set_instance(&inst)?;

        Ok((inst, deps_committed))
    }

    pub fn handle_accept(&self, req: &AcceptRequest) -> AcceptReply {
        let inst = self._accept(req);
        match inst {
            Ok(inst) => MakeReply::accept(&inst),
            Err(e) => AcceptReply {
                err: Some(e.into()),
                ..Default::default()
            },
        }
    }

    fn _accept(&self, req: &AcceptRequest) -> Result<Instance, ReplicaError> {
        // TODO locking
        let (ballot, iid) = check_req_common(self.replica_id, &req.cmn)?;

        let mut inst = self._get_instance(iid)?;
        // TODO check instance status if committed or executed

        inst.last_ballot = inst.ballot;

        // allow inst.ballot to be None
        if Some(ballot) >= inst.ballot {
            inst.ballot = Some(ballot);

            inst.final_deps = req.final_deps.clone();
            self.storage.set_instance(&inst)?;
        }

        Ok(inst)
    }

    pub fn handle_commit(&self, req: &CommitRequest) -> CommitReply {
        // TODO protocol wrapping may be better to be in server impl instead of being here

        match self._commit(req) {
            Ok(inst) => MakeReply::commit(&inst),
            Err(e) => CommitReply {
                err: Some(e.into()),
                ..Default::default()
            },
        }
    }

    fn _commit(&self, req: &CommitRequest) -> Result<Instance, ReplicaError> {
        let (ballot, iid) = check_req_common(self.replica_id, &req.cmn)?;

        // TODO locking
        let mut inst = self._get_instance(iid)?;

        // TODO issue: after commit, inst.last_ballot might be >= inst.ballot, which might confuse
        // other procedure.
        inst.last_ballot = inst.ballot;
        inst.ballot = Some(ballot);

        inst.cmds = req.cmds.clone();
        inst.final_deps = req.final_deps.clone();
        inst.committed = true;

        self.storage.set_instance(&inst)?;

        Ok(inst)
    }

    fn _get_instance(&self, iid: InstanceId) -> Result<Instance, ReplicaError> {
        let inst = self.storage.get_instance(iid)?;

        let inst = match inst {
            Some(inst) => inst,
            // not found
            None => self._empty_instance(Some(iid)),
        };

        Ok(inst)
    }

    fn _empty_instance(&self, iid: Option<InstanceId>) -> Instance {
        Instance {
            instance_id: iid,
            ..Default::default()
        }
    }
}

fn check_req_common(
    myrid: ReplicaID,
    cm: &Option<RequestCommon>,
) -> Result<(BallotNum, InstanceId), ProtocolError> {
    let cm = cm.as_ref().ok_or(ProtocolError::LackOf("cmn".into()))?;

    let replica_id = cm.to_replica_id;
    if replica_id != myrid {
        return Err((replica_id, myrid).into());
    }

    let ballot = cm
        .ballot
        .ok_or(ProtocolError::LackOf("cmn.ballot".into()))?;

    let iid = cm
        .instance_id
        .ok_or(ProtocolError::LackOf("cmn.instance_id".into()))?;

    Ok((ballot, iid))
}

fn check_repl_common(cm: &Option<ReplyCommon>) -> Result<(BallotNum, InstanceId), ProtocolError> {
    let cm = cm.as_ref().ok_or(ProtocolError::LackOf("cmn".into()))?;
    let ballot = cm
        .last_ballot
        .ok_or(ProtocolError::LackOf("cmn.last_ballot".into()))?;
    let iid = cm
        .instance_id
        .ok_or(ProtocolError::LackOf("cmn.instance_id".into()))?;

    Ok((ballot, iid))
}

pub async fn handle_accept_reply(
    ra: &Replica,
    repl: &AcceptReply,
    st: &mut AcceptStatus,
) -> Result<(), ReplicaError> {
    if let Some(_) = repl.err {
        return Ok(());
    }

    let (last_ballot, iid) = check_repl_common(&repl.cmn)?;
    let mut inst = ra._get_instance(iid)?;

    // ignore delay reply
    if inst.status() != InstanceStatus::Accepted {
        return Ok(());
    }

    if inst.ballot.unwrap() < last_ballot {
        return Ok(());
    }

    if st.finish() {
        inst.committed = true;
        ra.storage.set_instance(&inst)?;
        bcast_commit(&ra.peers, &inst).await;
    }

    Ok(())
}

macro_rules! bcast_msg {
    ($peers:expr, $make_req:expr, $func:ident) => {{
        let mut rst = Vec::with_capacity($peers.len());
        for p in $peers.iter() {
            let mut client = match QPaxosClient::connect(p.addr.clone()).await {
                Ok(c) => c,
                // TODO just ignore the err
                Err(e) => {
                    println!("{:?} while connect to {:?}", e, &p.addr);
                    continue;
                }
            };

            let req = $make_req(p.replica_id);
            let repl = match client.$func(req).await {
                Ok(r) => r,
                // TODO just ignore the err
                Err(e) => {
                    println!("{:?} while request to {:?}", e, &p.addr);
                    continue;
                }
            };

            rst.push(repl);
        }

        if rst.len() > 0 {
            return Some(rst);
        }

        return None;
    }};
}

pub async fn bcast_fast_accept(
    peers: &Vec<ReplicaPeer>,
    inst: &Instance,
    deps_committed: &[bool],
) -> Option<Vec<Response<FastAcceptReply>>> {
    bcast_msg!(
        peers,
        |rid| MakeRequest::fast_accept(rid, inst, deps_committed),
        fast_accept
    );
}

pub async fn bcast_accept(
    peers: &Vec<ReplicaPeer>,
    inst: &Instance,
) -> Option<Vec<Response<AcceptReply>>> {
    bcast_msg!(peers, |rid| MakeRequest::accept(rid, inst), accept);
}

pub async fn bcast_commit(
    peers: &Vec<ReplicaPeer>,
    inst: &Instance,
) -> Option<Vec<Response<CommitReply>>> {
    bcast_msg!(peers, |rid| MakeRequest::commit(rid, inst), commit);
}

pub async fn bcast_prepare(
    peers: &Vec<ReplicaPeer>,
    inst: &Instance,
) -> Option<Vec<Response<PrepareReply>>> {
    bcast_msg!(peers, |rid| MakeRequest::prepare(rid, inst), prepare);
}
