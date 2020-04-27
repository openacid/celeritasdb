use std::cell::RefCell;
use std::collections::HashMap;
use std::time::SystemTime;

use crate::qpaxos::{Instance, InstanceId, InstanceIdVec, OpCode};
use crate::replica::ExecuteResult;
use crate::replica::Replica;
use storage::StorageError;
use storage::WriteEntry;
use tokio::sync::oneshot::Sender;

thread_local! {
    static PROBLEM_INSTS: RefCell<Vec<(InstanceId, SystemTime)>> = RefCell::new(vec![]);
}

impl Replica {
    fn recover_instances(&self, _inst_ids: &InstanceIdVec) {}
    // R1          R2
    // -------------
    // |           |
    // d(NotFound) |
    // | ↖ ........b(Committed)
    // a(Executed) |
    //
    // instances((a, d]) not exist in this replica, recover instance(a+1) first
    pub fn find_missing_insts(
        &self,
        min_insts: &Vec<Instance>,
        exec_up_to: &InstanceIdVec,
    ) -> Option<InstanceIdVec> {
        let mut rst = InstanceIdVec::from([0; 0]);
        let mut iids = InstanceIdVec::from([0; 0]);
        let mut all_dep_iids = InstanceIdVec::from([0; 0]);

        for inst in min_insts {
            iids.push(inst.instance_id.unwrap());
            all_dep_iids.extend(inst.final_deps.as_ref().unwrap().iter());
        }

        for dep_iid in all_dep_iids.iter() {
            if let Some(_) = iids.get(dep_iid.replica_id) {
                continue;
            }

            let missing: InstanceId = match exec_up_to.get(dep_iid.replica_id) {
                None => (dep_iid.replica_id, 0).into(),
                Some(iid) => {
                    if dep_iid.idx <= iid.idx {
                        continue;
                    }

                    (dep_iid.replica_id, iid.idx + 1).into()
                }
            };

            if let Some(_) = rst.get(dep_iid.replica_id) {
                continue;
            }
            rst.push(missing);
        }

        if rst.len() > 0 {
            return Some(rst);
        }

        None
    }

    async fn send_replies(&self, mut replies: Vec<(InstanceId, Vec<ExecuteResult>)>) {
        let mut wrpls = self.waiting_replies.lock().await;
        while let Some((iid, r)) = replies.pop() {
            let tx = match wrpls.remove(&iid) {
                Some(t) => t,
                None => continue,
            };
            if let Err(_) = tx.send(r) {
                println!("the receiver dropped for {:?}", iid);
            }
        }
    }

    pub async fn insert_tx(&self, iid: InstanceId, tx: Sender<Vec<ExecuteResult>>) {
        let mut wrpls = self.waiting_replies.lock().await;
        wrpls.insert(iid, tx);
    }

    pub async fn execute_commands(
        &self,
        mut insts: Vec<Instance>,
    ) -> Result<Vec<InstanceId>, StorageError> {
        let mut rst = Vec::with_capacity(insts.len());
        let mut entrys: Vec<WriteEntry> = Vec::with_capacity(insts.len());
        let mut existed = HashMap::new();
        let mut replies = Vec::with_capacity(insts.len());

        for inst in insts.iter() {
            let iid = inst.instance_id.unwrap();
            rst.push(iid);

            let mut repl = Vec::with_capacity(inst.cmds.len());
            for cmd in inst.cmds.iter() {
                entrys.push(cmd.into());

                if cmd.op == OpCode::Get as i32 {
                    if !existed.contains_key(&cmd.key) {
                        let v = self.storage.get_kv(&cmd.key)?;
                        existed.insert(&cmd.key, v);
                    }
                    repl.push(ExecuteResult::SuccessWithVal(existed[&cmd.key].clone()));
                } else if cmd.op == OpCode::NoOp as i32 {
                    repl.push(ExecuteResult::Success);
                } else {
                    let v = if cmd.op == OpCode::Delete as i32 {
                        None
                    } else {
                        Some(cmd.value.clone())
                    };
                    existed.insert(&cmd.key, v);
                    repl.push(ExecuteResult::Success);
                }
            }

            entrys.push(iid.into());
            replies.push((iid, repl));
        }

        while let Some(mut inst) = insts.pop() {
            inst.executed = true;
            entrys.push(inst.into());
        }

        self.storage.write_batch(&entrys)?;
        self.send_replies(replies).await;
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
    pub async fn execute_instances(
        &self,
        mut insts: Vec<Instance>,
    ) -> Result<Vec<InstanceId>, StorageError> {
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

        for i in (0..late.len()).rev() {
            if !late[i] && early[i] {
                can_exec.push(insts.remove(i));
            }
        }

        if can_exec.len() == 0 {
            can_exec = insts;
        }

        can_exec.sort_by(|a, b| a.instance_id.cmp(&b.instance_id));
        self.execute_commands(can_exec).await
    }

    // only save one smallest problem instance of every replica with problem_inst_ids.
    // when find a new problem instance just replace it if instance of this replica
    // already in problem_inst_ids.
    pub fn timeout_to_committed(&self, iid: InstanceId) -> bool {
        let now = SystemTime::now();
        let mut rst = false;
        PROBLEM_INSTS.with(|p| {
            let mut p = p.borrow_mut();

            if let Some(p) = p.iter().find(|x| x.0 == iid) {
                let dt = now.duration_since(p.1).unwrap();
                if dt.as_millis() as i32 >= self.committed_timeout {
                    rst = true;
                }
            } else {
                match p.iter_mut().find(|x| x.0.replica_id == iid.replica_id) {
                    Some(p) => {
                        p.0 = iid;
                        p.1 = now;
                    }
                    None => p.push((iid, now)),
                }
            }
        });

        rst
    }

    pub fn get_insts_if_committed(
        &self,
        inst_ids: &Vec<InstanceId>,
    ) -> Result<Vec<Instance>, StorageError> {
        let mut rst = Vec::new();
        let mut recover_iids = InstanceIdVec::from([0; 0]);

        for iid in inst_ids {
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

    pub async fn execute(&self) -> Result<Vec<InstanceId>, StorageError> {
        let mut exec_up_to = InstanceIdVec::from([0; 0]);
        let mut smallest_inst_ids = InstanceIdVec::from([0; 0]);
        for rid in self.group_replica_ids.iter() {
            let exec_iid = self.storage.get_ref("exec", *rid)?;
            let max_iid = self.storage.get_ref("max", *rid)?;
            if let None = max_iid {
                continue;
            }

            let exec_iid = exec_iid.unwrap_or((*rid, -1).into());
            let max_iid = max_iid.unwrap();

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

        self.execute_instances(instances).await
    }
}
