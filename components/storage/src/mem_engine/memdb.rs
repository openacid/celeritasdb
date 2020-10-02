use std::collections::{BTreeMap, HashMap};
use std::ops::Bound::Included;
use std::ops::Bound::Unbounded;
use std::sync::Mutex;

use crate::{Base, DBColumnFamily, MemEngine, StorageError, WriteEntry};

impl MemEngine {
    pub fn new() -> Result<MemEngine, StorageError> {
        let db = HashMap::new();
        Ok(MemEngine {
            _db: Mutex::new(db),
        })
    }
}

impl Base for MemEngine {
    // TODO lock().unwrap() need to deal with poisoning
    // https://doc.rust-lang.org/std/sync/struct.Mutex.html#poisoning

    // TODO concurrent test

    fn set(&self, cf: DBColumnFamily, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        let mut cfs = self._db.lock().unwrap();
        let bt = cfs.entry(cf.into()).or_insert(BTreeMap::new());
        bt.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn get(&self, cf: DBColumnFamily, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let mut cfs = self._db.lock().unwrap();
        let bt = cfs.entry(cf.into()).or_insert(BTreeMap::new());
        Ok(bt.get(key).map(|x| x.clone()))
    }

    fn delete(&self, cf: DBColumnFamily, key: &[u8]) -> Result<(), StorageError> {
        let mut cfs = self._db.lock().unwrap();
        let bt = cfs.entry(cf.into()).or_insert(BTreeMap::new());
        bt.remove(key);
        Ok(())
    }

    fn next(&self, cf: DBColumnFamily, key: &[u8], include: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        let mut cfs = self._db.lock().unwrap();
        let bt = cfs.entry(cf.into()).or_insert(BTreeMap::new());

        for (k, v) in bt.range(key.to_vec()..) {
            if include == false && key == k.as_slice() {
                continue;
            }

            return Some((k.to_vec(), v.to_vec()));
        }

        None
    }

    fn prev(&self, cf: DBColumnFamily, key: &[u8], include: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        let mut cfs = self._db.lock().unwrap();
        let bt = cfs.entry(cf.into()).or_insert(BTreeMap::new());

        for (k, v) in bt.range((Unbounded, Included(key.to_vec()))).rev() {
            if include == false && key == k.as_slice() {
                continue;
            }

            return Some((k.to_vec(), v.to_vec()));
        }

        None
    }

    // TODO now just execute these commands in order
    fn write_batch(&self, entrys: &Vec<WriteEntry>) -> Result<(), StorageError> {
        for en in entrys {
            match en {
                WriteEntry::Nil => {}
                WriteEntry::Set(cf, k, v) => {
                    self.set(*cf, k, v).unwrap();
                }
                WriteEntry::Delete(cf, k) => {
                    self.delete(*cf, k).unwrap();
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_engine::*;
    use crate::*;

    #[test]
    fn test_engine() {
        {
            let eng = MemEngine::new().unwrap();
            test_base_trait(&eng);
        }

        {
            let eng = MemEngine::new().unwrap();
            test_record_trait(&eng);
        }

        {
            let eng = MemEngine::new().unwrap();
            test_columned_trait(&eng);
        }

        {
            let eng = MemEngine::new().unwrap();
            test_instance_trait(&eng);
        }
    }
}
