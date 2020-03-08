use std::collections::BTreeMap;
use std::sync::Mutex;

use super::super::*;
use super::MemEngine;

use crate::qpaxos::*;

impl MemEngine {
    pub fn new() -> Result<MemEngine, Error> {
        let db = BTreeMap::new();
        Ok(MemEngine {
            _db: db,
            _mutex: Mutex::new(0),
        })
    }
}

impl Base for MemEngine {
    fn set_kv(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<(), Error> {
        self._db.insert(key, value);
        Ok(())
    }

    fn get_kv(&self, key: &Vec<u8>) -> Result<Vec<u8>, Error> {
        if let Some(v) = self._db.get(key) {
            Ok(v.to_vec())
        } else {
            Err(Error::NotFound {})
        }
    }

    fn next_kv(&self, key: &Vec<u8>, include: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        for (k, v) in self._db.range(key.to_vec()..) {
            if include == false && key == k {
                continue;
            }

            return Some((k.to_vec(), v.to_vec()));
        }

        None
    }

    fn get_iter(&self, key: Vec<u8>, include: bool) -> BaseIter {
        BaseIter {
            cursor: key,
            include,
            engine: self,
        }
    }
}

impl ObjectEngine for MemEngine {
    type ObjId = InstanceID;
    type Obj = Instance;
}

impl ColumnedEngine for MemEngine {
    type ColumnId = ReplicaID;
    fn make_ref_key(&self, typ: &str, col_id: Self::ColumnId) -> Vec<u8> {
        match typ {
            "max" => format!("/status/max_instance_id/{:016x}", col_id).into_bytes(),
            "exec" => format!("/status/max_exec_instance_id/{:016x}", col_id).into_bytes(),
            _ => panic!("unknown type ref"),
        }
    }
}

impl InstanceEngine for MemEngine {
    fn next_instance_id(&mut self, rid: ReplicaID) -> Result<InstanceID, Error> {
        // TODO locking
        // TODO Need to incr max-ref and add new-instance in a single tx.
        //      Or iterator may encounter an empty instance slot.
        let max = self.get_ref("max", rid);
        let mut max = match max {
            Ok(v) => v,
            Err(e) => match e {
                Error::NotFound => (rid, -1).into(),
                _ => return Err(e),
            },
        };

        max.idx += 1;
        self.set_ref("max", rid, max)?;
        Ok(max)
    }

    fn set_instance(&mut self, inst: &Instance) -> Result<(), Error> {
        // does not guarantee in a transaction
        let _ = self._mutex.lock().unwrap();

        let iid = inst.instance_id.unwrap();

        self.set_obj(iid, &inst).unwrap();

        let lowest = InstanceID::from((iid.replica_id, -1));

        if inst.executed {
            self.set_ref_if("exec", iid.replica_id, iid, lowest, |x| x < iid)?;
        }

        Ok(())
    }
    /// get an instance with instance id
    fn get_instance(&self, iid: InstanceID) -> Result<Option<Instance>, Error> {
        self.get_obj(iid)
    }

    fn get_instance_iter(&self, iid: InstanceID, include: bool) -> InstanceIter {
        InstanceIter {
            curr_inst_id: iid,
            include,
            engine: self,
        }
    }
}

impl TxEngine for MemEngine {
    fn trans_begin(&mut self) {}
    fn trans_commit(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn trans_rollback(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn get_kv_for_update(&self, _key: &Vec<u8>) -> Result<Vec<u8>, Error> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kv() {
        let cases = vec![("key1", "val1"), ("key2", "val2")];

        for case in cases {
            let mut engine = MemEngine::new().unwrap();

            let k = case.0.as_bytes().to_vec();
            let v = case.1.as_bytes().to_vec();

            let _ = engine.set_kv(k.clone(), v.clone()).unwrap();
            let act = engine.get_kv(&k).unwrap();

            assert_eq!(v, act);
        }
    }

    #[test]
    fn test_instance() {
        let mut ints = Vec::<Instance>::new();

        let mut engine = MemEngine::new().unwrap();

        for rid in 1..4 {
            for idx in 0..10 {
                let iid = InstanceID::from((rid, idx));

                let cmds = vec![Command::of(
                    OpCode::NoOp,
                    format!("k1{:}", rid * idx).as_bytes(),
                    format!("v1{:}", rid * idx).as_bytes(),
                )];

                let ballot = (rid as i32, idx as i32, 0).into();

                let deps = vec![InstanceID::from((rid + 1, idx + 1))];

                let mut inst = Instance::of(&cmds[..], ballot, &deps[..]);
                inst.instance_id = Some(iid);

                engine.set_instance(&inst).unwrap();

                let act = engine.get_obj(iid).unwrap().unwrap();

                assert_eq!(act.cmds, cmds);
                assert_eq!(act.ballot, Some(ballot));

                for (idx, inst_id) in act.initial_deps.iter().enumerate() {
                    assert_eq!(*inst_id, deps[idx]);
                }

                ints.push(inst);
            }
        }

        let cases = vec![
            ((1, 0).into(), true, &ints[..10]),
            ((1, 1).into(), true, &ints[1..10]),
            ((1, 9).into(), true, &ints[9..10]),
            ((1, 10).into(), true, &[]),
            ((3, 0).into(), true, &ints[20..3 * 10]),
            ((0, 0).into(), true, &[]), // before any present instance.
            ((6, 0).into(), true, &[]), // after all present instance.
            ((1, 0).into(), false, &ints[1..10]),
            ((1, 1).into(), false, &ints[2..10]),
            ((1, 9).into(), false, &[]),
            ((1, 10).into(), true, &[]),
            ((3, 0).into(), false, &ints[21..3 * 10]),
        ];

        for (iid, include, exp_insts) in cases {
            let mut n = 0;
            for act_inst in engine.get_instance_iter(iid, include) {
                assert_eq!(act_inst, exp_insts[n]);
                n = n + 1;
            }

            assert_eq!(exp_insts.len(), n);
        }
    }

    #[test]
    fn test_status() {
        let cases = vec![(1i64, 2), (2i64, 3)];

        for (rid, idx) in cases {
            let mut engine = MemEngine::new().unwrap();

            let iid = InstanceID::from((rid, idx));

            engine.set_ref("max", rid, iid).unwrap();
            let act = engine.get_ref("max", rid).unwrap();

            assert_eq!(act, iid);

            engine.set_ref("exec", rid, iid).unwrap();
            let act = engine.get_ref("exec", rid).unwrap();

            assert_eq!(act, iid);
        }
    }

    #[test]
    fn test_transaction() {
        let mut engine = MemEngine::new().unwrap();
        let k = "foo".as_bytes().to_vec();
        let v = "bar".as_bytes().to_vec();
        engine.trans_begin();
        engine.set_kv(k.clone(), v.clone()).unwrap();
        engine.trans_commit().unwrap();

        assert_eq!(v, engine.get_kv(&k).unwrap());
    }

    #[test]
    fn test_instance_engine() {
        {
            let mut engine = MemEngine::new().unwrap();
            test_set_instance(&mut engine);
        }

        {
            let mut engine = MemEngine::new().unwrap();
            test_get_instance(&mut engine);
        }

        {
            let mut engine = MemEngine::new().unwrap();
            test_next_instance_id(&mut engine);
        }
    }
}
