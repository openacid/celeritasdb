/// protocol buffer serialized
type InstanceNum = int32;
struct InstanceID {
    replica: ReplicaID,
    instance_num: InstanceNum,
}

enum OpCode {
    NoOp,
    Put,
    Get,
}

// FIXME(lsl): for simplicity, string type for both key and value
pub struct Command {
    pub op: Opcode,
    pub key: String,
    pub value: String, // TODO(lsl): must consider type, value and max capacity used
}

enum InstanceStatus {
    NA(String), // status not available means None, but `None` is a key workd.
    PreAcceptedEQ(String),
    PreAccepted(String),
    Accepted(String),
    Committed(String),
    Executed(String),
}

type ReplicaCmd = (ReplicaID, InstanceID);
struct DepCmds(Vec<(ReplicaID, InstanceID)>);

struct Instance {
    status: InstanceStatus,
    cmds: Vec<Command>,
    ballot: BallotNum,
    seq: uint64,
    deps: DepCmds,
}
