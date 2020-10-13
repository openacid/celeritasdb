use super::open;
use crate::DBColumnFamily;
use crate::WriteEntry;
use crate::{RawKV, RocksDBEngine, StorageError};
use rocksdb::{CFHandle, SeekKey, Writable, WriteBatch};

impl RocksDBEngine {
    /// Open a Engine base on rocksdb to use snapshot.
    ///
    /// # Examples:
    /// ```
    /// use tempfile::Builder;
    /// use crate::storage::RocksDBEngine;
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
    pub fn new(path: &str) -> Result<RocksDBEngine, StorageError> {
        let db = open(path)?;

        Ok(RocksDBEngine { db })
    }

    /// make rocksdb column family handle
    fn _get_cf_handle(&self, cf: DBColumnFamily) -> Result<&CFHandle, StorageError> {
        match self.db.cf_handle(cf.into()) {
            Some(h) => Ok(h),
            None => Err(format!("got column family {:?} handle failed", cf).into()),
        }
    }

    // TODO merge it into next()
    fn _range(
        &self,
        cf: DBColumnFamily,
        key: &[u8],
        include: bool,
        reverse: bool,
    ) -> Result<Option<(Vec<u8>, Vec<u8>)>, StorageError> {
        let cf = self._get_cf_handle(cf)?;
        let mut iter = self.db.iter_cf(cf);

        let rst = iter.seek(SeekKey::from(&key[..]));
        let valid = match rst {
            Ok(valid) => valid,
            Err(estr) => {
                return Err(estr.into());
            }
        };

        if !valid {
            return Ok(None);
        }

        let k = iter.key();
        let v = iter.value();
        if include {
            return Ok(Some((k.to_vec(), v.to_vec())));
        }

        if k != key {
            return Ok(Some((k.to_vec(), v.to_vec())));
        }

        let valid = {
            if reverse {
                iter.prev()?
            } else {
                iter.next()?
            }
        };
        if !valid {
            return Ok(None);
        }

        Ok(Some((iter.key().to_vec(), iter.value().to_vec())))
    }
}

impl RawKV for RocksDBEngine {
    fn set_raw(&self, cf: DBColumnFamily, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        let cfh = self._get_cf_handle(cf)?;
        Ok(self.db.put_cf(cfh, key, value)?)
    }

    fn get_raw(&self, cf: DBColumnFamily, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let cfh = self._get_cf_handle(cf)?;
        let r = self.db.get_cf(cfh, key)?;
        Ok(r.map(|x| x.to_vec()))
    }

    fn delete_raw(&self, cf: DBColumnFamily, key: &[u8]) -> Result<(), StorageError> {
        let cfh = self._get_cf_handle(cf)?;
        Ok(self.db.delete_cf(cfh, key)?)
    }

    fn next_raw(
        &self,
        cf: DBColumnFamily,
        key: &[u8],
        forward: bool,
        include: bool,
    ) -> Result<Option<(Vec<u8>, Vec<u8>)>, StorageError> {
        self._range(cf, key, include, !forward)
    }

    fn write_batch(&self, entrys: &Vec<WriteEntry>) -> Result<(), StorageError> {
        let batch = WriteBatch::with_capacity(entrys.len());
        for en in entrys {
            match en {
                WriteEntry::Nil => {}
                WriteEntry::Set(cf, k, v) => {
                    let cfh = self._get_cf_handle(*cf)?;
                    batch.put_cf(cfh, k, v)?;
                }
                WriteEntry::Delete(cf, k) => {
                    let cfh = self._get_cf_handle(*cf)?;
                    batch.delete_cf(cfh, k)?;
                }
            }
        }

        Ok(self.db.write(&batch)?)
    }
}

#[cfg(test)]
mod tests {

    use crate::test_storage::test_base_trait;
    use crate::*;
    use tempfile::Builder;

    fn new_eng() -> RocksDBEngine {
        let tmp_root = Builder::new().tempdir().unwrap();
        let db_path = format!("{}/test", tmp_root.path().display());

        RocksDBEngine::new(&db_path).unwrap()
    }

    #[test]
    fn test_engine() {
        let eng = new_eng();
        test_base_trait(&eng);
    }
}
