use super::{Error, Engine, DBCF, DBPath};

mod dbutil;
use dbutil::*;

impl Engine<'_> {
    /// Open a new Engine to use snapshot.
    ///
    /// # Examples:
    /// ```
    /// use crate::epaxos::snapshot::{DBCF, DBPath, Engine};
    ///
    /// let my_eng;
    /// match Engine::new(DBPath::Test, DBCF::Default) {
    ///     Ok(eng) => my_eng = eng,
    ///     Err(err) => println!("failed to get snapshot engine, failed: {}", err),
    /// };
    /// ```
    pub fn new<'a>(path: DBPath, cf: DBCF) -> Result<Engine<'a>, Error> {
        let db = open(path)?;

        Ok(Engine{
            _db: db,
            _cf: cf.as_str(),
        })
    }
}
