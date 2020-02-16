use std::collections::BTreeMap;
use protobuf::{parse_from_bytes, Message};

use super::InstanceIter;
use super::super::replica::ReplicaID;
use super::super::instance::{Instance, InstanceID, InstanceStatus, BallotNum};
use super::super::command::{Command, OpCode};
use super::{Error, KVEngine, InstanceEngine, StatusEngine};


pub struct MemEngine {
    pub _db: BTreeMap<Vec<u8>, Vec<u8>>,
}


impl MemEngine {
    pub fn new() -> Result<MemEngine, Error> {
        let db = BTreeMap::new();
        Ok(MemEngine{_db: db})
    }

    pub fn next_kv(&self, key:&Vec<u8>) -> Result<Vec<u8>, Error>{
        Err(Error::MemDBError)
    }
}

impl KVEngine for MemEngine {
    fn set_kv(&mut self, key: &Vec<u8>, value: &Vec<u8>) -> Result<(), Error>{
        self._db.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn get_kv(&self, key: &Vec<u8>) -> Result<Vec<u8>, Error>{
        if let Some(v) = self._db.get(key) {
            Ok(v.to_vec())
        } else {
            Err(Error::MemDBError)
        }
    }
}

impl InstanceEngine<MemEngine> for MemEngine {
    fn set_instance(&mut self, iid: &InstanceID, inst: Instance) -> Result<(), Error>{
        let key = self.instance_id_to_key(iid);
        let value: Vec<u8> = inst.write_to_bytes().unwrap();
        self.set_kv(&key, &value)
    }

    fn update_instance(&mut self,iid: &InstanceID, inst: Instance) -> Result<(), Error>{
        self.set_instance(iid, inst)
    }

    fn get_instance(&self, iid: &InstanceID) -> Result<Instance, Error>{
        let key = self.instance_id_to_key(iid);
        let val_bytes: Vec<u8> = self.get_kv(&key)?;

        match parse_from_bytes::<Instance>(&val_bytes) {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::MemDBError),
        }
    }

    fn get_instance_iter(&self, rid: ReplicaID) -> Result<InstanceIter<MemEngine>, Error>{
        let iid = InstanceID::of(rid.clone(), 0);
        Ok(InstanceIter{curr_inst_id: iid, engine: self})
    }
}

impl StatusEngine for MemEngine {
    fn get_max_instance_id(&self, rid: ReplicaID) -> Result<InstanceID, Error>{
        let key = self.max_instance_id_key(rid);
        let val_bytes: Vec<u8> = self.get_kv(&key)?;

        match parse_from_bytes::<InstanceID>(&val_bytes) {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::MemDBError),
        }
    }

    fn set_max_instance_id(&mut self, rid: ReplicaID, iid: InstanceID) -> Result<(), Error>{
        let key = self.max_instance_id_key(rid);
        let value: Vec<u8> = iid.write_to_bytes().unwrap();
        self.set_kv(&key, &value)
    }

    fn get_max_exec_instance_id(&self, rid: ReplicaID) -> Result<InstanceID, Error>{
        let key = self.max_exec_instance_id_key(rid);
        let val_bytes: Vec<u8> = self.get_kv(&key)?;

        match parse_from_bytes::<InstanceID>(&val_bytes) {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::MemDBError),
        }
    }

    fn set_max_exec_instance_id(&mut self, rid: ReplicaID, iid: InstanceID) -> Result<(), Error>{
        let key = self.max_exec_instance_id_key(rid);
        let value: Vec<u8> = iid.write_to_bytes().unwrap();
        self.set_kv(&key, &value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kv() {
        let cases = vec![
            (
                "key1",
                "val1"
            ),
            (
                "key2",
                "val2"
            ),
        ];

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
        let cases = vec![
           (
                InstanceID::of(1, 10),
                vec![
                    Command::of(OpCode::NoOp, "k1".as_bytes(), "v1".as_bytes()),
                    Command::of(OpCode::NoOp, "k2".as_bytes(), "v2".as_bytes()),
                ],
                BallotNum::of(1, 2, 3),
                vec![
                    InstanceID::of(2, 20),
                    InstanceID::of(3, 30),
                ],
            )
        ];

        for (iid, cmds, ballot, deps) in cases {
            let mut engine = MemEngine::new().unwrap();

            let inst = Instance::of(&cmds[..], &ballot, &deps[..]);

            let _ = engine.set_instance(&iid, inst).unwrap();
            let act = engine.get_instance(&iid).unwrap();

            assert_eq!(act.cmds.into_vec(), cmds);
            assert_eq!(*act.ballot.get_ref(), ballot);

            for (idx, inst_id) in act.initial_deps.iter().enumerate() {
                assert_eq!(*inst_id, deps[idx]);
            }
        }

    }

    #[test]
    fn test_status() {
        let cases = vec![
            (
                1i64, 2,
            ),
            (
                2i64, 3,
            ),
        ];

        for (rid, idx) in cases {
            let mut engine = MemEngine::new().unwrap();

            let iid = InstanceID::of(rid, idx);

            let _ = engine.set_max_instance_id(rid, iid.clone()).unwrap();
            let act = engine.get_max_instance_id(rid).unwrap();

            assert_eq!(act, iid);

            let _ = engine.set_max_exec_instance_id(rid, iid.clone()).unwrap();
            let act = engine.get_max_exec_instance_id(rid).unwrap();

            assert_eq!(act, iid);
        }
    }
}
