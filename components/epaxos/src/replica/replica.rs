use std::net::{SocketAddr, TcpStream};
use std::time::SystemTime;

use super::super::conf::ClusterInfo;

use super::super::qpaxos::*;

use super::super::snapshot::{Error, InstanceEngine, TxEngine};

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
    pub exec: bool,                  // exec comamnd or not
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
    pub latest_cp: InstanceID, // record the instance id in the lastest communication

    // storage
    pub storage: Box<dyn InstanceEngine<ColumnId = ReplicaID, ObjId = InstanceID, Obj = Instance>>,

    // to recover uncommitted instance
    pub problem_inst_ids: Vec<(InstanceID, SystemTime)>,
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

    /// this is the main logic to implement recovery procedure in epaxos protocol
    /// as described in figure 3.
    fn explicit_prepare(&mut self, instance: &Instance) {}

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

    fn handle_pre_accept(&mut self, req: &FastAcceptRequest) -> Result<FastAcceptReply, String> {
        Err("not implemented".to_string())
    }

    fn handle_accept(&mut self, req: &AcceptRequest) -> AcceptReply {
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

        inst.last_ballot = inst.ballot;

        // allow inst.ballot to be None
        if Some(ballot) >= inst.ballot {
            inst.ballot = Some(ballot);

            inst.final_deps = req.final_deps.clone();
            self.storage.set_instance(&inst)?;
        }

        Ok(inst)
    }

    pub fn handle_commit(&mut self, req: &CommitRequest) -> CommitReply {
        // TODO protocol wrapping may be better to be in server impl instead of being here
        // TODO check to_replica_id

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
    ) -> Result<(BallotNum, InstanceID), Error> {
        let cm = match cm {
            Some(v) => v,
            None => return Err(Error::LackOf("cmn".into())),
        };

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

    fn _get_instance(&mut self, iid: InstanceID) -> Result<Instance, Error> {
        let inst = self.storage.get_instance(iid)?;

        let inst = match inst {
            Some(inst) => inst,
            // not found
            None => self._empty_instance(Some(iid)),
        };

        Ok(inst)
    }

    fn _empty_instance(&self, iid: Option<InstanceID>) -> Instance {
        Instance {
            instance_id: iid,
            ..Default::default()
        }
    }
}
