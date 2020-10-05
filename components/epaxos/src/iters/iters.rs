use crate::qpaxos::*;
use crate::Storage;
use prost::Message;
use storage::*;

pub struct BaseIter {
    pub cursor: Vec<u8>,
    pub include: bool,
    pub storage: Storage,
    pub reverse: bool,
    pub cf: DBColumnFamily,
}

impl Iterator for BaseIter {
    type Item = (Vec<u8>, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        let r = if self.reverse {
            self.storage.prev_raw(self.cf, &self.cursor, self.include)
        } else {
            self.storage.next_raw(self.cf, &self.cursor, self.include)
        };

        self.include = false;
        match r {
            Some(kv) => {
                self.cursor = kv.0.clone();
                Some(kv)
            }
            None => None,
        }
    }
}
pub struct InstanceIter {
    pub curr_inst_id: InstanceId,
    pub include: bool,
    pub storage: Storage,
    pub reverse: bool,
}

impl Iterator for InstanceIter {
    type Item = Instance;

    fn next(&mut self) -> Option<Instance> {
        let k = self.curr_inst_id.to_key();
        let (key_bytes, val_bytes) = if self.reverse {
            self.storage
                .prev_raw(DBColumnFamily::Instance, &k, self.include)?
        } else {
            self.storage
                .next_raw(DBColumnFamily::Instance, &k, self.include)?
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

        let inst = match Self::Item::decode(val_bytes.as_slice()) {
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

pub trait Iter {
    fn get_instance_iter(&self, iid: InstanceId, include: bool, reverse: bool) -> InstanceIter;
    fn get_iter(&self, cur: Vec<u8>, include: bool, reverse: bool, cf: DBColumnFamily) -> BaseIter;
}

impl Iter for Storage {
    fn get_instance_iter(&self, iid: InstanceId, include: bool, reverse: bool) -> InstanceIter {
        InstanceIter {
            curr_inst_id: iid,
            include,
            storage: self.clone(),
            reverse,
        }
    }

    fn get_iter(&self, cur: Vec<u8>, include: bool, reverse: bool, cf: DBColumnFamily) -> BaseIter {
        BaseIter {
            cursor: cur,
            include,
            storage: self.clone(),
            reverse,
            cf,
        }
    }
}
