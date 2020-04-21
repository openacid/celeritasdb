use crate::qpaxos::BallotNum;
use crate::qpaxos::Command;
use crate::qpaxos::Instance;
use crate::qpaxos::InstanceId;
use crate::qpaxos::InstanceIdVec;
use crate::qpaxos::OpCode;
use std::fmt;

trait ToStringExt {
    fn tostr_ext(&self) -> String;
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
impl_tostr_ext!(BallotNum, "({}, {}, {})", epoch, num, replica_id);
impl_tostr_ext!(
    Instance,
    "{{id:{}, blt:{}, cmds:{}, deps:{}{}{}, c/e:{}/{}}}",
    instance_id,
    ballot,
    cmds,
    initial_deps,
    deps,
    final_deps,
    committed,
    executed
);

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
