use std::sync::Arc;

use storage::{MemEngine, NsStorage, Storage};

use crate::inst;
use crate::instid;
use crate::instids;
use crate::Command;
use crate::Instance;
use crate::InstanceId;
use crate::InstanceIds;
use crate::{ReplicaStatus, StorageAPI};

fn new_eng() -> Storage {
    let eng = Arc::new(MemEngine::new().unwrap());
    let eng = NsStorage::new(5, eng);
    let eng = Storage::new(Arc::new(eng));
    eng
}

//TODO test
#[test]
fn test_status_trait() {
    let eng = new_eng();
    let noninst = eng.get_status(&ReplicaStatus::Exec).unwrap();
    assert_eq!(None, noninst);

    let inst = instids![(1, 2), (3, 4)];
    eng.set_status(&ReplicaStatus::Exec, &inst).unwrap();

    let got = eng.get_status(&ReplicaStatus::Exec).unwrap();
    assert_eq!(Some(inst), got);
}

#[test]
fn test_instance_trait() {
    let eng = new_eng();
    let noninst = eng.get_instance(instid!(1, 2)).unwrap();
    assert_eq!(None, noninst);

    let inst = inst!((1, 2), (3, _), [(x = y)]);
    eng.set_instance(&inst).unwrap();

    let got = eng.get_instance(instid!(1, 2)).unwrap();
    assert_eq!(Some(inst), got);
}

#[test]
fn test_record_trait() {
    let eng = new_eng();
    let none = eng.next_kv(&"init".as_bytes().to_vec(), true);
    assert_eq!(none, None);

    let none = eng.prev_kv(&"init".as_bytes().to_vec(), true);
    assert_eq!(none, None);

    let prefix = "k".as_bytes().to_vec();
    let kvs = vec![
        ("k0".as_bytes().to_vec(), "v0".as_bytes().to_vec()),
        ("k1".as_bytes().to_vec(), "v1".as_bytes().to_vec()),
        ("k2".as_bytes().to_vec(), "v2".as_bytes().to_vec()),
        ("k3".as_bytes().to_vec(), "v3".as_bytes().to_vec()),
    ];

    eng.set_kv(&kvs[0].0, &kvs[0].1).unwrap();
    let v0 = eng.get_kv(&kvs[0].0).unwrap().unwrap();
    assert_eq!(v0, kvs[0].1.clone());

    let next0 = eng.next_kv(&prefix, true);
    assert_eq!(next0, Some(kvs[0].clone()));

    for (k, v) in kvs.iter() {
        eng.set_kv(k, v).unwrap();
    }

    let next0 = eng.next_kv(&kvs[0].0, true);
    assert_eq!(next0, Some(kvs[0].clone()));

    let next1 = eng.next_kv(&kvs[0].0, false);
    assert_eq!(next1, Some(kvs[1].clone()));

    let next_last = eng.next_kv(&kvs[3].0.clone(), false);
    assert_eq!(next_last, None);

    let prev0 = eng.prev_kv(&kvs[3].0, true);
    assert_eq!(prev0, Some(kvs[3].clone()));

    let prev1 = eng.prev_kv(&kvs[3].0, false);
    assert_eq!(prev1, Some(kvs[2].clone()));

    let prev2 = eng.prev_kv(&kvs[0].0, false);
    assert_eq!(prev2, None);

    eng.delete_kv(&kvs[0].0).unwrap();
    let none = eng.get_kv(&kvs[0].0).unwrap();
    assert!(none.is_none());
}
