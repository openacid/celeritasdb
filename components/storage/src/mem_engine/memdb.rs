use std::collections::{BTreeMap, HashMap};
use std::ops::Bound::Included;
use std::ops::Bound::Unbounded;
use std::sync::Mutex;

use crate::{DBColumnFamily, MemEngine, RawKV, StorageError, WriteEntry};

impl MemEngine {
    pub fn new() -> Result<MemEngine, StorageError> {
        let db = HashMap::new();
        Ok(MemEngine {
            _db: Mutex::new(db),
        })
    }
}

impl RawKV for MemEngine {
    // TODO lock().unwrap() need to deal with poisoning
    // https://doc.rust-lang.org/std/sync/struct.Mutex.html#poisoning

    // TODO concurrent test

    fn set_raw(&self, cf: DBColumnFamily, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        let mut cfs = self._db.lock().unwrap();
        let bt = cfs.entry(cf.into()).or_insert(BTreeMap::new());
        bt.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn get_raw(&self, cf: DBColumnFamily, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let mut cfs = self._db.lock().unwrap();
        let bt = cfs.entry(cf.into()).or_insert(BTreeMap::new());
        Ok(bt.get(key).map(|x| x.clone()))
    }

    fn delete_raw(&self, cf: DBColumnFamily, key: &[u8]) -> Result<(), StorageError> {
        let mut cfs = self._db.lock().unwrap();
        let bt = cfs.entry(cf.into()).or_insert(BTreeMap::new());
        bt.remove(key);
        Ok(())
    }

    fn next_raw(
        &self,
        cf: DBColumnFamily,
        key: &[u8],
        forward: bool,
        include: bool,
    ) -> Result<Option<(Vec<u8>, Vec<u8>)>, StorageError> {
        let mut cfs = self._db.lock().unwrap();
        let bt = cfs.entry(cf.into()).or_insert(BTreeMap::new());

        let rng: Box<dyn Iterator<Item = _>> = if forward {
            Box::new(bt.range(key.to_vec()..))
        } else {
            Box::new(bt.range((Unbounded, Included(key.to_vec()))).rev())
        };

        for (k, v) in rng {
            if include == false && key == k.as_slice() {
                continue;
            }

            return Ok(Some((k.to_vec(), v.to_vec())));
        }

        Ok(None)
    }

    // TODO now just execute these commands in order
    fn write_batch(&self, entrys: &Vec<WriteEntry>) -> Result<(), StorageError> {
        for en in entrys {
            match en {
                WriteEntry::Nil => {}
                WriteEntry::Set(cf, k, v) => {
                    self.set_raw(*cf, k, v).unwrap();
                }
                WriteEntry::Delete(cf, k) => {
                    self.delete_raw(*cf, k).unwrap();
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::test_storage::test_base_trait;
    use crate::MemEngine;

    #[test]
    fn test_engine() {
        let eng = MemEngine::new().unwrap();
        test_base_trait(&eng);
    }
}
