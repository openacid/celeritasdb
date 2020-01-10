use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread::JoinHandle;

use super::super::command::Command;
use super::super::conf::ClusterInfo;
use super::super::instance::{InstIDs, Instance, InstanceID, InstanceIdx};
use super::super::message::*;

#[cfg(test)]
#[path = "./tests/replica_tests.rs"]
mod tests;

pub type ReplicaID = i64;

/// information of communication peer
pub struct ReplicaPeer {
    pub replica_id: ReplicaID,
    pub addr: SocketAddr, // ip: port pairs of each replica
    pub conn: TcpStream,  // tcp connection cache
    pub alive: bool,      // if peer is alive or not
}

/// misc configuration info
pub struct ReplicaConf {
    pub thrifty: bool, // send msg only to a quorum or the full set
    pub exec: bool,    // exec comamnd or not
    pub dreply: bool,  // delay replying to client after command has been executed or not
    pub beacon: bool,  // periodicity detect the speed of each known replica or not
}

/// status of a replica
pub enum ReplicaStatus {
    Joining,
    Running,
    ShuttingDown,
    Down,
}

// TODO(lsl): use defination from @yipu
pub struct SMR {}

/// structure to represent a replica
pub struct Replica {
    pub replica_id: ReplicaID,        // replica id
    pub status: ReplicaStatus,        // status record used internally
    pub client_listener: TcpListener, // tcp listener to client
    pub listener: TcpListener,        // tcp listener for replicas
    pub peers: Vec<ReplicaPeer>, // peers in communication, if need access from multi-thread, wrap it by Arc<>
    pub conf: ReplicaConf,       // misc conf

    pub smr: SMR, // state machine replication

    pub inst_idx: InstanceIdx,
    pub crt_inst: InstIDs, // highest active instance numbers that this replica knows
    pub replica_committed: InstIDs, // highest continuous committed instance per replica that known
    pub replica_executed: InstIDs,  // highest executed instance per replica that known

    // TODO(lsl): get exec thread handle from @baohai
    pub exec_worker: JoinHandle<()>, // handle of exec thread
    pub max_seq: i64,                // max seq ever known in cluster
    pub latest_cp: InstanceID,       // record the instance id in the lastest communication
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
    fn send_msg(&mut self, to: ReplicaID, msg: &Message) -> Result<Message, String> {
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

    /// send prepare request and receive reponse, return updated reponse or error.
    fn make_prepare_req(&mut self, cmds: &Vec<Command>) -> Result<PrepareReply, String> {
        // ReplicaID and ballot are stored in self.
        Err("not implemented".to_string())
    }

    // ASK(lsl->xp): is it ok to return a result and ask http-framework to judge type and send
    // reply to requester?
    // I think that would be a great place to do statistics and measurement.
    // and by making Replica independent,it's easier to do test.
    fn handle_prepare(&mut self, req: &PrepareReq) -> Result<PrepareReply, String> {
        Err("not implemented".to_string())
    }

    fn make_pre_accept_req(&mut self, cmds: &Vec<Command>) -> Result<PreAcceptReply, String> {
        // instance id, ballot, seq and deps are constructed right here.
        Err("not implemented".to_string())
    }

    fn handle_pre_accept(&mut self, req: &PreAcceptReq) -> Result<PreAcceptReply, String> {
        Err("not implemented".to_string())
    }

    fn make_try_pre_accept_req(
        &mut self,
        instance: &Instance,
    ) -> Result<TryPreAcceptReply, String> {
        Err("not implemented".to_string())
    }

    fn handle_try_pre_accept(
        &mut self,
        req: &TryPreAcceptReq,
    ) -> Result<TryPreAcceptReply, String> {
        Err("not implemented".to_string())
    }

    fn make_accept_req(&mut self, instance: &Instance) -> Result<AcceptReply, String> {
        Err("not implemented".to_string())
    }

    fn handle_accept(&mut self, req: &AcceptReq) -> Result<AcceptReply, String> {
        Err("not implemented".to_string())
    }

    fn make_commit_req(&mut self, instance: &Instance) -> Result<(), String> {
        Err("not implemented".to_string())
    }

    fn handle_commit(&mut self, req: &CommitReq) -> Result<(), String> {
        Err("not implemented".to_string())
    }
}
