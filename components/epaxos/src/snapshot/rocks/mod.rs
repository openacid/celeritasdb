use super::{DBColumnFamily, Engine, Error};

mod dbutil;
use dbutil::*;

impl Engine {
    /// Open a new Engine to use snapshot.
    ///
    /// # Examples:
    /// ```
    /// use tempfile::Builder;
    /// use crate::epaxos::snapshot::{DBColumnFamily, Engine};
    ///
    /// let tmp_root = Builder::new().tempdir().unwrap();
    /// let db_path = format!("{}/test", tmp_root.path().display());
    ///
    /// let my_eng;
    /// match Engine::new(&db_path, DBColumnFamily::Default) {
    ///     Ok(eng) => my_eng = eng,
    ///     Err(err) => println!("failed to get snapshot engine, failed: {}", err),
    /// };
    /// ```
    pub fn new(path: &str, cf: DBColumnFamily) -> Result<Engine, Error> {
        let db = open(path)?;

        Ok(Engine {
            _db: db,
            _cf: cf.as_str(),
        })
    }
}
