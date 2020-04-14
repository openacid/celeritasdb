use super::open;
use super::{Base, Error, RocksDBEngine};
use crate::snapshot::DBColumnFamily;
use crate::snapshot::WriteEntry;
use rocksdb::{CFHandle, SeekKey, Writable, WriteBatch};
use std::str;

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

    fn _range(
        &self,
        cf: DBColumnFamily,
        key: &Vec<u8>,
        include: bool,
        reverse: bool,
    ) -> Option<(Vec<u8>, Vec<u8>)> {
        let cf = self._make_cf_handle(cf).ok()?;
        let mut iter = self.db.iter_cf(cf);

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
    fn set(&self, cf: DBColumnFamily, key: &Vec<u8>, value: &Vec<u8>) -> Result<(), Error> {
        let cfh = self._make_cf_handle(cf)?;
        Ok(self.db.put_cf(cfh, key, value)?)
    }

    fn get(&self, cf: DBColumnFamily, key: &Vec<u8>) -> Result<Option<Vec<u8>>, Error> {
        let cfh = self._make_cf_handle(cf)?;
        let r = self.db.get_cf(cfh, key)?;
        Ok(r.map(|x| x.to_vec()))
    }

    fn delete(&self, cf: DBColumnFamily, key: &Vec<u8>) -> Result<(), Error> {
        let cfh = self._make_cf_handle(cf)?;
        Ok(self.db.delete_cf(cfh, key)?)
    }

    fn next(&self, cf: DBColumnFamily, key: &Vec<u8>, include: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        self._range(cf, key, include, false)
    }

    fn prev(&self, cf: DBColumnFamily, key: &Vec<u8>, include: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        self._range(cf, key, include, true)
    }

    fn write_batch(&self, entrys: &Vec<WriteEntry>) -> Result<(), Error> {
        let batch = WriteBatch::with_capacity(entrys.len());
        for en in entrys {
            match en {
                WriteEntry::Nil => {}
                WriteEntry::Set(cf, k, v) => {
                    let cfh = self._make_cf_handle(*cf)?;
                    batch.put_cf(cfh, k, v)?;
                }
                WriteEntry::Delete(cf, k) => {
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
    use crate::snapshot::*;
    use tempfile::Builder;

    let tmp_root = Builder::new().tempdir().unwrap();
    let db_path = format!("{}/test", tmp_root.path().display());

    let eng = RocksDBEngine::new(&db_path).unwrap();

    let k0 = "key0";
    let v0 = "value0";

    eng.set_kv(&k0.as_bytes().to_vec(), &v0.as_bytes().to_vec())
        .unwrap();

    let v_get = eng
        .get(DBColumnFamily::Default, &k0.as_bytes().to_vec())
        .unwrap();
    assert_eq!(v_get.unwrap(), v0.as_bytes());

    let k1 = "k1".as_bytes().to_vec();
    let k2 = "k2".as_bytes().to_vec();
    let v1 = "v1".as_bytes().to_vec();
    let v2 = "v2".as_bytes().to_vec();

    let cmds = vec![
        WriteEntry::Set(DBColumnFamily::Default, k1.clone(), v1.clone()),
        WriteEntry::Set(DBColumnFamily::Default, k2.clone(), v2.clone()),
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
        WriteEntry::Set(DBColumnFamily::Default, k1.clone(), v1.clone()),
        WriteEntry::Delete(DBColumnFamily::Default, k1.clone()),
    ];

    eng.write_batch(&cmds).unwrap();
    assert_eq!(None, eng.get_kv(&"k1".as_bytes().to_vec()).unwrap());
}
