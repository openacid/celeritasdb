use rocksdb::{Writable, WriteBatch};

use super::{Error, Engine, DBCF, DBPath};

mod dbutil;
use dbutil::*;

impl Engine<'_> {
    pub fn new<'a>(path: DBPath, cf: DBCF) -> Engine<'a> {
        let db = open(path).unwrap();

        Engine{
            db: db,
            cf: cf.as_str(),
        }
    }

    // Set a key-value pair to rocksdb.
    pub fn set(&self, k: &[u8], v: &[u8]) -> Result<(), Error> {
        let cfh = self.db.cf_handle(self.cf).unwrap();
        Ok(self.db.put_cf(cfh, k, v)?)
    }

    // Set multi keys-values to rocksdb.
    pub fn mset(&self, keys: &Vec<&str>, values: &Vec<&str>) -> Result<(), Error> {
        let wb = WriteBatch::new();
        let cfh = self.db.cf_handle(self.cf).unwrap();

        let len = keys.len();

        for i in 0..len {
            let k = keys[i];
            let v = values[i];

            wb.put_cf(cfh, k.as_bytes(), v.as_bytes())?;
        }

        Ok(self.db.write(&wb)?)
    }


    // Get a value in rocksdb with it's key.
    pub fn get(&self, k: &str) -> Result<String, Error> {
        let cfh = self.db.cf_handle(self.cf).unwrap();

        match self.db.get_cf(cfh, k.as_bytes()){
            Ok(option_val) => match option_val{
                Some(val) => {return Ok(val.to_utf8().unwrap().to_string());},
                None => {return Err(Error::RocksDBError{msg: format!("value for key {} not found!", k)});},
            },
            Err(err) => {return Err(Error::RocksDBError{msg: format!("load key {} got error: {}", k, err)});},
        };
    }
}
