/// Contain some data structs that are used in this module but should be defined out side later.
pub mod out_side {
    pub struct Instance {
        pub id: InstanceID,
        pub replica_id: ReplicaID,
        pub cmds: Vec<Command>,
        pub seq: CommandSeq,
        pub deps: Vec<Instance>,
        pub status: InstanceStatus,
    }

    pub enum Command {
        Put(String, String),
        Get(String),
        Delete(String),
    }

    pub enum InstanceStatus {
        PreAccepted,
        Accepted,
        Committed,
        Executed,
        Purged,
    }

    pub type InstanceID = String;

    pub type ReplicaID = String;

    pub type CommandSeq = String;
}

/// Used to storage `Instance` log. It is expected to be a rocksdb column family instance.
/// But still under constrauction.
type InstanceStorage = Vec<out_side::Instance>;  // TODO:: completely definition detail.

/// An `Instance` interator for load instance from `Snapshot` one by one.
/// Specify a `replica_id` to be a filter.
///
/// # Examples:
///
/// ```
/// let replica_id = some_where::ReplicaID::new();
///
/// let ins_iter = snapshot::InstanceIter::new(replica_id);
///
/// for ins in ins_iter {
///     assert_eq!(&ins.replica_id, "replica001");
/// }
/// ```
pub struct InstanceIter {
    instances: InstanceStorage,
    replica_id: out_side::ReplicaID,
}

impl InstanceIter {
    fn new(replica_id: out_side::ReplicaID) -> InstanceIter {
        InstanceIter {
            instances: InstanceStorage::new(),
            replica_id: replica_id,
        }
    }
}

impl Iterator for InstanceIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        for ins in &self.instances {
            if ins.replica_id == self.replica_id {
                return Some(format!("json instance: {}-{}", ins.replica_id, ins.id));
            }
        }
        None
    }
}

/// Errors may occur when load from `Snapshot`.
pub enum LoadError {
    // TODO:: implements Display.
    NoSuchValue(String),
    NoSuchInstance(String),
}

/// Errors may occur when dump to `Snapshot`.
pub enum DumpError {
    // TODO:: implements Display.
    DumpValueError(String),
    DumpInstanceError(String),
}

/// What returned when load a `value` from `Snapshot`.
pub type LoadValueResult = Result<String, LoadError>;

/// What returned when load a `instance` from `Snapshot`.
pub type LoadInstanceResult = Result<out_side::Instance, LoadError>;

/// What returned when dump to `Snapshot`.
pub type DumpResult = Result<(), DumpError>;

/// Dump an `instance` to `Snapshot`.
///
/// # Examples:
///
/// ```
/// let my_instance = some_where::Instance::new();
///
/// match snapshot::dump_instance(my_instance) {
///     Ok(()) => println!("dump instane OK!"),
///     Err(err) => println!("dump instance failed: {}", err),
/// }
/// ```
pub fn dump_instance(instance: out_side::Instance) -> DumpResult {
    println!("dump instance: {}-{}", instance.replica_id, instance.id);
    Ok(())
}

/// Load an `instance` from `Snapshot`.
///
/// # Examples:
///
/// ```
/// let replica_id = some_where::ReplicaID::new();
/// let instance_id = some_where::InstanceID::new();
///
/// let my_instance = match snapshot::load_instance(replica_id, instance_id) {
///     Ok(rst) => rst,
///     Err(err) = panic!("load instance error: {}", err),
/// }
/// ```
pub fn load_instance(replica_id: out_side::ReplicaID, instance_id: out_side::InstanceID) -> LoadInstanceResult {
    println!("load instance: {}-{}", replica_id, instance_id);
    return Err(LoadError::NoSuchInstance(format!("no such instace: {}-{}", replica_id, instance_id)));
}

/// Dump `values` to `snapshot`, also with the instance that affect them.
/// The dump process will in transaction.
///
/// # Examples:
///
/// ```
/// let my_instance = some_where::get_instance();
/// let result = some_where::execute_instance(my_instance);
///
/// match dump_values(result.keys, result.values, my_instance) {
///     Ok(()) => println!("dump values OK!"),
///     Err(err) => println!("dump values failed: {}", err),
/// }
/// ```
pub fn dump_values(keys: Vec<&str>, values: Vec<&str>, instance: out_side::Instance) -> DumpResult {
    if keys.len() != values.len() {
        return Err(DumpError::DumpValueError("length of keys and values not equals".to_string()));
    }

    let len = keys.len();

    let mut dump_str = String::new();

    for i in 0..len {
        let k = &keys[i];
        let v = &values[i];

        dump_str.push_str(&format!("dump key: {}, values: {};\n", k, v));
    }

    dump_str.push_str(&format!("whose instance is: {}-{}", instance.replica_id, instance.id));

    Ok(())
}

/// Load value from `Snapshot`.
///
/// # Examples:
///
/// ```
/// let my_key = "key001";
///
/// let my_value = match snapshot::load_value(my_key) {
///     Ok(rst) => rst,
///     Err(err) => println!("load value failed: {}", err),
/// }
/// ```
pub fn load_value(key: &str) -> LoadValueResult {
    println!("load key: {}", key);
    Ok("value".to_string())
}

/// Get an `InstanceIter` to load `instance` one by one.
/// Go to InstanceIter definition for more details.
pub fn get_instance_iter(replica_id: out_side::ReplicaID) -> InstanceIter {
    println!("iter instances for replica: {}", replica_id);
    InstanceIter::new(replica_id)
}

// TODO:: define a struct to do "get_for_update --> update --> commit" with values.
// TODO:: define attributes of snapshot like: max instance id in a replica, max executed instance...
// and get interface for user.
