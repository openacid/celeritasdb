use crate::test_engine::*;
use crate::DBColumnFamily;
use crate::MemEngine;
use crate::NsStorage;
use std::sync::Arc;

use crate::traits::RawKV;
use crate::WriteEntry;

use crate::NameSpace;
use crate::Storage;

#[test]
fn test_namespace() {
    assert_eq!("5/foo".as_bytes().to_vec(), 5i64.wrap_ns("foo".as_bytes()));
    assert_eq!(
        "bar/foo".as_bytes().to_vec(),
        "bar".wrap_ns("foo".as_bytes())
    );

    assert_eq!(None, 5i64.unwrap_ns("6/foo".as_bytes()));
    assert_eq!(
        Some("foo".as_bytes().to_vec()),
        5i64.unwrap_ns("5/foo".as_bytes())
    );
}

#[test]
fn test_ns_storage() {
    {
        let eng = MemEngine::new().unwrap();
        let eng = Arc::new(eng);
        let w = NsStorage::new(5, eng);
        test_base_trait(&w);
    }

    {
        let eng = MemEngine::new().unwrap();
        let eng = Arc::new(eng);
        let w = NsStorage::new(5, eng);
        test_record_trait(&w);
    }

    {
        let eng = MemEngine::new().unwrap();
        let eng = Arc::new(eng);
        let w = NsStorage::new(5, eng);
        test_instance_trait(&w);
    }

    {
        let eng = MemEngine::new().unwrap();
        let eng = Arc::new(eng);
        let w = NsStorage::new(5, eng);
        test_status_trait(&w);
    }
}
#[test]
fn test_ns_storage_objectkv() {
    {
        let eng = MemEngine::new().unwrap();
        let eng = Arc::new(eng);
        let w = NsStorage::new(5, eng);
        let s = Storage::new(Arc::new(w));
        test_objectkv_trait(&s);
    }
}

#[test]
fn test_ns_storage_no_overriding() {
    let eng = MemEngine::new().unwrap();
    let eng = Arc::new(eng);
    let w1 = NsStorage::new(1, eng.clone());
    let w2 = NsStorage::new(2, eng.clone());

    let k = "foo".as_bytes().to_vec();
    let v1 = "111".as_bytes().to_vec();
    let v2 = "222".as_bytes().to_vec();

    {
        // no overriding for get/set

        w1.set_raw(DBColumnFamily::Status, &k, &v1).unwrap();
        w2.set_raw(DBColumnFamily::Status, &k, &v2).unwrap();

        let r = w1.get_raw(DBColumnFamily::Status, &k).unwrap();
        assert_eq!(v1, r.unwrap());

        let r = w2.get_raw(DBColumnFamily::Status, &k).unwrap();
        assert_eq!(v2, r.unwrap());
    }

    {
        // next/prev is bounded by namespace

        let r = w1.next_raw(DBColumnFamily::Status, &k, true);
        assert_eq!((k.clone(), v1.clone()), r.unwrap());

        let r = w1.next_raw(DBColumnFamily::Status, &k, false);
        assert!(r.is_none(), "next should not get k/v from other namespace");

        let r = w2.prev_raw(DBColumnFamily::Status, &k, false);
        assert!(r.is_none(), "prev should not get k/v from other namespace");
    }

    {
        // write_batch should not override

        let k1 = "k1".as_bytes().to_vec();
        let k2 = "k2".as_bytes().to_vec();

        let batch = vec![
            WriteEntry::Set(DBColumnFamily::Record, k1.clone(), v1.clone()),
            WriteEntry::Set(DBColumnFamily::Status, k2.clone(), v1.clone()),
        ];

        w1.write_batch(&batch).unwrap();

        let r = w1.get_raw(DBColumnFamily::Record, &k1).unwrap();
        assert_eq!(Some(v1), r);

        let r = w2.get_raw(DBColumnFamily::Record, &k1).unwrap();
        assert!(r.is_none());

        let r = w2.get_raw(DBColumnFamily::Status, &k2).unwrap();
        assert!(r.is_none());
    }
}
