use std::collections::BTreeMap;
use std::ops::Bound::Included;
use std::ops::Bound::Unbounded;
use std::sync::Mutex;

use super::super::*;
use super::MemEngine;

use crate::qpaxos::*;
use crate::snapshot::Command;

impl MemEngine {
    pub fn new() -> Result<MemEngine, Error> {
        let db = BTreeMap::new();
        Ok(MemEngine {
            _db: Mutex::new(db),
        })
    }
}

impl Base for MemEngine {
    // TODO lock().unwrap() need to deal with poisoning
    // https://doc.rust-lang.org/std/sync/struct.Mutex.html#poisoning

    fn set_kv(&self, key: &Vec<u8>, value: &Vec<u8>) -> Result<(), Error> {
        let mut bt = self._db.lock().unwrap();
        bt.insert(key.clone(), value.clone());
        Ok(())
    }

    fn get_kv(&self, key: &Vec<u8>) -> Result<Option<Vec<u8>>, Error> {
        let bt = self._db.lock().unwrap();
        Ok(bt.get(key).map(|x| x.clone()))
    }

    fn delete_kv(&self, key: &Vec<u8>) -> Result<(), Error> {
        let mut bt = self._db.lock().unwrap();
        bt.remove(key);
        Ok(())
    }

    fn next_kv(&self, key: &Vec<u8>, include: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        let bt = self._db.lock().unwrap();

        for (k, v) in bt.range(key.to_vec()..) {
            if include == false && key == k {
                continue;
            }

            return Some((k.to_vec(), v.to_vec()));
        }

        None
    }

    fn prev_kv(&self, key: &Vec<u8>, include: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        let bt = self._db.lock().unwrap();

        for (k, v) in bt.range((Unbounded, Included(key.to_vec()))).rev() {
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

    // TODO now just execute these commands in order
    fn write_batch(&self, cmds: &Vec<Command>) -> Result<(), Error> {
        for cmd in cmds {
            match cmd {
                Command::Get(_, _) => panic!("write batch don't support Get command"),
                Command::Set(_, k, v) => {
                    self.set_kv(k, v).unwrap();
                }
                Command::Delete(_, k) => {
                    self.delete_kv(k).unwrap();
                }
            }
        }

        Ok(())
    }
}

impl ObjectEngine for MemEngine {
    type ObjId = InstanceId;
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
    fn next_instance_id(&self, rid: ReplicaID) -> Result<InstanceId, Error> {
        // TODO locking
        // TODO Need to incr max-ref and add new-instance in a single tx.
        //      Or iterator may encounter an empty instance slot.
        let max = self.get_ref("max", rid)?;
        let mut max = max.unwrap_or((rid, -1).into());
        max.idx += 1;
        self.set_ref("max", rid, max)?;
        Ok(max)
    }

    fn set_instance(&self, inst: &Instance) -> Result<(), Error> {
        // TODO does not guarantee in a transaction

        let iid = inst.instance_id.unwrap();

        self.set_obj(iid, &inst).unwrap();

        let lowest = InstanceId::from((iid.replica_id, -1));

        if inst.executed {
            self.set_ref_if("exec", iid.replica_id, iid, lowest, |x| x < iid)?;
        }

        Ok(())
    }
    /// get an instance with instance id
    fn get_instance(&self, iid: InstanceId) -> Result<Option<Instance>, Error> {
        self.get_obj(iid)
    }

    fn get_instance_iter(&self, iid: InstanceId, include: bool, reverse: bool) -> InstanceIter {
        InstanceIter {
            curr_inst_id: iid,
            include,
            engine: self,
            reverse,
        }
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
