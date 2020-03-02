use crate::qpaxos::{Instance, InstanceID, OpCode};
use crate::replica::{errors::Error, Replica};
use crate::snapshot::Error as SnapError;

#[cfg(test)]
#[path = "./tests/exec_tests.rs"]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecuteResult {
    Success,
    SuccessWithVal { value: Vec<u8> },
    NotFound,
}

impl Replica {
    // R1          R2
    // -------------
    // |           |
    // d(NotFound) |
    // | ↖ ........b(Committed)
    // a(Executed) |
    //
    // instances((a, d]) not exist in this replica, recover instance(a+1) first
    fn find_missing_insts(
        &self,
        min_insts: &Vec<Instance>,
        exec_up_to: &Vec<InstanceID>,
    ) -> Option<Vec<InstanceID>> {
        let mut rst = Vec::new();

        for inst in min_insts {
            for dep_inst_id in inst.final_deps.iter() {
                if let Some(_) = min_insts.iter().find(|x| match &x.instance_id {
                    Some(iid) => return iid.replica_id == dep_inst_id.replica_id,
                    None => return false,
                }) {
                    continue;
                }

                if let Some(iid) = exec_up_to
                    .iter()
                    .find(|x| x.replica_id == dep_inst_id.replica_id)
                {
                    if dep_inst_id.idx > iid.idx {
                        rst.push(InstanceID::from((iid.replica_id, iid.idx + 1)));
                    }
                }
            }
        }

        if rst.len() > 0 {
            return Some(rst);
        }

        None
    }

    fn execute_commands(&mut self, inst: &Instance) -> Result<Vec<ExecuteResult>, Error> {
        let mut rst = Vec::new();
        for cmd in inst.cmds.iter() {
            if OpCode::NoOp as i32 == cmd.op {
                rst.push(ExecuteResult::Success);
            } else if OpCode::Set as i32 == cmd.op {
                self.storage.set_kv(cmd.key.clone(), cmd.value.clone())?;
                rst.push(ExecuteResult::Success);
            } else if OpCode::Get as i32 == cmd.op {
                match self.storage.get_kv(&cmd.key) {
                    Ok(r) => rst.push(ExecuteResult::SuccessWithVal { value: r }),
                    Err(e) => match e {
                        SnapError::NotFound => rst.push(ExecuteResult::NotFound),
                        _ => return Err(Error::from(e)),
                    },
                }
            } else {
                return Err(Error::CmdNotSupport(format!("{:?}", cmd.op)));
            }
        }

        let mut new_inst = inst.clone();
        new_inst.executed = true;
        self.storage.set_instance(&new_inst)?;

        Ok(rst)
    }

    /// Find out the set of smallest instances of every leader: S
    /// If there are any a → b relations(a.final_deps ⊃ b.final_deps) in S,
    /// replace S with: S = S - {y | x ∈ S, y ∈ S and (∃y: y → x)}
    /// Execute all instances in S in instance-id-order
    ///
    /// an examle:
    /// S = {a, b, c, d, e}
    /// a → b → c
    /// S = S - {y | x ∈ S, y ∈ S and (∃y: y → x)}
    /// so S = {c, d, e}
    /// sort S by instance_id and execute
    fn execute_instances(&mut self, insts: &Vec<Instance>) -> Result<Vec<InstanceID>, Error> {
        let mut can_exec: Vec<&Instance> = insts
            .iter()
            .filter(|&x| {
                if let Some(_) = insts
                    .iter()
                    .find(|&y| y.instance_id.unwrap() != x.instance_id.unwrap() && x.after(y))
                {
                    return false;
                }

                return true;
            })
            .collect();

        can_exec.sort_by(|a, b| a.instance_id.unwrap().cmp(&b.instance_id.unwrap()));

        let mut rst = Vec::new();
        let mut replys = Vec::new();
        self.storage.trans_begin();
        for inst in can_exec.iter() {
            match self.execute_commands(inst) {
                Ok(r) => replys.push(r),
                Err(e) => {
                    self.storage.trans_rollback()?;
                    return Err(e);
                }
            };
            rst.push(inst.instance_id.unwrap());
        }

        match self.storage.trans_commit() {
            // TODO send replys to client
            Ok(_) => {}
            Err(e) => {
                self.storage.trans_rollback()?;
                return Err(Error::from(e));
            }
        }

        Ok(rst)
    }
}
