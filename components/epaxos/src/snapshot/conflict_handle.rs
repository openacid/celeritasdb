use super::{Engine, Error};
use super::util::inst_id_to_string;

use crate::instance::InstanceID;

impl Engine<'_> {
    /// Dump one key's depend instances
    ///
    /// # Examples:
    /// ```
    /// use crate::epaxos::instance::{InstanceID};
    /// use crate::epaxos::snapshot::{DBCF, DBPath, Engine};
    ///
    /// let my_replica = 0;
    /// let my_key = "my key";
    /// let mut my_dep_instance_ids = vec![];
    ///
    /// let instance_id0 = InstanceID{replica_id: my_replica, num: 0};
    /// let instance_id1 = InstanceID{replica_id: my_replica, num: 1};
    ///
    /// my_dep_instance_ids.push(&instance_id0);
    /// my_dep_instance_ids.push(&instance_id1);
    ///
    /// let eng = Engine::new(DBPath::Test, DBCF::Default);
    ///
    /// match eng.dump_conflicts(my_key, &my_dep_instance_ids) {
    ///     Ok(()) => println!("dump conflicts OK!"),
    ///     Err(err) => println!("dump instance failed: {}", err),
    /// }
    /// ```
    pub fn dump_conflicts(&self, key: &str, inst_ids: &Vec<&InstanceID>) -> Result<(), Error> {
        let mut ks = Vec::with_capacity(inst_ids.len());
        let mut vs = Vec::with_capacity(inst_ids.len());

        for inst_id in inst_ids {
            let k = format!("{}-{}", key, inst_id.replica_id);
            let v = inst_id_to_string(inst_id);

            ks.push(k);
            vs.push(v);
        }

        let ref_ks: Vec<&str> = ks.iter().map(|s| &s[..]).collect();
        let ref_vs: Vec<&str> = vs.iter().map(|s| &s[..]).collect();

        self.mset(&ref_ks, &ref_vs)
    }

    /// Load one key's depend instances
    ///
    /// # Examples:
    /// ```
    /// use crate::epaxos::snapshot::{DBCF, DBPath, Engine};
    ///
    /// let my_key = "my_key";
    ///
    /// let eng = Engine::new(DBPath::Test, DBCF::Default);
    ///
    /// let my_conflicts;
    /// match eng.load_conflicts(my_key) {
    ///     Ok(rst) => my_conflicts = rst,
    ///     Err(err) => println!("load conflicts failed, got error: {}", err),
    /// };
    /// ```
    pub fn load_conflicts(&self, _key: &str) -> Result<Vec<InstanceID>, Error> {
        // scan all key starts with start and end
        return Err(Error::LoadConflictError(format!("snapshot is in constraction...")))
    }
}
