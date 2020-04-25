use crate::qpaxos::{InvalidRequest, QError, ReplicaId};

quick_error! {
    #[derive(Debug, Eq, PartialEq)]
    pub enum ProtocolError {
        NoSuchReplica(rid: ReplicaId, my_rid: ReplicaId) {
            from(ids: (ReplicaId, ReplicaId)) -> (ids.0, ids.1)
            display("no such replica:{}, my replica_id:{}", rid, my_rid)
        }

        LackOf(field: String) {
            display("lack of required field:{}", field)
        }

        NotMatch(field: String, want: String, got: String) {
            display("field: {} expect: {} but: {}", field, want, got)
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

            // TODO not InvalidRequest but InvalidReply
            Self::NotMatch(f, _want, _got) => QError {
                req: Some(InvalidRequest {
                    field: f.clone(),
                    problem: "NotMatch".into(),
                    ctx,
                }),
                ..Default::default()
            },

            // TODO rename InvalidRequest to InvalidMessage in order to use it with request and
            // reply.
            Self::Incomplete(f, ..) => QError {
                req: Some(InvalidRequest {
                    field: f.clone(),
                    problem: "Incomplete".into(),
                    ctx,
                }),
                ..Default::default()
            },
        }
    }
}
