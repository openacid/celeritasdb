use std::collections::BTreeMap;
use std::sync::Mutex;

use super::MemEngine;
use prost::Message;

use super::super::{
    Error, InstanceEngine, InstanceIter, KVEngine, StatusEngine, TransactionEngine,
};
use crate::qpaxos::{BallotNum, Instance, InstanceID};
use crate::qpaxos::{Command, OpCode};
use crate::replica::ReplicaID;

use super::super::super::tokey::ToKey;

impl MemEngine {
    pub fn new() -> Result<MemEngine, Error> {
        let db = BTreeMap::new();
        Ok(MemEngine {
            _db: db,
            _mutex: Mutex::new(0),
        })
    }

    pub fn next_kv(&self, key: &Vec<u8>, include: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        for (k, v) in self._db.range(key.to_vec()..) {
            if include == false && key == k {
                continue;
            }

            return Some((k.to_vec(), v.to_vec()));
        }

        None
    }
}

impl KVEngine for MemEngine {
    fn set_kv(&mut self, key: &Vec<u8>, value: &Vec<u8>) -> Result<(), Error> {
        self._db.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn get_kv(&self, key: &Vec<u8>) -> Result<Vec<u8>, Error> {
        if let Some(v) = self._db.get(key) {
            Ok(v.to_vec())
        } else {
            Err(Error::NotFound {})
        }
    }
}

impl InstanceEngine<MemEngine> for MemEngine {
    fn set_instance(&mut self, iid: &InstanceID, inst: Instance) -> Result<(), Error> {
        // does not guarantee in a transaction
        let _ = self._mutex.lock().unwrap();

        let key = iid.to_key();
        let mut value = vec![];
        inst.encode(&mut value).unwrap();
        let _ = self.set_kv(&key, &value)?;

        let max_iid = self.get_max_instance_id(iid.replica_id);
        let max_iid = match max_iid {
            Ok(v) => v,
            Err(err) => {
                if err == Error::NotFound {
                    InstanceID::of(iid.replica_id, -1)
                } else {
                    return Err(err);
                }
            }
        };

        if &max_iid < iid {
            let key = self.max_instance_id_key(iid.replica_id);
            let _ = self.set_instance_id(&key, iid.clone())?;
        }

        if inst.executed && &max_iid < iid {
            let key = self.max_exec_instance_id_key(iid.replica_id);
            let _ = self.set_instance_id(&key, iid.clone())?;
        }

        Ok(())
    }

    fn update_instance(&mut self, iid: &InstanceID, inst: Instance) -> Result<(), Error> {
        self.set_instance(iid, inst)
    }

    fn get_instance(&self, iid: &InstanceID) -> Result<Instance, Error> {
        let key = iid.to_key();
        let val_bytes: Vec<u8> = self.get_kv(&key)?;

        match Instance::decode(val_bytes.as_slice()) {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::DBError {
                msg: "parse instance error".to_string(),
            }),
        }
    }

    fn get_instance_iter(&self, rid: ReplicaID) -> Result<InstanceIter<MemEngine>, Error> {
        let iid = InstanceID::of(rid.clone(), 0);
        Ok(InstanceIter {
            curr_inst_id: iid,
            include: true,
            engine: self,
        })
    }
}

impl StatusEngine for MemEngine {
    fn get_max_instance_id(&self, rid: ReplicaID) -> Result<InstanceID, Error> {
        let key = self.max_instance_id_key(rid);
        let val_bytes: Vec<u8> = self.get_kv(&key)?;

        match InstanceID::decode(val_bytes.as_slice()) {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::DBError {
                msg: "parse instance id error".to_string(),
            }),
        }
    }

    fn get_max_exec_instance_id(&self, rid: ReplicaID) -> Result<InstanceID, Error> {
        let key = self.max_exec_instance_id_key(rid);
        let val_bytes: Vec<u8> = self.get_kv(&key)?;

        match InstanceID::decode(val_bytes.as_slice()) {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::DBError {
                msg: "parse instance id error".to_string(),
            }),
        }
    }
}

impl TransactionEngine<MemEngine> for MemEngine {
    fn trans_begin(&mut self) {}
    fn trans_commit(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn trans_rollback(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn get_kv_for_update(&self, key: &Vec<u8>) -> Result<Vec<u8>, Error> {
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

            let _ = engine.set_kv(&k, &v).unwrap();
            let act = engine.get_kv(&k).unwrap();

            assert_eq!(v, act);
        }
    }

    #[test]
    fn test_instance() {
        let mut ints = Vec::<Instance>::new();

        let mut engine = MemEngine::new().unwrap();

        for rid in 0..3 {
            for idx in 0..10 {
                let iid = InstanceID::of(rid, idx);

                let cmds = vec![Command::of(
                    OpCode::NoOp,
                    format!("k1{:}", rid * idx).as_bytes(),
                    format!("v1{:}", rid * idx).as_bytes(),
                )];

                let ballot = BallotNum::of(rid as i32, idx as i32, 0);

                let deps = vec![InstanceID::of(rid + 1, idx + 1)];

                let inst = Instance::of(&cmds[..], &ballot, &deps[..]);

                let _ = engine.set_instance(&iid, inst.clone()).unwrap();

                let act = engine.get_max_instance_id(rid).unwrap();
                assert_eq!(act, iid);

                let act = engine.get_instance(&iid).unwrap();

                assert_eq!(act.cmds, cmds);
                assert_eq!(act.ballot, Some(ballot));

                for (idx, inst_id) in act.initial_deps.iter().enumerate() {
                    assert_eq!(*inst_id, deps[idx]);
                }

                ints.push(inst);
            }
        }

        let cases = vec![
            (0, &ints[..10]),
            (2, &ints[20..3 * 10]),
            (4, &ints[ints.len()..]),
        ];

        for (rid, exp_insts) in cases {
            let mut n = 0;
            for act_inst in engine.get_instance_iter(rid).unwrap() {
                assert_eq!(act_inst.cmds, exp_insts[n].cmds);
                assert_eq!(act_inst.ballot, exp_insts[n].ballot);

                assert_eq!(act_inst.instance_id, exp_insts[n].instance_id);

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

            let iid = InstanceID::of(rid, idx);

            let key = engine.max_instance_id_key(rid);
            let _ = engine.set_instance_id(&key, iid.clone()).unwrap();
            let act = engine.get_max_instance_id(rid).unwrap();

            assert_eq!(act, iid);

            let key = engine.max_exec_instance_id_key(rid);
            let _ = engine.set_instance_id(&key, iid.clone()).unwrap();
            let act = engine.get_max_exec_instance_id(rid).unwrap();

            assert_eq!(act, iid);
        }
    }

    #[test]
    fn test_transaction() {
        let mut engine = MemEngine::new().unwrap();
        let k = "foo".as_bytes().to_vec();
        let v = "bar".as_bytes().to_vec();
        engine.trans_begin();
        engine.set_kv(&k, &v);
        engine.trans_commit();

        assert_eq!(v, engine.get_kv(&k).unwrap());
    }
}
