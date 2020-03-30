use crate::qpaxos::{InvalidRequest, QError, ReplicaID};

quick_error! {
    #[derive(Debug, Eq, PartialEq)]
    pub enum ProtocolError {
        NoSuchReplica(rid: ReplicaID, my_rid: ReplicaID) {
            from(ids: (ReplicaID, ReplicaID)) -> (ids.0, ids.1)
            display("no such replica:{}, my replica_id:{}", rid, my_rid)
        }

        LackOf(field: String) {
            display("lack of required field:{}", field)
        }

        Incomplete(field: String, want: i32, actual: i32) {
            display("incomplete field:{}, need:{}, but:{}", field, want, actual)
        }
    }
}

impl Into<QError> for ProtocolError {
    fn into(self) -> QError {
        let ctx = format!("{}", self);
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

            // TODO rename InvalidRequest to InvalidMessage in order to use it with request and
            // reply.
            Self::Incomplete(f, ..) => QError {
                req: Some(InvalidRequest {
                    field: f.clone(),
                    problem: "Incomplete".into(),
                    ctx: ctx,
                }),
                ..Default::default()
            },
        }
    }
}
