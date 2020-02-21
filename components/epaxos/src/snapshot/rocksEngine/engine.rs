use super::open;
use super::{DBColumnFamily, Error, RocksDBEngine};
use rocksdb::{CFHandle, Writable, WriteBatch};
use std::str;

struct CfKV<'a> {
    cf: &'a DBColumnFamily,
    k: &'a [u8],
    v: &'a [u8],
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

        Ok(RocksDBEngine { db: db })
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
    fn _set(&mut self, cfkv: &CfKV) -> Result<(), Error> {
        let cfh = self._make_cf_handle(cfkv.cf)?;
        Ok(self.db.put_cf(cfh, cfkv.k, cfkv.v)?)
    }

    /// Get a value from rocksdb with it's key.
    fn _get(&self, cf: &DBColumnFamily, k: &[u8]) -> Result<Vec<u8>, Error> {
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
    fn _mset(&mut self, cfkvs: &Vec<CfKV>) -> Result<(), Error> {
        let wb = WriteBatch::new();

        for cfkv in cfkvs {
            let cfh = self._make_cf_handle(cfkv.cf)?;

            wb.put_cf(cfh, cfkv.k, cfkv.v)?;
        }

        Ok(self.db.write(wb)?)
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

    eng._set(&CfKV {
        cf: &DBColumnFamily::Default,
        k: k0.as_bytes(),
        v: v0.as_bytes(),
    })
    .unwrap();

    let v_get = eng._get(&DBColumnFamily::Default, k0.as_bytes()).unwrap();
    assert_eq!(v_get, v0.as_bytes());

    let cfkvs = vec![
        CfKV {
            cf: &DBColumnFamily::Default,
            k: "key1".as_bytes(),
            v: "value1".as_bytes(),
        },
        CfKV {
            cf: &DBColumnFamily::Instance,
            k: "key2".as_bytes(),
            v: "value2".as_bytes(),
        },
        CfKV {
            cf: &DBColumnFamily::Status,
            k: "key3".as_bytes(),
            v: "value3".as_bytes(),
        },
    ];

    eng._mset(&cfkvs).unwrap();

    for cfkv in cfkvs {
        let v_get = eng._get(cfkv.cf, cfkv.k).unwrap();
        assert_eq!(v_get, cfkv.v);
    }
}
