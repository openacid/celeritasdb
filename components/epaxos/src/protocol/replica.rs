use std::net::{SocketAddr, TcpStream, TcpListener};
use std::thread::JoinHandle;
use std::collections::HashMap;
use std::error::Error;

struct BallotNum {
    epoch: uint32,
    num: uint32,
    replica: int32,
}

struct ReplicaID(int32);

// information of communication peer
struct ReplicaPeer {
    replica: ReplicaID,
    addr: SocketAddr,      // ip: port pairs of each replica
    conn: TcpStream,       // tcp connection cache
    alive: bool,           // if peer is alive or not
    listener: TcpListener, // tcp listener to get msg, TODO(lsl): may not necessary
}

// misc configuration info
struct ReplicaConf {
    thrifty: bool, // send msg only to a quorum or the full set
    exec: bool,    // exec comamnd or not
    dreply: bool,  // delay replying to client after command has been executed or not
    beacon: bool,  // periodicity detect the speed of each known replica or not
}

// status of a replica
enum Status {
    Joining,
    Running,
    Shutdown,
}

// structure to represent a replica
struct Replica {
    pub id: ReplicaID,                      // replica id
        status: Status,                         // status record used internally
        client_listener: TcpListener, // tcp listener to client
        replica_listener: TcpListener,     // tcp listener for replicas
        peers: Vec<ReplicaPeer>,                // peers in communication, if need access from multi-thread, wrap it by Arc<>
        conf: ReplicaConf,                      // misc conf

        // TODO(lsl): store state value, aka HashMap<key, value> inside state machine？
        // we may use a channel to communicate with the exec-threads, TBD ?
        stateMachine: StateMachine, // state machine to store and execute commands,
        // FIXME(lsl): what is it used for ? still go code.
        // I think it should be a record in StateMachine to store all instances known from all
        // replicas
        // instances_space: [][]*Instance, // the space of all instances (used and not yet used)
        // I think the conflicts should be in StatMachine
        // conflicts: []map[state.Key]int32,
        // put it in StatMachine to record max seq of each (cmd|instance) ?
        // maxSeqPerKey: map[state.Key]int32,

        // protocol part
        instance_num: InstanceNum,

        crt_instance: Vec<(ReplicaID, InstanceNum)>, // highest active instance numbers that this replica knows about
        committed_upto: Vec<(ReplicaID, InstanceNum)>, // highest committed instance per replica that this replica knows about
        executed_upto: Vec<(ReplicaID, int32)>, // instance up to which all commands have been executed (including itself)

        // TODO(lsl): JoinHandle may not enough, we need a wrapper to contain JoinHandle and data used by
        // exec thread
        exec_worker: JoinHandle, // handle of exec thread
        maxSeq: int32, // max seq ever known in cluster

        // FIXME(lsl): what is it used for ? see only assignment but never used
        // latestCPReplica: ReplicaID, // record the replica peer in the latest communication
        // latestCPInstance: InstanceNum,    // record the instance id in the latest communication
        // combine the above two fields in latestCP
        latestCP: InstanceID, // record the instance id in the lastest communication

        // FIXME(lsl): this channel is used to do recovery.
        // as discussed, we use a in ballot number(epoch.a.replica) as a hint of recovery,
        // so instancesToRecover is not necessary to us.
        // instancesToRecover: chan *instanceId,
}

impl Replica {
    /// create a new Replica
    /// do all the initialization and start all necessary threads here,
    /// so after this call, replica is fully functional.
    pub fn new(replica: ReplicaID, cluster: &ClusterInfo, thrifty: bool, exec: bool, beacon: bool,) -> Result<Replica, Error> {
    }

    /// handle a client request
    /// this is the main logic to implement choosing commands in epaxos protocol
    /// as described in figure 2.
    pub fn propose(&mut self, cmds: &Vec<Command>) {
    }

    /// do graceful shutdown of epaxos replica part
    pub fn shutdown() {
    }

    /// send a message
    /// if conn is None, make connection
    /// TODO(lsl): we really need something like a request context
    ///            to store conn and anything should be shared in a request or proposing an instance
    fn send_msg(&mut self, conn: Option(&mut TcpStream), to: ReplicaID, msg: &Message) -> (&mut TcpStream, Result<Message, Error>) {
    }

    /// start exec thread
    fn start_exec_thread(&mut self) {
    }

    /// this is the main logic to implement recovery procedure in epaxos protocol
    /// as described in figure 3.
    fn explicit_prepare(&mut self, instance: &Instance) {
    }

    // FIXME(lsl): these methods predestined to run in multi-thread and change self,
    //             so is it good to implement as a method or a function to take Replica as arg?
    //             but for now, it doesn't matter since we just want the interface.

    /// send prepare request and receive reponse, return updated reponse or error.
    fn make_prepare_req(&mut self, cmds: &Vec<Command>) -> Result<PrepareReply, Error> {
        // ReplicaID and ballot are stored in self.
    }

    // ASK(lsl->xp): is it ok to return a result and ask http-framework to judge type and send
    // reply to requester?
    // I think that would be a great place to do statistics and measurement.
    // and by making Replica independent,it's easier to do test.
    fn handle_prepare(&mut self, req: &PrepareReq) -> Result<PrepareReply, Error> {
    }

    fn make_pre_accept_req(&mut self, cmds: &Vec<Command>) -> Result<PreAcceptReply, Error> {
        // instance id, ballot, seq and deps are constructed right here.
    }

    fn handle_pre_accept(&mut self, req: &PreAcceptReq) -> Result<PreAcceptReply, Error>{
    }

    fn make_try_pre_accept_req(&mut self, instance: &Instance) -> Result<TryPreAcceptReply, Error> {
    }

    fn handle_try_pre_accept(&mut self, req: &TryPreAcceptReq) -> Result<TryPreAcceptReply, Error> {
    }

    // func (r *Replica) bcastAccept(replica int32, instance int32, ballot int32, count int32, seq int32, deps [DS]int32) {
    fn make_accept_req(&mut self, instance: &Instance) -> Result<AcceptReply, Error> {
    }

    fn handle_accept(&mut self, req: &AcceptReq) -> Result<AcceptReply, Error> {
    }

    fn make_commit_req(&mut self, instance: &Instance) -> Result<CommitReply, Error> {
    }

    fn handle_commit(&mut self, req: &CommitReq) -> Result<CommitReply, Error> {
    }
}
