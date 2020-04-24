use std::i64;

use crate::conf::ClusterInfo;
use crate::qpaxos::replicate_reply;
use crate::qpaxos::replicate_request::Phase;
use crate::qpaxos::AcceptReply;
use crate::qpaxos::AcceptRequest;
use crate::qpaxos::Command;
use crate::qpaxos::CommitReply;
use crate::qpaxos::CommitRequest;
use crate::qpaxos::Conflict;
use crate::qpaxos::FastAcceptReply;
use crate::qpaxos::FastAcceptRequest;
use crate::qpaxos::Instance;
use crate::qpaxos::InstanceId;
use crate::qpaxos::InstanceIdVec;
use crate::qpaxos::PrepareReply;
use crate::qpaxos::PrepareRequest;
use crate::qpaxos::ProtocolError;
use crate::qpaxos::ReplicaId;
use crate::qpaxos::ReplicateReply;
use crate::qpaxos::ReplicateRequest;
use crate::replica::ReplicaError;
use crate::replication::RpcHandlerError;
use crate::Iter;
use crate::Storage;
use std::sync::Arc;
use storage::StorageError;
use storage::WithNs;

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
#[derive(Debug, PartialEq, Eq)]
pub struct ReplicaPeer {
    pub replica_id: ReplicaId,
    pub addr: String, // ip: port pairs of each replica
    pub alive: bool,  // if peer is alive or not
}

impl ReplicaPeer {
    pub fn new(rid: ReplicaId, addr: String, alive: bool) -> Self {
        Self {
            replica_id: rid,
            addr,
            alive,
        }
    }
}

impl From<(ReplicaId, &str, bool)> for ReplicaPeer {
    fn from(t: (ReplicaId, &str, bool)) -> ReplicaPeer {
        ReplicaPeer::new(t.0, t.1.to_string(), t.2)
    }
}

impl From<(ReplicaId, String, bool)> for ReplicaPeer {
    fn from(t: (ReplicaId, String, bool)) -> ReplicaPeer {
        ReplicaPeer::new(t.0, t.1, t.2)
    }
}

/// structure to represent a replica
pub struct Replica {
    pub replica_id: ReplicaId,
    pub group_replica_ids: Vec<ReplicaId>,
    pub peers: Vec<ReplicaPeer>,
    pub storage: Storage,
    pub committed_timeout: i32,
}

impl Replica {
    /// create a new Replica
    pub fn new(rid: ReplicaId, cinfo: &ClusterInfo, sto: Storage) -> Result<Replica, ReplicaError> {
        let group = cinfo
            .get_group(rid)
            .ok_or(ReplicaError::ReplicaNotFound(rid))?;
        let mut peers = vec![];

        for prid in group.replicas.keys() {
            let node = cinfo
                .get_replica_node(*prid)
                .ok_or(ReplicaError::ReplicaNotFound(*prid))?;

            if *prid == rid {
                continue;
            }

            // TODO check the replica is alive or not
            let addr = format!("http://{}", node.replication.to_string());
            peers.push((*prid, addr, true).into());
        }

        Ok(Replica {
            replica_id: rid,
            group_replica_ids: group.replicas.keys().cloned().collect(),
            peers,
            storage: Arc::new(WithNs::new(rid, sto)),
            // TODO get from conf
            committed_timeout: 10000,
        })
    }

    /// new_instance creates a new instance with initial_deps and deps initialized and stores it in
    /// replica storage.
    /// initial_deps and deps could contains (x, -1) if a leader has not yet propose any instance.
    pub fn new_instance(&self, cmds: &[Command]) -> Result<Instance, StorageError> {
        // TODO locking
        // TODO do not need to store max instance id, store it in replica and when starting, scan
        // backward to find the max
        // TODO test storage error

        // TODO ensure replica_ids are sorted

        let rid = self.replica_id;
        let maxs = self.get_max_instance_ids(&self.group_replica_ids);

        let this_iid = maxs.get(rid).unwrap();
        let iid = (rid, this_iid.idx + 1).into();

        let mut inst = Instance::of(cmds, (0, 0, rid).into(), &maxs);
        inst.deps = inst.initial_deps.clone();
        inst.instance_id = Some(iid);

        self.storage.set_instance(&inst)?;

        Ok(inst)
    }

    /// get_max_instance_ids returns the max instance-id for every specified replica.
    /// If there is no instance at all by a replica, a `(rid, -1)` is filled.
    pub fn get_max_instance_ids(&self, rids: &[ReplicaId]) -> InstanceIdVec {
        // TODO move the max-ids into replica. it does not need to be in storage.
        // This way, every time the server starts, a replica need to load the max ids from storage.

        let mut iids = Vec::with_capacity(rids.len());

        for rid in rids.iter() {
            let start_iid = (*rid, i64::MAX).into();
            let mut it = self.storage.get_instance_iter(start_iid, true, true);
            let inst = it.next();
            let max = match inst {
                Some(v) => v.instance_id.unwrap(),
                None => (*rid, -1).into(),
            };

            iids.push(max);
        }
        iids.into()
    }

    pub fn handle_replicate(
        &self,
        req: ReplicateRequest,
    ) -> Result<ReplicateReply, RpcHandlerError> {
        let ballot = req.ballot;
        ballot.ok_or(ProtocolError::LackOf("ballot".into()))?;

        let iid = req
            .instance_id
            .ok_or(ProtocolError::LackOf("instance_id".into()))?;

        let mut inst = self.get_instance(iid)?;
        let last_ballot = inst.ballot;

        println!("replica handle replicate for inst:{}", inst);

        let phase = req
            .phase
            .as_ref()
            .ok_or(ProtocolError::LackOf("phase".into()))?;

        match phase {
            Phase::Fast(_) | Phase::Accept(_) | Phase::Prepare(_) => {
                if req.ballot < inst.ballot {
                    return Ok(ReplicateReply {
                        err: None,
                        last_ballot,
                        instance_id: Some(iid),
                        phase: None,
                    });
                }
                inst.ballot = req.ballot;
            }
            Phase::Commit(_) => {}
        };

        let reply_phase: replicate_reply::Phase = match phase {
            Phase::Fast(r) => self.handle_fast_accept(r, &mut inst)?.into(),
            Phase::Accept(r) => self.handle_accept(r, &mut inst)?.into(),
            Phase::Commit(r) => self.handle_commit(r, &mut inst)?.into(),
            Phase::Prepare(r) => self.handle_prepare(r, &mut inst)?.into(),
        };

        self.storage.set_instance(&inst)?;

        Ok(ReplicateReply {
            err: None,
            last_ballot,
            instance_id: Some(iid),
            phase: Some(reply_phase),
        })
    }

    pub fn handle_prepare(
        &self,
        req: &PrepareRequest,
        inst: &mut Instance,
    ) -> Result<PrepareReply, RpcHandlerError> {
        let _ = req;
        let _ = inst;
        Ok(PrepareReply {
            ..Default::default()
        })
    }

    pub fn handle_fast_accept(
        &self,
        req: &FastAcceptRequest,
        inst: &mut Instance,
    ) -> Result<FastAcceptReply, RpcHandlerError> {
        let iid = ref_or_bug!(inst.instance_id);
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

        Ok(FastAcceptReply {
            deps: inst.deps.clone(),
            deps_committed: deps_committed,
        })
    }

    pub fn handle_accept(
        &self,
        req: &AcceptRequest,
        inst: &mut Instance,
    ) -> Result<AcceptReply, RpcHandlerError> {
        // TODO locking
        // TODO check instance status if committed or executed
        inst.final_deps = req.final_deps.clone();
        Ok(AcceptReply {})
    }

    pub fn handle_commit(
        &self,
        req: &CommitRequest,
        inst: &mut Instance,
    ) -> Result<CommitReply, RpcHandlerError> {
        inst.cmds = req.cmds.clone();
        inst.final_deps = req.final_deps.clone();
        inst.committed = true;

        Ok(CommitReply {})
    }

    pub fn get_instance(&self, iid: InstanceId) -> Result<Instance, ReplicaError> {
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
