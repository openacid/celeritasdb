use super::traits::*;
use crate::qpaxos::{Instance, InstanceID};
use prost::Message;

use crate::tokey::ToKey;

pub struct InstanceIter<'a> {
    pub curr_inst_id: InstanceID,
    pub include: bool,
    pub engine: &'a dyn Base,
}

impl<'a> Iterator for InstanceIter<'a> {
    type Item = Instance;

    fn next(&mut self) -> Option<Instance> {
        let k = self.curr_inst_id.to_key();
        let (key_bytes, val_bytes) = self.engine.next_kv(&k, self.include)?;

        let key = String::from_utf8(key_bytes).unwrap();
        let iid = InstanceID::from_key(&key[..])?;

        match Instance::decode(val_bytes.as_slice()) {
            Ok(v) => {
                if iid.replica_id == self.curr_inst_id.replica_id {
                    self.curr_inst_id = iid;
                    self.include = false;
                    Some(v)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::mem_engine::*;
    use super::*;
    use crate::qpaxos::{Command, OpCode};

    #[test]
    fn test_instanceIter() {
        let mut ints = Vec::<Instance>::new();
        let mut engine = MemEngine::new().unwrap();

        for rid in 0..3 {
            for idx in 0..10 {
                let iid = InstanceID::from((rid, idx));

                let cmds = vec![Command::of(
                    OpCode::NoOp,
                    format!("k1{:}", rid * idx).as_bytes(),
                    format!("v1{:}", rid * idx).as_bytes(),
                )];

                let ballot = (rid as i32, idx as i32, 0).into();
                let deps = vec![InstanceID::from((rid + 1, idx + 1))];
                let inst = Instance::of(&cmds[..], &ballot, &deps[..]);

                let _ = engine.set_instance(iid, inst.clone()).unwrap();

                let act = engine.get_instance(&iid).unwrap();
                assert_eq!(act.cmds, cmds);

                ints.push(inst);
            }
        }

        let cases = vec![
            (InstanceID::from((0, 0)), true, &ints[..10]),
            (InstanceID::from((0, 0)), false, &ints[1..10]),
            (InstanceID::from((2, 0)), true, &ints[20..3 * 10]),
            (InstanceID::from((4, 0)), true, &ints[ints.len()..]),
        ];

        for (start_iid, include, exp_insts) in cases {
            let mut n = 0;

            let iter = InstanceIter {
                curr_inst_id: start_iid,
                include: include,
                engine: &engine,
            };

            for act_inst in iter {
                assert_eq!(act_inst.cmds, exp_insts[n].cmds);
                assert_eq!(act_inst.ballot, exp_insts[n].ballot);

                assert_eq!(act_inst.instance_id, exp_insts[n].instance_id);

                n = n + 1;
            }

            assert_eq!(exp_insts.len(), n);
        }
    }
}
