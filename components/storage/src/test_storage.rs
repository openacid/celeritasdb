use std::sync::Arc;

use prost::Message;

use crate::AsStorageKey;
use crate::DBColumnFamily;
use crate::MemEngine;
use crate::NameSpace;
use crate::ObjectKV;
use crate::RawKV;
use crate::Storage;
use crate::WithNameSpace;
use crate::WriteEntry;

#[derive(Clone, PartialEq, Message, Copy, Eq, Ord, PartialOrd, Hash)]
pub struct TestInstance {
    #[prost(int64, tag = "1")]
    pub id: i64,
    #[prost(int64, tag = "2")]
    pub foo: i64,
}

#[derive(Clone, PartialEq, Message, Copy, Eq, Ord, PartialOrd, Hash)]
pub struct TestId {
    #[prost(int64, tag = "1")]
    pub id: i64,
}

impl AsStorageKey for TestId {
    fn into_key(&self) -> Vec<u8> {
        format!("key: {:?}", self.id).as_bytes().to_vec()
    }
    fn from_key(buf: &[u8]) -> Self {
        let s = String::from_utf8_lossy(buf);
        let id = match u64::from_str_radix(&s, 16) {
            Ok(v) => v as i64,
            Err(_) => 0,
        };
        TestId { id }
    }

    fn key_len(&self) -> usize {
        self.into_key().len()
    }
}
impl AsStorageKey for TestInstance {
    fn into_key(&self) -> Vec<u8> {
        format!("key: {:?}", self.id).as_bytes().to_vec()
    }
}

fn new_inst() -> TestInstance {
    TestInstance { id: 0, foo: 1 }
}

pub fn test_base_trait(eng: &dyn RawKV) {
    let none = eng
        .next_raw(
            DBColumnFamily::Record,
            &"init".as_bytes().to_vec(),
            true,
            true,
        )
        .unwrap();
    assert_eq!(none, None);
    let none = eng
        .next_raw(
            DBColumnFamily::Instance,
            &"init".as_bytes().to_vec(),
            true,
            true,
        )
        .unwrap();
    assert_eq!(none, None);

    let none = eng
        .next_raw(
            DBColumnFamily::Record,
            &"init".as_bytes().to_vec(),
            false,
            true,
        )
        .unwrap();
    assert_eq!(none, None);

    let none = eng
        .next_raw(
            DBColumnFamily::Instance,
            &"init".as_bytes().to_vec(),
            false,
            true,
        )
        .unwrap();
    assert_eq!(none, None);

    let none = eng
        .get_raw(DBColumnFamily::Record, &"init".as_bytes().to_vec())
        .unwrap();
    assert_eq!(none, None);
    let none = eng
        .get_raw(DBColumnFamily::Instance, &"init".as_bytes().to_vec())
        .unwrap();
    assert_eq!(none, None);

    let r = eng.delete_raw(DBColumnFamily::Record, &"init".as_bytes().to_vec());
    assert!(r.is_ok());

    let kvs = vec![
        ("k0".as_bytes().to_vec(), "v0".as_bytes().to_vec()),
        ("k1".as_bytes().to_vec(), "v1".as_bytes().to_vec()),
        ("k2".as_bytes().to_vec(), "v2".as_bytes().to_vec()),
        ("k3".as_bytes().to_vec(), "v3".as_bytes().to_vec()),
    ];

    for (k, v) in kvs.iter() {
        eng.set_raw(DBColumnFamily::Status, k, v).unwrap();
    }

    let r = eng.get_raw(DBColumnFamily::Record, &kvs[0].0).unwrap();
    assert_eq!(None, r);
    let r = eng.get_raw(DBColumnFamily::Status, &kvs[0].0).unwrap();
    assert_eq!(r, Some(kvs[0].1.clone()));

    let next = eng
        .next_raw(DBColumnFamily::Record, &kvs[0].0, true, true)
        .unwrap();
    assert!(next.is_none());

    let next = eng
        .next_raw(DBColumnFamily::Status, &kvs[0].0, true, true)
        .unwrap();
    assert_eq!(next, Some(kvs[0].clone()));

    let next = eng
        .next_raw(DBColumnFamily::Status, &kvs[0].0, true, false)
        .unwrap();
    assert_eq!(next, Some(kvs[1].clone()));

    let next = eng
        .next_raw(DBColumnFamily::Status, &kvs[3].0, true, false)
        .unwrap();
    assert!(next.is_none());

    let prev = eng
        .next_raw(DBColumnFamily::Record, &kvs[0].0, false, true)
        .unwrap();
    assert!(prev.is_none());

    let prev = eng
        .next_raw(DBColumnFamily::Status, &kvs[0].0, false, true)
        .unwrap();
    assert_eq!(prev, Some(kvs[0].clone()));

    let prev = eng
        .next_raw(DBColumnFamily::Status, &kvs[0].0, false, false)
        .unwrap();
    assert!(prev.is_none());

    let prev = eng
        .next_raw(DBColumnFamily::Status, &kvs[3].0, false, true)
        .unwrap();
    assert_eq!(prev, Some(kvs[3].clone()));

    eng.delete_raw(DBColumnFamily::Record, &kvs[0].0).unwrap();

    eng.delete_raw(DBColumnFamily::Status, &kvs[0].0).unwrap();
    let r = eng.get_raw(DBColumnFamily::Status, &kvs[0].0).unwrap();
    assert!(r.is_none());

    let k1 = "k1".as_bytes().to_vec();
    let k2 = "k2".as_bytes().to_vec();
    let v1 = "v1".as_bytes().to_vec();
    let v2 = "v2".as_bytes().to_vec();

    let cmds = vec![
        WriteEntry::Set(DBColumnFamily::Record, k1.clone(), v1.clone()),
        WriteEntry::Set(DBColumnFamily::Status, k2.clone(), v2.clone()),
    ];

    eng.write_batch(&cmds).unwrap();
    assert_eq!(
        v1.clone(),
        eng.get_raw(DBColumnFamily::Record, &k1).unwrap().unwrap()
    );
    assert_eq!(
        v2.clone(),
        eng.get_raw(DBColumnFamily::Status, &k2).unwrap().unwrap()
    );

    let cmds = vec![
        WriteEntry::Set(DBColumnFamily::Record, k1.clone(), v1.clone()),
        WriteEntry::Delete(DBColumnFamily::Record, k1.clone()),
    ];

    eng.write_batch(&cmds).unwrap();
    assert_eq!(None, eng.get_raw(DBColumnFamily::Record, &k1).unwrap());
}

pub fn test_objectkv_trait(eng: &Storage) {
    let noninst: Option<TestInstance> = eng.get(DBColumnFamily::Status, &TestId { id: 0 }).unwrap();
    assert_eq!(None, noninst);

    let inst = new_inst();
    eng.set(DBColumnFamily::Status, &TestId { id: 0 }, &inst)
        .unwrap();

    let got: Option<TestInstance> = eng.get(DBColumnFamily::Status, &TestId { id: 0 }).unwrap();
    assert_eq!(Some(inst), got);
}

fn two_storages() -> (Storage, Storage) {
    let eng = MemEngine::new().unwrap();
    let eng = Arc::new(eng);
    let w1 = Storage::new(NameSpace { ns: "1/".into() }, eng.clone());
    let w2 = Storage::new(NameSpace { ns: "2/".into() }, eng.clone());
    (w1, w2)
}

#[test]
fn test_ns_storage() {
    let eng = MemEngine::new().unwrap();
    let eng = Arc::new(eng);
    let w = Storage::new(NameSpace { ns: "5/".into() }, eng);
    test_base_trait(&w);
}
#[test]
fn test_ns_storage_objectkv() {
    {
        let eng = MemEngine::new().unwrap();
        let eng = Arc::new(eng);
        let w = Storage::new(NameSpace { ns: "5/".into() }, eng);
        test_objectkv_trait(&w);
    }
}

#[test]
fn test_storage_no_overriding() {
    let k = "foo".as_bytes().to_vec();
    let v1 = "111".as_bytes().to_vec();
    let v2 = "222".as_bytes().to_vec();

    {
        // rawkv api does not support name space.
        let (w1, w2) = two_storages();

        w1.set_raw(DBColumnFamily::Status, &k, &v1).unwrap();
        w2.set_raw(DBColumnFamily::Status, &k, &v2).unwrap();

        let r = w1.get_raw(DBColumnFamily::Status, &k).unwrap();
        assert_eq!(v2, r.unwrap());

        let r = w2.get_raw(DBColumnFamily::Status, &k).unwrap();
        assert_eq!(v2, r.unwrap());
    }

    {
        // no overriding for get/set
        let (w1, w2) = two_storages();

        w1.set(DBColumnFamily::Status, &k, &v1).unwrap();
        w2.set(DBColumnFamily::Status, &k, &v2).unwrap();

        let r: Option<Vec<u8>> = w1.get(DBColumnFamily::Status, &k).unwrap();
        assert_eq!(v1, r.unwrap());

        let r: Option<Vec<u8>> = w2.get(DBColumnFamily::Status, &k).unwrap();
        assert_eq!(v2, r.unwrap());
    }

    {
        // next/prev is bounded by namespace
        let (w1, w2) = two_storages();
        w1.set(DBColumnFamily::Status, &k, &v1).unwrap();
        w2.set(DBColumnFamily::Status, &k, &v2).unwrap();

        let r = w1.next(DBColumnFamily::Status, &k, true, true).unwrap();
        assert_eq!((k.clone(), v1.clone()), r.unwrap());

        let r = w1
            .next::<Vec<u8>, Vec<u8>>(DBColumnFamily::Status, &k, true, false)
            .unwrap();
        assert!(r.is_none(), "next should not get k/v from other namespace");

        let r = w2
            .next::<Vec<u8>, Vec<u8>>(DBColumnFamily::Status, &k, false, false)
            .unwrap();
        assert!(r.is_none(), "prev should not get k/v from other namespace");
    }

    {
        // write_batch should not override
        let (w1, w2) = two_storages();

        let k1 = "k1".as_bytes().to_vec();
        let k2 = "k2".as_bytes().to_vec();

        let mut b = Vec::new();
        v1.encode(&mut b).unwrap();

        let batch = vec![
            WriteEntry::Set(
                DBColumnFamily::Record,
                w1.prepend_ns(&k1.clone()),
                b.clone(),
            ),
            WriteEntry::Set(
                DBColumnFamily::Status,
                w1.prepend_ns(&k2.clone()),
                b.clone(),
            ),
        ];

        w1.write_batch(&batch).unwrap();

        let r = w1.get(DBColumnFamily::Record, &k1);
        let r = r.unwrap();
        assert_eq!(Some(v1), r);

        let r: Option<Vec<u8>> = w2.get(DBColumnFamily::Record, &k1).unwrap();
        assert!(r.is_none());

        let r: Option<Vec<u8>> = w2.get(DBColumnFamily::Status, &k2).unwrap();
        assert!(r.is_none());
    }
}
