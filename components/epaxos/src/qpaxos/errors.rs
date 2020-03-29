use crate::qpaxos::{InvalidRequest, QError, ReplicaID};

quick_error! {
    #[derive(Debug)]
    pub enum ProtocolError {
        NoSuchReplica(rid: ReplicaID, my_rid: ReplicaID) {
            from(ids: (ReplicaID, ReplicaID)) -> (ids.0, ids.1)
            display("no such replica:{}, my replica_id:{}", rid, my_rid)
        }

        LackOf(field: String) {
            display("lack of required field:{}", field)
        }
    }
}

impl Into<QError> for ProtocolError {
    fn into(self) -> QError {
        match self {
            Self::NoSuchReplica(rid, my_rid) => QError {
                req: Some(InvalidRequest {
                    field: "cmn.to_replica_id".into(),
                    problem: "NotFound".into(),
                    ctx: format!("{}; my replica_id: {}", rid, my_rid).into(),
                }),
                ..Default::default()
            },

            Self::LackOf(f) => QError {
                req: Some(InvalidRequest {
                    field: f.clone(),
                    problem: "LackOf".into(),
                    ctx: "".into(),
                }),
                ..Default::default()
            },
        }
    }
}
