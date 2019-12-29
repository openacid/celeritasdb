use super::{Engine, Error};
use super::util::inst_id_to_string;

use crate::instance::{Instance, InstanceID};
use crate::replica::{ReplicaID};

impl Engine<'_> {
    /// Dump an `instance` to `Snapshot`.
    ///
    /// # Examples:
    /// ```
    /// use crate::epaxos::command::{Command, OpCode};
    /// use crate::epaxos::instance::{Instance, InstanceID, InstanceStatus};
    /// use crate::epaxos::replica::{BallotNum, ReplicaID};
    /// use crate::epaxos::snapshot::{DBCF, DBPath, Engine};
    ///
    /// let my_replica = 0;
    ///
    /// let my_instance = Instance{
    ///     status: InstanceStatus::NA,
    ///     cmds: vec![Command{op: OpCode::Set, key: String::from("my key"), value: String::from("my value")}, ],
    ///     ballot: BallotNum{epoch: 0, num: 1, replica_id: my_replica},
    ///     seq: 0,
    ///     deps: vec![InstanceID{replica_id: 1, num: 1}, ],
    /// };
    ///
    /// let inst_id = InstanceID{
    ///     replica_id: my_replica,
    ///     num: 1,
    /// };
    ///
    /// let eng = Engine::new(DBPath::Test, DBCF::Default);
    ///
    /// match eng.dump_instance(&inst_id, &my_instance) {
    ///     Ok(()) => println!("dump instane OK!"),
    ///     Err(err) => println!("dump instance failed: {}", err),
    /// }
    /// ```
    pub fn dump_instance(&self, inst_id: &InstanceID, _instance: &Instance) -> Result<(), Error> {
        let _inst_k = inst_id_to_string(inst_id);

        // TODO: marshal instacne

        return Err(Error::DumpInstanceError(format!("snapshot is in constrauction...")));
    }

    /// Load an `instance` from `Snapshot`.
    ///
    /// # Examples:
    /// ```
    /// use crate::epaxos::instance::InstanceID;
    /// use crate::epaxos::snapshot::{DBCF, DBPath, Engine};
    ///
    /// let instance_id = InstanceID{
    ///   replica_id: 0,
    ///   num: 1,
    /// };
    ///
    /// let eng = Engine::new(DBPath::Test, DBCF::Default);
    ///
    /// let my_instance;
    /// match eng.load_instance(&instance_id) {
    ///     Ok(rst) => my_instance = rst,
    ///     Err(err) => println!("load instance error: {}", err),
    /// };
    /// ```
    pub fn load_instance(&self, inst_id: &InstanceID) -> Result<Instance, Error> {
        let inst_k = inst_id_to_string(inst_id);
        let _inst_v = self.get(&inst_k)?;

        // TODO: unmarshal Instance

        return Err(Error::LoadInstanceError(format!("snapshot is in constrauction...")));
    }

    /// Get all `instance`s from `Snapshot` of one replica.
    pub fn scan_instances(&self, _rep_id: &ReplicaID) -> Result<Vec<(InstanceID, Instance)>, Error> {
        // TODO: iter with start and end
        return Err(Error::ScanInstancesError(format!("snapshot is in constrauction...")));
    }
}
