use crate::instance::{InstanceID, Instance};
use super::MemEngine;
use super::InstanceEngine;
use protobuf::{parse_from_bytes, Message};

pub struct InstanceIter<'a, T> {
    pub curr_inst_id: InstanceID,
    pub engine: &'a T,
}

// TODO impl Iterator for MemEngine

impl <'a> Iterator for InstanceIter<'a, MemEngine> {
    type Item = Instance;

    fn next(&mut self) -> Option<Instance> {
        let k = self.engine.instance_id_to_key(&self.curr_inst_id);
        let val_bytes = self.engine.next_kv(&k).unwrap();

        match parse_from_bytes::<Instance>(&val_bytes) {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }
}
