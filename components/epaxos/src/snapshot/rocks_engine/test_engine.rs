use tempfile::Builder;

use super::{Base, RocksDBEngine};

#[test]
fn test_base() {
    let mut eng = new_foo_rocks_engine();

    let none = eng.next_kv(&"init".as_bytes().to_vec(), true);
    assert_eq!(none, None);

    let prefix = "k".as_bytes().to_vec();
    let kvs = vec![
        ("k0".as_bytes().to_vec(), "v0".as_bytes().to_vec()),
        ("k1".as_bytes().to_vec(), "v1".as_bytes().to_vec()),
        ("k2".as_bytes().to_vec(), "v2".as_bytes().to_vec()),
        ("k3".as_bytes().to_vec(), "v3".as_bytes().to_vec()),
    ];

    eng.set_kv(kvs[0].0.clone(), kvs[0].1.clone()).unwrap();
    let v0 = eng.get_kv(&kvs[0].0).unwrap();
    assert_eq!(v0, kvs[0].1.clone());

    let next0 = eng.next_kv(&prefix, true);
    assert_eq!(next0, Some(kvs[0].clone()));

    for (k, v) in kvs.clone() {
        eng.set_kv(k, v).unwrap();
    }

    let next0 = eng.next_kv(&kvs[0].0, true);
    assert_eq!(next0, Some(kvs[0].clone()));

    let next1 = eng.next_kv(&kvs[0].0, false);
    assert_eq!(next1, Some(kvs[1].clone()));

    let next_last = eng.next_kv(&kvs[3].0.clone(), false);
    assert_eq!(next_last, None);

    let iter = eng.get_iter(kvs[0].0.clone(), true);
    for (idx, item) in iter.enumerate() {
        assert_eq!(kvs[idx], item)
    }
}

fn new_foo_rocks_engine() -> RocksDBEngine {
    let tmp_root = Builder::new().tempdir().unwrap();
    let db_path = format!("{}/test", tmp_root.path().display());

    return RocksDBEngine::new(&db_path).unwrap();
}
