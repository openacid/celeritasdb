use super::open;
use super::{Base, BaseIter, DBColumnFamily, Error, RocksDBEngine};
use rocksdb::{CFHandle, SeekKey, Writable, WriteBatch};
use std::str;

#[allow(dead_code)]
struct CfKV<'a> {
    cf: &'a DBColumnFamily,
    k: &'a [u8],
    v: &'a [u8],
}

impl<'a> From<(&'a DBColumnFamily, &'a [u8], &'a [u8])> for CfKV<'a> {
    fn from(cfkv: (&'a DBColumnFamily, &'a [u8], &'a [u8])) -> CfKV<'a> {
        CfKV {
            cf: cfkv.0,
            k: cfkv.1,
            v: cfkv.2,
        }
    }
}

// just for test
impl<'a> From<(&'a str, &'a str, &'a str)> for CfKV<'a> {
    fn from(cfkv: (&'a str, &'a str, &'a str)) -> CfKV<'a> {
        let cf = DBColumnFamily::from_str(cfkv.0).unwrap();

        CfKV {
            cf,
            k: cfkv.1.as_bytes(),
            v: cfkv.2.as_bytes(),
        }
    }
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
    fn _make_cf_handle(&self, cf: &DBColumnFamily) -> Result<&CFHandle, Error> {
        match self.db.cf_handle(cf.as_str()) {
            Some(h) => Ok(h),
            None => Err(Error::DBError {
                msg: format!("got column family {} handle failed", cf.as_str()),
            }),
        }
    }

    /// Set a key-value pair to rocksdb.
    fn set(&self, cfkv: &CfKV) -> Result<(), Error> {
        let cfh = self._make_cf_handle(cfkv.cf)?;
        Ok(self.db.put_cf(cfh, cfkv.k, cfkv.v)?)
    }

    /// Get a value from rocksdb with it's key.
    fn get(&self, cf: &DBColumnFamily, k: &[u8]) -> Result<Vec<u8>, Error> {
        let cfh = self._make_cf_handle(cf)?;

        match self.db.get_cf(cfh, k) {
            Ok(option_val) => match option_val {
                Some(val) => {
                    return Ok(val.to_vec());
                }
                None => {
                    let k_str = match str::from_utf8(k) {
                        Ok(s) => s,
                        Err(err) => {
                            return Err(Error::DBError {
                                msg: format!("{} while converting utf8 to str", err),
                            });
                        }
                    };
                    return Err(Error::DBError {
                        msg: format!("key not found: {}", k_str),
                    });
                }
            },
            Err(err) => {
                let k_str = match str::from_utf8(k) {
                    Ok(s) => s,
                    Err(err) => {
                        return Err(Error::DBError {
                            msg: format!("{} while converting utf8 to str", err),
                        });
                    }
                };
                return Err(Error::DBError {
                    msg: format!("{} while loading key {}", err, k_str),
                });
            }
        };
    }

    /// Set multi keys-values to rocksdb atomically.
    fn _mset(&self, cfkvs: &Vec<CfKV>) -> Result<(), Error> {
        let wb = WriteBatch::new();

        for cfkv in cfkvs {
            let cfh = self._make_cf_handle(cfkv.cf)?;

            wb.put_cf(cfh, cfkv.k, cfkv.v)?;
        }

        Ok(self.db.write(wb)?)
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
    fn set_kv(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), Error> {
        self.set(&CfKV {
            cf: &DBColumnFamily::Default,
            k: &key,
            v: &value,
        })
    }

    fn get_kv(&self, key: &Vec<u8>) -> Result<Vec<u8>, Error> {
        self.get(&DBColumnFamily::Default, key)
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
}

#[test]
fn test_rocks_engine() {
    use tempfile::Builder;

    let tmp_root = Builder::new().tempdir().unwrap();
    let db_path = format!("{}/test", tmp_root.path().display());

    let mut eng = RocksDBEngine::new(&db_path).unwrap();

    let k0 = "key0";
    let v0 = "value0";

    eng.set(&("default", k0, v0).into()).unwrap();

    let v_get = eng.get(&DBColumnFamily::Default, k0.as_bytes()).unwrap();
    assert_eq!(v_get, v0.as_bytes());

    let cfkvs = vec![
        ("default", "key1", "value1").into(),
        ("instance", "key2", "value2").into(),
        ("status", "key3", "value3").into(),
    ];

    eng._mset(&cfkvs).unwrap();

    for cfkv in cfkvs {
        let v_get = eng.get(cfkv.cf, cfkv.k).unwrap();
        assert_eq!(v_get, cfkv.v);
    }
}
