use crate::qpaxos::replicate_reply;
use crate::qpaxos::replicate_request;
use crate::qpaxos::AcceptReply;
use crate::qpaxos::AcceptRequest;
use crate::qpaxos::BallotNum;
use crate::qpaxos::Command;
use crate::qpaxos::CommitReply;
use crate::qpaxos::CommitRequest;
use crate::qpaxos::Dep;
use crate::qpaxos::Deps;
use crate::qpaxos::FastAcceptReply;
use crate::qpaxos::FastAcceptRequest;
use crate::qpaxos::Instance;
use crate::qpaxos::InstanceId;
use crate::qpaxos::InstanceIdVec;
use crate::qpaxos::InvalidRequest;
use crate::qpaxos::OpCode;
use crate::qpaxos::QError;
use crate::qpaxos::ReplicateReply;
use crate::qpaxos::ReplicateRequest;
use crate::qpaxos::StorageFailure;
use std::fmt;

pub trait ToStringExt {
    fn tostr_ext(&self) -> String;
}

impl ToStringExt for String {
    fn tostr_ext(&self) -> String {
        format!("{}", self)
    }
}

impl<T: ToStringExt> ToStringExt for Option<T> {
    fn tostr_ext(&self) -> String {
        match self {
            Some(v) => v.tostr_ext(),
            None => "None".into(),
        }
    }
}

impl<T: ToStringExt> ToStringExt for Vec<T> {
    fn tostr_ext(&self) -> String {
        let mut rst = String::from("[");
        for (i, x) in self.iter().enumerate() {
            rst.push_str(&x.tostr_ext());
            if i < self.len() - 1 {
                rst.push_str(", ");
            }
        }
        rst.push_str("]");
        rst
    }
}

impl ToStringExt for InstanceIdVec {
    fn tostr_ext(&self) -> String {
        self.ids.tostr_ext()
    }
}

impl ToStringExt for Deps {
    fn tostr_ext(&self) -> String {
        self.dep_vec.tostr_ext()
    }
}

impl ToStringExt for Command {
    fn tostr_ext(&self) -> String {
        match self.op {
            v if v == (OpCode::NoOp as i32) => format!("NoOp"),
            v if v == (OpCode::Get as i32) => format!("Get:{}", String::from_utf8_lossy(&self.key)),
            v if v == (OpCode::Set as i32) => format!(
                "Set:{}={}",
                String::from_utf8_lossy(&self.key),
                String::from_utf8_lossy(&self.value),
            ),
            v if v == (OpCode::Delete as i32) => {
                format!("Delete:{}", String::from_utf8_lossy(&self.key),)
            }
            _ => format!("UnknownCmd"),
        }
    }
}

impl ToStringExt for replicate_request::Phase {
    fn tostr_ext(&self) -> String {
        match self {
            Self::Fast(v) => format!("Fast{}", v),
            Self::Accept(v) => format!("Accept{}", v),
            Self::Commit(v) => format!("Commit{}", v),
        }
    }
}

impl ToStringExt for replicate_reply::Phase {
    fn tostr_ext(&self) -> String {
        match self {
            Self::Fast(v) => format!("Fast{}", v),
            Self::Accept(v) => format!("Accept{}", v),
            Self::Commit(v) => format!("Commit{}", v),
        }
    }
}

macro_rules! impl_tostr_ext {
    ($typ:path) => {
        impl ToStringExt for $typ {
            fn tostr_ext(&self) -> String {
                format!("{}", self)
            }
        }
    };

    ($typ:path, $fmt:expr, $($field:tt),*) => {
        impl ToStringExt for $typ {
            fn tostr_ext(&self) -> String {
                format!($fmt, $(self.$field.tostr_ext()),*)
            }
        }
    };
}

impl_tostr_ext!(bool);
impl_tostr_ext!(i64);
impl_tostr_ext!(i32);
impl_tostr_ext!(InstanceId, "({}, {})", replica_id, idx);
impl_tostr_ext!(Dep, "({}, {}, {})", replica_id, idx, seq);
impl_tostr_ext!(BallotNum, "({}, {}, {})", epoch, num, replica_id);
impl_tostr_ext!(
    Instance,
    "{{id:{}, blt:{}, ablt:{}, cmds:{}, deps:{}, c/e:{}/{}}}",
    instance_id,
    ballot,
    vballot,
    cmds,
    deps,
    committed,
    executed
);

// replication requests

impl_tostr_ext!(
    ReplicateRequest,
    "{{to:{}, blt:{}, iid:{}, phase:{}}}",
    to_replica_id,
    ballot,
    instance_id,
    phase
);

impl_tostr_ext!(
    FastAcceptRequest,
    "{{cmds:{}, deps:{}, c:{}}}",
    cmds,
    deps,
    deps_committed
);

impl_tostr_ext!(AcceptRequest, "{{deps:{}}}", deps);
impl_tostr_ext!(CommitRequest, "{{cmds:{}, deps:{}}}", cmds, deps);

// replication replies

impl_tostr_ext!(
    ReplicateReply,
    "{{err:{}, last:{}, iid:{}, phase:{}}}",
    err,
    last_ballot,
    instance_id,
    phase
);

impl_tostr_ext!(
    FastAcceptReply,
    "{{deps[1]:{}, c:{}}}",
    deps,
    deps_committed
);

impl_tostr_ext!(AcceptReply, "{{}}",);
impl_tostr_ext!(CommitReply, "{{}}",);

// replication errors

impl_tostr_ext!(QError, "{{sto:{}, req:{}}}", sto, req);
impl_tostr_ext!(StorageFailure, "StorageFailure",);
impl_tostr_ext!(InvalidRequest, "{{{}: '{}', ctx:{}}}", problem, field, ctx);

macro_rules! impl_display {
    ($typ:path) => {
        impl fmt::Display for $typ {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.tostr_ext())
            }
        }
    };
}

impl_display!(InstanceIdVec);
impl_display!(Command);
impl_display!(InstanceId);
impl_display!(BallotNum);
impl_display!(Instance);

impl_display!(ReplicateRequest);
impl_display!(FastAcceptRequest);
impl_display!(AcceptRequest);
impl_display!(CommitRequest);

impl_display!(ReplicateReply);
impl_display!(FastAcceptReply);
impl_display!(AcceptReply);
impl_display!(CommitReply);

impl_display!(QError);
impl_display!(StorageFailure);
impl_display!(InvalidRequest);
