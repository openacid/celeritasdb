use crate::qpaxos::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InstanceStatus {
    Na,
    FastAccepted,
    Accepted,
    Committed,
    Executed,
}

impl Instance {
    pub fn status(&self) -> InstanceStatus {
        if self.executed {
            return InstanceStatus::Executed;
        }

        if self.committed {
            return InstanceStatus::Committed;
        }

        if self.final_deps.is_some() {
            return InstanceStatus::Accepted;
        }

        if self.deps.is_some() {
            return InstanceStatus::FastAccepted;
        }

        InstanceStatus::Na
    }
}

pub struct Status {
    pub quorum: i32,
    pub ok_count: i32,
}

impl Status {
    pub fn new(quorum: i32) -> Self {
        Self {
            quorum,
            ok_count: 1,
        }
    }

    pub fn finish(&mut self) -> bool {
        self.ok_count += 1;
        self.ok_count >= self.quorum
    }
}

pub type FastAcceptStatus = Status;
pub type AcceptStatus = Status;
pub type PrepareStatus = Status;
