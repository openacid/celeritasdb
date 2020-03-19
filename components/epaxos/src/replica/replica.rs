use std::collections::HashMap;
use std::i64;
use std::net::{SocketAddr, TcpStream};
use std::time::SystemTime;

use super::super::conf::ClusterInfo;

use super::super::qpaxos::*;

use super::super::snapshot::{Error, InstanceEngine};

/// information of communication peer
pub struct ReplicaPeer {
    pub replica_id: ReplicaID,
    pub addr: SocketAddr, // ip: port pairs of each replica
    pub conn: TcpStream,  // tcp connection cache
    pub alive: bool,      // if peer is alive or not
}

/// misc configuration info
#[derive(Default)]
pub struct ReplicaConf {
    pub thrifty: bool,               // send msg only to a quorum or the full set
    pub dreply: bool, // delay replying to client after command has been executed or not
    pub beacon: bool, // periodicity detect the speed of each known replica or not
    pub inst_committed_timeout: i32, // instance committed timeout
}

/// status of a replica
pub enum ReplicaStatus {
    Joining,
    Running,
    ShuttingDown,
    Down,
}

/// structure to represent a replica
pub struct Replica {
    pub replica_id: ReplicaID,             // replica id
    pub group_replica_ids: Vec<ReplicaID>, // all replica ids in this group
    pub status: ReplicaStatus,             // status record used internally
    pub peers: Vec<ReplicaPeer>, // peers in communication, if need access from multi-thread, wrap it by Arc<>
    pub conf: ReplicaConf,       // misc conf

    pub inst_idx: InstanceIdx,
    pub latest_cp: InstanceId, // record the instance id in the lastest communication

    // storage
    pub storage: Box<dyn InstanceEngine<ColumnId = ReplicaID, ObjId = InstanceId, Obj = Instance>>,

    // to recover uncommitted instance
    pub problem_inst_ids: Vec<(InstanceId, SystemTime)>,

    pub fast_accept_ok: HashMap<InstanceId, i32>,
    pub accept_ok: HashMap<InstanceId, i32>,
}

impl Replica {
    /// create a new Replica
    /// do all the initialization and start all necessary threads here,
    /// so after this call, replica is fully functional.
    pub fn new(
        replica_id: ReplicaID,
        cluster: &ClusterInfo,
        thrifty: bool,
        exec: bool,
        beacon: bool,
    ) -> Result<Replica, String> {
        Err("not implemented".to_string())
    }

    /// handle a client request
    /// this is the main logic to implement choosing commands in epaxos protocol
    /// as described in figure 2.
    pub fn propose(&mut self, cmds: &Vec<Command>) {}

    /// do graceful shutdown of epaxos replica part
    pub fn shutdown() {}

    /// send a message
    /// if conn is None, make connection
    /// TODO(lsl): we really need something like a request context
    ///            to store conn and anything should be shared in a request or proposing an instance
    fn send_msg(&mut self, to: ReplicaID, msg: &AcceptRequest) -> Result<AcceptReply, String> {
        Err("not implemented".to_string())
    }

    /// start exec thread
    fn start_exec_thread(&mut self) {}

    // FIXME(lsl): these methods predestined to run in multi-thread and change self,
    //             so is it good to implement as a method or a function to take Replica as arg?
    //             but for now, it doesn't matter since we just want the interface.

    // ASK(lsl->xp): is it ok to return a result and ask http-framework to judge type and send
    // reply to requester?
    // I think that would be a great place to do statistics and measurement.
    // and by making Replica independent,it's easier to do test.
    fn handle_prepare(&mut self, req: &PrepareRequest) -> Result<PrepareReply, String> {
        Err("not implemented".to_string())
    }

    pub fn handle_fast_accept(&mut self, req: &FastAcceptRequest) -> FastAcceptReply {
        match self._fast_accept(req) {
            Ok((inst, deps_committed)) => MakeReply::fast_accept(&inst, &deps_committed),
            Err(e) => FastAcceptReply {
                err: Some(e.to_qerr()),
                ..Default::default()
            },
        }
    }

    fn _fast_accept(&mut self, req: &FastAcceptRequest) -> Result<(Instance, Vec<bool>), Error> {
        let (ballot, iid) = self._check_req_common(&req.cmn)?;

        let mut inst = match self.storage.get_instance(iid)? {
            Some(v) => {
                if v.ballot.is_none() || v.ballot.unwrap().num != 0 {
                    return Err(Error::Existed {});
                }
                v
            }
            None => self._empty_instance(Some(iid)),
        };

        inst.ballot = Some(ballot);
        inst.last_ballot = inst.ballot;

        inst.cmds = req.cmds.clone();
        inst.initial_deps = req.initial_deps.clone();
        inst.deps = req.initial_deps.clone();

        // TODO update local commited status by deps_committed[i] is true
        let mut deps_committed = req.deps_committed.clone();

        for rid in self.group_replica_ids.iter() {
            let start_iid = (*rid, i64::MAX).into();

            for local_inst in self.storage.get_instance_iter(start_iid, true, true) {
                if local_inst.deps == None {
                    continue;
                };

                if let Some(y) = local_inst
                    .deps
                    .as_ref()
                    .unwrap()
                    .iter()
                    .find(|y| y.replica_id == iid.replica_id)
                {
                    if y.idx >= iid.idx {
                        continue;
                    }
                } else {
                    continue;
                }
                // TODO try this snippet. not passed. do it later. -- xp
                // let q = match local_inst.deps {
                //     Some(v) => &v,
                //     None => continue
                // };

                // q.get(iid.replica_id);

                // let y = local_inst.deps.unwrap().get(iid.replica_id);
                // let y = match y {
                //     Some(v) => v,
                //     None => continue
                // };
                // // local_inst already depends on req.instance, thus req.instance would never depends on local_inst
                // if y.idx >= iid.idx {
                //     continue;
                // }

                if inst.conflict(&local_inst) == false && local_inst.committed == false {
                    continue;
                }

                let x_iid = local_inst.instance_id.unwrap();

                if let Some(lx_idx) = inst
                    .deps
                    .as_ref()
                    // TODO unwrap
                    .unwrap()
                    .iter()
                    .position(|y| y.replica_id == x_iid.replica_id)
                {
                    if x_iid > inst.deps.as_ref().unwrap().ids[lx_idx] {
                        inst.deps.as_mut().unwrap().ids[lx_idx] = x_iid;
                        deps_committed[lx_idx] = local_inst.committed;
                    } else if x_iid == inst.deps.as_ref().unwrap().ids[lx_idx] {
                        deps_committed[lx_idx] = deps_committed[lx_idx] || local_inst.committed;
                    }
                }
            }
        }

        self.storage.set_instance(&inst)?;

        Ok((inst, deps_committed))
    }

    pub fn handle_accept(&mut self, req: &AcceptRequest) -> AcceptReply {
        let inst = self._accept(req);
        match inst {
            Ok(inst) => MakeReply::accept(&inst),
            Err(e) => AcceptReply {
                err: Some(e.to_qerr()),
                ..Default::default()
            },
        }
    }

    fn _accept(&mut self, req: &AcceptRequest) -> Result<Instance, Error> {
        // TODO locking
        let (ballot, iid) = self._check_req_common(&req.cmn)?;

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

    pub fn handle_accept_reply(&mut self, repl: &AcceptReply) {
        match self._accept_reply(repl) {
            Ok(_) => {}
            Err(e) => {
                println!("handle accept reply err: {:?}", e);
            }
        }
    }

    fn _accept_reply(&mut self, repl: &AcceptReply) -> Result<(), Error> {
        if let Some(_) = repl.err {
            return Ok(());
        }

        let (last_ballot, iid) = self._check_repl_common(&repl.cmn)?;
        let mut inst = self._get_instance(iid)?;

        // ignore delay reply
        if inst.status() != InstanceStatus::Accepted {
            return Ok(());
        }

        if inst.ballot.unwrap() < last_ballot {
            return Ok(());
        }

        let cnt = self.accept_ok.entry(iid).or_insert(0);
        *cnt += 1;

        if *cnt + 1 >= self.quorum() {
            self.accept_ok.remove(&iid);
            inst.committed = true;
            self.storage.set_instance(&inst)?;
            // TODO reply to client
            if !self.conf.dreply {}
            self._bcast_commit(&inst);
        }

        Ok(())
    }

    pub fn handle_commit(&mut self, req: &CommitRequest) -> CommitReply {
        // TODO protocol wrapping may be better to be in server impl instead of being here

        match self._commit(req) {
            Ok(inst) => MakeReply::commit(&inst),
            Err(e) => CommitReply {
                err: Some(e.to_qerr()),
                ..Default::default()
            },
        }
    }

    fn _commit(&mut self, req: &CommitRequest) -> Result<Instance, Error> {
        let (ballot, iid) = self._check_req_common(&req.cmn)?;

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

    fn _check_req_common(
        &mut self,
        cm: &Option<RequestCommon>,
    ) -> Result<(BallotNum, InstanceId), Error> {
        let cm = cm.as_ref().ok_or(Error::LackOf("cmn".into()))?;

        let replica_id = cm.to_replica_id;
        if replica_id != self.replica_id {
            return Err(Error::NoSuchReplica {
                replica_id,
                my_replica_id: self.replica_id,
            });
        }

        let ballot = cm.ballot.ok_or(Error::LackOf("cmn.ballot".into()))?;

        let iid = cm
            .instance_id
            .ok_or(Error::LackOf("cmn.instance_id".into()))?;

        Ok((ballot, iid))
    }

    fn _check_repl_common(
        &mut self,
        cm: &Option<ReplyCommon>,
    ) -> Result<(BallotNum, InstanceId), Error> {
        let cm = cm.as_ref().ok_or(Error::LackOf("cmn".into()))?;
        let ballot = cm
            .last_ballot
            .ok_or(Error::LackOf("cmn.last_ballot".into()))?;
        let iid = cm
            .instance_id
            .ok_or(Error::LackOf("cmn.instance_id".into()))?;

        Ok((ballot, iid))
    }

    fn _get_instance(&mut self, iid: InstanceId) -> Result<Instance, Error> {
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

    fn _bcast_fast_accept(&mut self, req: &FastAcceptRequest) {
        let iid = req.cmn.as_ref().unwrap().instance_id.unwrap();
        self.fast_accept_ok.remove(&iid);
    }

    fn _bcast_accept(&mut self, req: &AcceptRequest) {
        let iid = req.cmn.as_ref().unwrap().instance_id.unwrap();
        self.accept_ok.remove(&iid);
    }

    fn _bcast_commit(&mut self, inst: &Instance) {}

    fn quorum(&self) -> i32 {
        let f = self.group_replica_ids.len() as i32 / 2;
        f + 1
    }

    fn fast_quorum(&self) -> i32 {
        let f = self.group_replica_ids.len() as i32 / 2;
        f + (f + 1) / 2
    }
}
