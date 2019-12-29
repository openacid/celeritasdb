use super::{Engine, Error};

use crate::instance::InstanceID;

impl Engine<'_> {
    /// Dump `values` to `snapshot`, also with the instance that affect them.
    /// The dump process will be in transaction.
    ///
    /// # Examples:
    ///
    /// ```
    /// use crate::epaxos::snapshot::{DBCF, DBPath, Engine};
    /// use crate::epaxos::instance::InstanceID;
    ///
    /// let my_keys = vec!["key bar", "key foo"];
    /// let my_values = vec!["value bar", "value foo"];
    /// let my_instance = InstanceID{
    ///     replica_id: 0,
    ///     num: 11,
    /// };
    ///
    /// let eng = Engine::new(DBPath::Test, DBCF::Default);
    ///
    /// match eng.dump_values(&my_keys, &my_values, &my_instance) {
    ///     Ok(()) => println!("dump values OK!"),
    ///     Err(err) => println!("dump values failed: {}", err),
    /// };
    /// ```
    pub fn dump_values(&self, keys: &Vec<&str>, values: &Vec<&str>, _inst_id: &InstanceID) -> Result<(), Error> {
        // TODO: update instance status to `executed` atomicly.
        self.mset(keys, values)
    }

    /// Load value from `Snapshot`.
    ///
    /// # Examples:
    ///
    /// ```
    /// use crate::epaxos::snapshot::{DBCF, DBPath, Engine};
    ///
    /// let my_key = "key bar";
    ///
    /// let eng = Engine::new(DBPath::Test, DBCF::Default);
    ///
    /// let my_value;
    /// match eng.load_value(my_key) {
    ///     Ok(rst) => my_value = rst,
    ///     Err(err) => println!("load value failed: {}", err),
    /// };
    /// ```
    pub fn load_value(&self, key: &str) -> Result<String, Error> {
        self.get(key)
    }
}

