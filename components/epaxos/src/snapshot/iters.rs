use super::traits::*;
use crate::qpaxos::*;

use crate::tokey::ToKey;

pub struct InstanceIter<'a> {
    pub curr_inst_id: InstanceId,
    pub include: bool,
    pub engine: &'a dyn InstanceEngine<ColumnId = ReplicaID, Obj = Instance, ObjId = InstanceId>,
    pub reverse: bool,
}

impl<'a> Iterator for InstanceIter<'a> {
    type Item = Instance;

    fn next(&mut self) -> Option<Instance> {
        let k = self.curr_inst_id.to_key();
        let (key_bytes, val_bytes) = if self.reverse {
            self.engine.prev_kv(&k, self.include)?
        } else {
            self.engine.next_kv(&k, self.include)?
        };

        let key = String::from_utf8(key_bytes);
        let key = match key {
            Ok(v) => v,
            Err(_) => {
                // this is not a key of instance id, done
                return None;
            }
        };

        let iid = InstanceId::from_key(&key[..]);
        let iid = match iid {
            Some(v) => v,
            None => {
                // this is not a key of instance id, done
                return None;
            }
        };

        if iid.replica_id != self.curr_inst_id.replica_id {
            // out of bound, done
            return None;
        }

        let inst = self.engine.decode_obj(&val_bytes);
        let inst = match inst {
            Ok(v) => v,
            Err(e) => {
                // TODO handle data damaging.
                // TODO add test of data corruption
                panic!(e);
            }
        };

        self.curr_inst_id = iid;
        self.include = false;

        Some(inst)
    }
}

#[cfg(test)]
mod tests {
    use super::super::mem_engine::*;
    use super::*;
    use crate::qpaxos::{Command, OpCode};

    #[test]
    fn test_instance_iter() {
        let mut ints = Vec::<Instance>::new();
        let mut engine = MemEngine::new().unwrap();

        for rid in 0..3 {
            for idx in 0..10 {
                let iid = InstanceId::from((rid, idx));

                let cmds = vec![Command::of(
                    OpCode::NoOp,
                    format!("k1{:}", rid * idx).as_bytes(),
                    format!("v1{:}", rid * idx).as_bytes(),
                )];

                let ballot = (rid as i32, idx as i32, 0).into();
                let deps = vec![InstanceId::from((rid + 1, idx + 1))];
                let mut inst = Instance::of(&cmds[..], ballot, &deps[..]);
                inst.instance_id = Some(iid);

                let _ = engine.set_instance(&inst).unwrap();

                let act = engine.get_obj(iid).unwrap().unwrap();
                assert_eq!(act.cmds, cmds);

                ints.push(inst);
            }
        }

        let cases = vec![
            (InstanceId::from((0, 0)), true, &ints[..10], &ints[0..1]),
            (InstanceId::from((0, 0)), false, &ints[1..10], &[]),
            (InstanceId::from((2, 0)), true, &ints[20..30], &ints[20..21]),
            (InstanceId::from((2, 9)), true, &ints[29..30], &ints[20..30]),
            (InstanceId::from((4, 0)), true, &[], &[]),
        ];

        for (start_iid, include, exp_insts, rev_exp_insts) in cases {
            let mut n = 0;

            let iter = engine.get_instance_iter(start_iid, include, false);

            for act_inst in iter {
                assert_eq!(act_inst.cmds, exp_insts[n].cmds);
                assert_eq!(act_inst.ballot, exp_insts[n].ballot);

                assert_eq!(act_inst.instance_id, exp_insts[n].instance_id);

                n = n + 1;
            }

            assert_eq!(exp_insts.len(), n);

            n = 0;
            let iter = engine.get_instance_iter(start_iid, include, true);

            let mut exp = vec![];
            exp.extend(rev_exp_insts.iter().rev());
            for act_inst in iter {
                assert_eq!(act_inst.cmds, exp[n].cmds);
                assert_eq!(act_inst.ballot, exp[n].ballot);

                assert_eq!(act_inst.instance_id, exp[n].instance_id);

                n = n + 1;
            }

            assert_eq!(exp.len(), n);
        }
    }
}
