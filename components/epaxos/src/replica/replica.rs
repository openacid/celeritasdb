use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::SystemTime;

use super::super::conf::ClusterInfo;

use super::super::qpaxos::*;

use super::super::snapshot::TxEngine;

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
#[derive(Default)]
pub struct ReplicaConf {
    pub thrifty: bool,        // send msg only to a quorum or the full set
    pub exec: bool,           // exec comamnd or not
    pub dreply: bool,         // delay replying to client after command has been executed or not
    pub beacon: bool,         // periodicity detect the speed of each known replica or not
    pub inst_co_timeout: i32, // instance committed timeout
}

/// status of a replica
pub enum ReplicaStatus {
    Joining,
    Running,
    ShuttingDown,
    Down,
}

/// structure to represent a replica
pub struct Replica<E> {
    pub replica_id: ReplicaID,             // replica id
    pub group_replica_ids: Vec<ReplicaID>, // all replica ids in this group
    pub status: ReplicaStatus,             // status record used internally
    pub client_listener: TcpListener,      // tcp listener to client
    pub listener: TcpListener,             // tcp listener for replicas
    pub peers: Vec<ReplicaPeer>, // peers in communication, if need access from multi-thread, wrap it by Arc<>
    pub conf: ReplicaConf,       // misc conf

    pub inst_idx: InstanceIdx,
    pub latest_cp: InstanceID, // record the instance id in the lastest communication

    // storage
    pub storage: Box<dyn TxEngine<E>>,

    // to recover uncommitted instance
    pub problem_inst_ids: Vec<(InstanceID, SystemTime)>,
}

impl<E> Replica<E> {
    /// create a new Replica
    /// do all the initialization and start all necessary threads here,
    /// so after this call, replica is fully functional.
    pub fn new(
        replica_id: ReplicaID,
        cluster: &ClusterInfo,
        thrifty: bool,
        exec: bool,
        beacon: bool,
    ) -> Result<Replica<E>, String> {
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

    fn handle_accept(&mut self, req: &AcceptRequest) -> Result<AcceptReply, String> {
        Err("not implemented".to_string())
    }

    fn handle_commit(&mut self, req: &CommitRequest) -> Result<(), String> {
        Err("not implemented".to_string())
    }
}
