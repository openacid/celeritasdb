use crate::DBColumnFamily;
use crate::MemEngine;
use crate::ObjectKV;
use crate::{test_engine::*, WithNameSpace};
use prost::Message;
use std::sync::Arc;

use crate::traits::RawKV;
use crate::WriteEntry;

use crate::NameSpace;
use crate::Storage;

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

fn two_storages() -> (Storage, Storage) {
    let eng = MemEngine::new().unwrap();
    let eng = Arc::new(eng);
    let w1 = Storage::new(NameSpace { ns: "1/".into() }, eng.clone());
    let w2 = Storage::new(NameSpace { ns: "2/".into() }, eng.clone());
    (w1, w2)
}

#[test]
fn test_ns_storage_no_overriding() {
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
            WriteEntry::Set(DBColumnFamily::Record, w1.prepend_ns(&k1), b.clone()),
            WriteEntry::Set(DBColumnFamily::Status, w1.prepend_ns(&k2), b.clone()),
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
