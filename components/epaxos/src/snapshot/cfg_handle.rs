use super::{Error, Engine};
use super::util::{inst_id_to_string, inst_id_from_string};

use crate::instance::InstanceID;
use crate::replica::ReplicaID;

pub enum Config {
    CommittedInstance,
    ExecutedInstance,
    CurrentInstance,
}

impl Config {
    fn to_string(&self) -> String {
        match self {
            Config::CommittedInstance => String::from("committed"),
            Config::CurrentInstance => String::from("current"),
            Config::ExecutedInstance => String::from("executed"),
        }
    }
}

impl Engine<'_> {
    /// Dump one config(value is an `instance`)
    ///
    /// # Examples:
    /// ```
    /// use crate::epaxos::instance::InstanceID;
    /// use crate::epaxos::snapshot::{Config, DBCF, DBPath, Engine};
    ///
    /// let my_instance_id = InstanceID{replica_id: 0, num: 0};
    ///
    /// let eng = Engine::new(DBPath::Test, DBCF::Default);
    ///
    /// match eng.dump_config(Config::CommittedInstance, &my_instance_id) {
    ///     Ok(()) => println!("dump committed instance OK!"),
    ///     Err(err) => println!("dump committed instance failed, got error: {}", err),
    /// };
    /// ```
    pub fn dump_config(&self, cfg: Config, inst_id: &InstanceID) -> Result<(), Error> {
        let k = format!("{}-{}", cfg.to_string(), inst_id.replica_id);
        let v = inst_id_to_string(inst_id);

        self.set(k.as_bytes(), v.as_bytes())
    }

    /// Load one config to get an `instance`
    ///
    /// # Examples:
    /// ```
    /// use crate::epaxos::replica::ReplicaID;
    /// use crate::epaxos::snapshot::{Config, DBCF, DBPath, Engine};
    ///
    /// let my_replica = 0;
    ///
    /// let eng = Engine::new(DBPath::Test, DBCF::Default);
    ///
    /// let my_executed;
    /// match eng.load_config(Config::ExecutedInstance, &my_replica) {
    ///     Ok(rst) => my_executed = rst,
    ///     Err(err) => println!("load executed instance failed, got error: {}", err),
    /// };
    /// ```
    pub fn load_config(&self, cfg: Config, rep_id: &ReplicaID) -> Result<InstanceID, Error> {
        let k = format!("{}-{}", cfg.to_string(), rep_id);
        let v = self.get(&k)?;

        inst_id_from_string(&v)
    }
}
