use std::collections::BTreeMap;
use std::ops::Bound::Included;
use std::ops::Bound::Unbounded;
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

    fn prev_kv(&self, key: &Vec<u8>, include: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        for (k, v) in self._db.range((Unbounded, Included(key.to_vec()))).rev() {
            if include == false && key == k {
                continue;
            }

            return Some((k.to_vec(), v.to_vec()));
        }

        None
    }

    fn get_iter(&self, key: Vec<u8>, include: bool, reverse: bool) -> BaseIter {
        BaseIter {
            cursor: key,
            include,
            engine: self,
            reverse,
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

    fn get_instance_iter(&self, iid: InstanceID, include: bool, reverse: bool) -> InstanceIter {
        InstanceIter {
            curr_inst_id: iid,
            include,
            engine: self,
            reverse,
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
    fn test_base() {
        let mut engine = MemEngine::new().unwrap();
        test_base_trait(&mut engine);
    }

    #[test]
    fn test_columned() {
        let mut engine = MemEngine::new().unwrap();
        test_columned_trait(&mut engine);
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
    fn test_instance() {
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

        {
            let mut engine = MemEngine::new().unwrap();
            test_get_instance_iter(&mut engine);
        }
    }
}
