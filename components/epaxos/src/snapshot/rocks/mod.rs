use super::{DBColumnFamily, DBPath, Engine, Error};

mod dbutil;
use dbutil::*;

impl Engine {
    /// Open a new Engine to use snapshot.
    ///
    /// # Examples:
    /// ```
    /// use crate::epaxos::snapshot::{DBColumnFamily, DBPath, Engine};
    ///
    /// let my_eng;
    /// match Engine::new(DBPath::Test, DBColumnFamily::Default) {
    ///     Ok(eng) => my_eng = eng,
    ///     Err(err) => println!("failed to get snapshot engine, failed: {}", err),
    /// };
    /// ```
    pub fn new(path: DBPath, cf: DBColumnFamily) -> Result<Engine, Error> {
        let db = open(path.as_str())?;

        Ok(Engine {
            _db: db,
            _cf: cf.as_str(),
        })
    }
}
