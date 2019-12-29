use crate::instance::InstanceID;
use crate::snapshot::{Config, DBCF, DBPath, Engine};

#[test]
fn test_rocks_set() {

    let my_key = "key";
    let my_value = "value";

    let eng = Engine::new(DBPath::Test, DBCF::Default);
    if let Err(err) = eng.set(my_key.as_bytes(), my_value.as_bytes()) {
        println!("set to rocksdb failed, got error: {}", err);
    };
}

#[test]
fn test_rocks_mset() {

    let my_keys = vec!["key1", "key2"];
    let my_values = vec!["value1", "value2"];

    let eng = Engine::new(DBPath::Test, DBCF::Default);
    if let Err(err) = eng.mset(&my_keys, &my_values) {
        println!("mset to rocksdb failed, got error: {}", err);
    };
}

#[test]
fn test_rocks_get() {
    let my_key = "key";
    let my_value = "value";

    let eng = Engine::new(DBPath::Test, DBCF::Default);
    eng.set(my_key.as_bytes(), my_value.as_bytes()).unwrap();

    match eng.get(my_key) {
        Ok(val) => assert_eq!(val, my_value),
        Err(err) => println!("get from rocksdb failed, got error: {}", err),
    };
}

#[test]
fn test_config() {
    let my_instance_id = InstanceID{replica_id: 0, num: 0};

    let eng = Engine::new(DBPath::Test, DBCF::Default);
    eng.dump_config(Config::CommittedInstance, &my_instance_id).unwrap();

    let committed_inst_id = eng.load_config(Config::CommittedInstance, &my_instance_id.replica_id).unwrap();

    assert_eq!(committed_inst_id.replica_id, my_instance_id.replica_id);
    assert_eq!(committed_inst_id.num, my_instance_id.num);
}
