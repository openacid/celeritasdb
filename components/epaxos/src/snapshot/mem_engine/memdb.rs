use std::collections::{BTreeMap, HashMap};
use std::ops::Bound::Included;
use std::ops::Bound::Unbounded;
use std::sync::Mutex;

use crate::snapshot::*;

impl MemEngine {
    pub fn new() -> Result<MemEngine, Error> {
        let db = HashMap::new();
        Ok(MemEngine {
            _db: Mutex::new(db),
        })
    }
}

impl Base for MemEngine {
    // TODO lock().unwrap() need to deal with poisoning
    // https://doc.rust-lang.org/std/sync/struct.Mutex.html#poisoning

    fn set(&self, cf: DBColumnFamily, key: &Vec<u8>, value: &Vec<u8>) -> Result<(), Error> {
        let mut bt = self._db.lock().unwrap();
        let bt = bt.entry(cf.into()).or_insert(BTreeMap::new());
        bt.insert(key.clone(), value.clone());
        Ok(())
    }

    fn get(&self, cf: DBColumnFamily, key: &Vec<u8>) -> Result<Option<Vec<u8>>, Error> {
        let mut bt = self._db.lock().unwrap();
        let bt = bt.entry(cf.into()).or_insert(BTreeMap::new());
        Ok(bt.get(key).map(|x| x.clone()))
    }

    fn delete(&self, cf: DBColumnFamily, key: &Vec<u8>) -> Result<(), Error> {
        let mut bt = self._db.lock().unwrap();
        let bt = bt.entry(cf.into()).or_insert(BTreeMap::new());
        bt.remove(key);
        Ok(())
    }

    fn next(&self, cf: DBColumnFamily, key: &Vec<u8>, include: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        let mut bt = self._db.lock().unwrap();
        let bt = bt.entry(cf.into()).or_insert(BTreeMap::new());

        for (k, v) in bt.range(key.to_vec()..) {
            if include == false && key == k {
                continue;
            }

            return Some((k.to_vec(), v.to_vec()));
        }

        None
    }

    fn prev(&self, cf: DBColumnFamily, key: &Vec<u8>, include: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        let mut bt = self._db.lock().unwrap();
        let bt = bt.entry(cf.into()).or_insert(BTreeMap::new());

        for (k, v) in bt.range((Unbounded, Included(key.to_vec()))).rev() {
            if include == false && key == k {
                continue;
            }

            return Some((k.to_vec(), v.to_vec()));
        }

        None
    }

    // TODO now just execute these commands in order
    fn write_batch(&self, cmds: &Vec<Command>) -> Result<(), Error> {
        for cmd in cmds {
            match cmd {
                Command::Get(_, _) => panic!("write batch don't support Get command"),
                Command::Set(cf, k, v) => {
                    self.set(*cf, k, v).unwrap();
                }
                Command::Delete(cf, k) => {
                    self.delete(*cf, k).unwrap();
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_base() {
        let engine = Arc::new(MemEngine::new().unwrap());
        test_base_trait(engine);
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
            let engine = Arc::new(MemEngine::new().unwrap());
            test_get_instance_iter(engine);
        }
    }
}
