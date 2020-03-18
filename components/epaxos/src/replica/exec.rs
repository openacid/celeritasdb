use std::time::SystemTime;

use crate::qpaxos::{Instance, InstanceId, OpCode};
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
    fn recover_instances(&self, inst_ids: &Vec<InstanceId>) {}
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
        exec_up_to: &Vec<InstanceId>,
    ) -> Option<Vec<InstanceId>> {
        let mut rst: Vec<InstanceId> = Vec::new();

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
                        rst.push((iid.replica_id, iid.idx + 1).into());
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

    /// Find out the set of smallest instances of every leader: S.
    /// If there are any a → b relations(a.final_deps ⊃ b.final_deps) in S,
    /// replace replace S with: S = {x | x ∈ S and (∃y: y → x)},
    /// until there is no a → b in S.
    /// Execute all instances in S in instance-id-order
    ///
    /// an examle:
    /// S = {a, b, c, d, e}; a → b  c → d ~ e
    /// S = {x | x ∈ S and (∃y: y → x)}
    /// so S = {b, d}
    /// sort S by instance_id and execute
    fn execute_instances(&mut self, insts: &Vec<Instance>) -> Result<Vec<InstanceId>, Error> {
        let mut early = vec![false; insts.len()];
        let mut late = vec![false; insts.len()];
        let mut can_exec = Vec::with_capacity(insts.len());
        for (i, x) in insts.iter().enumerate() {
            for (j, y) in insts.iter().enumerate() {
                if x.after(y) {
                    early[j] = true;
                    late[i] = true;
                }
            }
        }

        for (i, inst) in insts.iter().enumerate() {
            if !late[i] && early[i] {
                can_exec.push(inst);
            }
        }

        if can_exec.len() == 0 {
            can_exec = insts.iter().collect();
        }

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

    // only save one smallest problem instance of every replica with problem_inst_ids.
    // when find a new problem instance just replace it if instance of this replica
    // already in problem_inst_ids.
    fn timeout_to_committed(&mut self, iid: InstanceId) -> bool {
        let now = SystemTime::now();
        if let Some(p) = self.problem_inst_ids.iter().find(|x| x.0 == iid) {
            let dt = now.duration_since(p.1).unwrap();
            if dt.as_millis() as i32 >= self.conf.inst_committed_timeout {
                return true;
            }
        } else {
            match self
                .problem_inst_ids
                .iter_mut()
                .find(|x| x.0.replica_id == iid.replica_id)
            {
                Some(p) => {
                    p.0 = iid;
                    p.1 = now;
                }
                None => self.problem_inst_ids.push((iid, now)),
            }
        }

        false
    }

    fn get_insts_if_committed(
        &mut self,
        inst_ids: &Vec<InstanceId>,
    ) -> Result<Vec<Instance>, Error> {
        let mut rst = Vec::new();
        let mut recover_iids = Vec::new();

        for iid in inst_ids.iter() {
            let inst = match self.storage.get_instance(*iid)? {
                Some(i) => i,
                None => {
                    recover_iids.push(*iid);
                    continue;
                }
            };
            if inst.committed {
                rst.push(inst);
                continue;
            }

            if self.timeout_to_committed(*iid) {
                recover_iids.push(*iid);
            }
        }

        self.recover_instances(&recover_iids);
        Ok(rst)
    }

    fn execute(&mut self) -> Result<Vec<InstanceId>, Error> {
        let mut exec_up_to = Vec::new();
        let mut smallest_inst_ids = Vec::new();
        for rid in self.group_replica_ids.iter() {
            let exec_iid = self.storage.get_ref("exec", *rid)?;
            let max_iid = self.storage.get_ref("max", *rid)?;
            exec_up_to.push(exec_iid);
            if exec_iid < max_iid {
                smallest_inst_ids.push((*rid, exec_iid.idx + 1).into());
            }
        }

        let instances = self.get_insts_if_committed(&smallest_inst_ids)?;
        if instances.len() == 0 {
            return Ok(vec![]);
        }

        if instances.len() < self.group_replica_ids.len() {
            if let Some(iids) = self.find_missing_insts(&instances, &exec_up_to) {
                self.recover_instances(&iids);
                return Ok(vec![]);
            }
        }

        return self.execute_instances(&instances);
    }
}
