use crate::qpaxos::*;
use crate::Storage;
use storage::*;

// TODO rename to RawKVIter
pub struct RawKVIter {
    pub cursor: Vec<u8>,
    pub include: bool,
    pub storage: Storage,
    pub reverse: bool,
    pub cf: DBColumnFamily,
}

impl Iterator for RawKVIter {
    type Item = Result<(Vec<u8>, Vec<u8>), StorageError>;

    fn next(&mut self) -> Option<Self::Item> {
        let r = self
            .storage
            .next_raw(self.cf, &self.cursor, !self.reverse, self.include);

        let r = match r {
            Ok(v) => v,
            Err(e) => {
                return Some(Err(e));
            }
        };

        self.include = false;
        match r {
            Some(kv) => {
                self.cursor = kv.0.clone();
                Some(Ok(kv))
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
        let nxt = self.storage.next::<InstanceId, Instance>(
            DBColumnFamily::Instance,
            &self.curr_inst_id,
            !self.reverse,
            self.include,
        );

        let nxt = match nxt {
            Err(e) => {
                // TODO handle StorageError
                // TODO handle data damaging.
                // TODO add test of data corruption
                panic!("storage error: {:?}", e);
            }
            Ok(nxt) => nxt,
        };

        let (iid, inst) = match nxt {
            None => {
                return None;
            }
            Some((a, b)) => (a, b),
        };

        if iid.replica_id != self.curr_inst_id.replica_id {
            // out of bound, done
            return None;
        }

        self.curr_inst_id = iid;
        self.include = false;

        Some(inst)
    }
}

pub trait Iter {
    fn get_instance_iter(&self, iid: InstanceId, include: bool, reverse: bool) -> InstanceIter;
    fn get_iter(&self, cur: Vec<u8>, include: bool, reverse: bool, cf: DBColumnFamily)
        -> RawKVIter;
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

    fn get_iter(
        &self,
        cur: Vec<u8>,
        include: bool,
        reverse: bool,
        cf: DBColumnFamily,
    ) -> RawKVIter {
        RawKVIter {
            cursor: cur,
            include,
            storage: self.clone(),
            reverse,
            cf,
        }
    }
}
