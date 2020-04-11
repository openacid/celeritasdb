use super::open;
use super::{Base, BaseIter, DBColumnFamily, Error, RocksDBEngine};
use rocksdb::{CFHandle, SeekKey, Writable, WriteBatch};
use std::str;

pub enum Command<'a> {
    Get(DBColumnFamily, &'a Vec<u8>),
    Set(DBColumnFamily, &'a Vec<u8>, &'a Vec<u8>),
    Delete(DBColumnFamily, &'a Vec<u8>),
}

impl RocksDBEngine {
    /// Open a Engine base on rocksdb to use snapshot.
    ///
    /// # Examples:
    /// ```
    /// use tempfile::Builder;
    /// use crate::epaxos::snapshot::RocksDBEngine;
    ///
    /// let tmp_root = Builder::new().tempdir().unwrap();
    /// let db_path = format!("{}/test", tmp_root.path().display());
    ///
    /// let my_eng;
    /// match RocksDBEngine::new(&db_path) {
    ///     Ok(eng) => my_eng = eng,
    ///     Err(err) => println!("failed to get rocksdb engine, failed: {}", err),
    /// };
    /// ```
    pub fn new(path: &str) -> Result<RocksDBEngine, Error> {
        let db = open(path)?;

        Ok(RocksDBEngine { db })
    }

    /// make rocksdb column family handle
    fn _make_cf_handle(&self, cf: DBColumnFamily) -> Result<&CFHandle, Error> {
        match self.db.cf_handle(cf.into()) {
            Some(h) => Ok(h),
            None => Err(format!("got column family {:?} handle failed", cf).into()),
        }
    }

    /// Set a key-value pair to rocksdb.
    fn set(&self, cf: DBColumnFamily, k: &[u8], v: &[u8]) -> Result<(), Error> {
        let cfh = self._make_cf_handle(cf)?;
        Ok(self.db.put_cf(cfh, k, v)?)
    }

    /// Get a value from rocksdb with it's key.
    /// if the key not found, return a None
    fn get(&self, cf: DBColumnFamily, k: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        let cfh = self._make_cf_handle(cf)?;
        let r = self.db.get_cf(cfh, k)?;
        Ok(r.map(|x| x.to_vec()))
    }

    /// Delete a key in rocksdb.
    fn delete(&self, cf: DBColumnFamily, k: &[u8]) -> Result<(), Error> {
        let cfh = self._make_cf_handle(cf)?;
        Ok(self.db.delete_cf(cfh, k)?)
    }

    fn _range(&self, key: &Vec<u8>, include: bool, reverse: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        let mut iter = self.db.iter();

        iter.seek(SeekKey::from(&key[..]));
        if !iter.valid() {
            // TODO may be a rocksdb panic here
            return None;
        }

        match iter.kv() {
            Some(kv) => {
                if include {
                    return Some(kv);
                };

                if &kv.0 != key {
                    return Some(kv);
                };
            }
            None => return None,
        }

        if reverse {
            iter.prev();
        } else {
            iter.next();
        }
        if !iter.valid() {
            // TODO may be a rocksdb panic here
            return None;
        }

        return iter.kv();
    }
}

impl Base for RocksDBEngine {
    fn set_kv(&self, key: &Vec<u8>, value: &Vec<u8>) -> Result<(), Error> {
        self.set(DBColumnFamily::Default, &key, &value)
    }

    fn get_kv(&self, key: &Vec<u8>) -> Result<Option<Vec<u8>>, Error> {
        self.get(DBColumnFamily::Default, key)
    }

    fn delete_kv(&self, key: &Vec<u8>) -> Result<(), Error> {
        self.delete(DBColumnFamily::Default, key)
    }

    fn next_kv(&self, key: &Vec<u8>, include: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        self._range(key, include, false)
    }

    fn prev_kv(&self, key: &Vec<u8>, include: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        self._range(key, include, true)
    }

    fn get_iter(&self, key: Vec<u8>, include: bool, reverse: bool) -> BaseIter {
        return BaseIter {
            cursor: key,
            include,
            engine: self,
            reverse,
        };
    }

    fn write_batch(&self, cmds: &Vec<Command>) -> Result<(), Error> {
        let batch = WriteBatch::with_capacity(cmds.len());
        for cmd in cmds {
            match cmd {
                Command::Get(_, _) => panic!("write batch don't support Get command"),
                Command::Set(cf, k, v) => {
                    let cfh = self._make_cf_handle(*cf)?;
                    batch.put_cf(cfh, k, v)?;
                }
                Command::Delete(cf, k) => {
                    let cfh = self._make_cf_handle(*cf)?;
                    batch.delete_cf(cfh, k)?;
                }
            }
        }

        Ok(self.db.write(batch)?)
    }
}

#[test]
fn test_rocks_engine() {
    use tempfile::Builder;

    let tmp_root = Builder::new().tempdir().unwrap();
    let db_path = format!("{}/test", tmp_root.path().display());

    let eng = RocksDBEngine::new(&db_path).unwrap();

    let k0 = "key0";
    let v0 = "value0";

    eng.set_kv(&k0.as_bytes().to_vec(), &v0.as_bytes().to_vec())
        .unwrap();

    let v_get = eng.get(DBColumnFamily::Default, k0.as_bytes()).unwrap();
    assert_eq!(v_get.unwrap(), v0.as_bytes());

    let k1 = "k1".as_bytes().to_vec();
    let k2 = "k2".as_bytes().to_vec();
    let v1 = "v1".as_bytes().to_vec();
    let v2 = "v2".as_bytes().to_vec();

    let cmds = vec![
        Command::Set(DBColumnFamily::Default, &k1, &v1),
        Command::Set(DBColumnFamily::Default, &k2, &v2),
    ];

    eng.write_batch(&cmds).unwrap();
    assert_eq!(
        "v1".as_bytes().to_vec(),
        eng.get_kv(&"k1".as_bytes().to_vec()).unwrap().unwrap()
    );
    assert_eq!(
        "v2".as_bytes().to_vec(),
        eng.get_kv(&"k2".as_bytes().to_vec()).unwrap().unwrap()
    );

    let cmds = vec![
        Command::Set(DBColumnFamily::Default, &k1, &v1),
        Command::Delete(DBColumnFamily::Default, &k1),
    ];

    eng.write_batch(&cmds).unwrap();
    assert_eq!(None, eng.get_kv(&"k1".as_bytes().to_vec()).unwrap());
}
