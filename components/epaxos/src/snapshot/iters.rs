use super::mem_engine::*;
use crate::qpaxos::{Instance, InstanceID};
use prost::Message;

use crate::tokey::ToKey;

pub struct InstanceIter<'a, T> {
    pub curr_inst_id: InstanceID,
    pub include: bool,
    pub engine: &'a T,
}

impl<'a> Iterator for InstanceIter<'a, MemEngine> {
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
