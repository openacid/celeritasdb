use crate::inst;
use crate::instid;
use crate::instids;

use crate::Command;
use crate::Instance;
use crate::InstanceId;
use crate::InstanceIds;
use crate::Record;
use crate::{ReplicaStatus, StorageAPI};

use crate::testutil::new_two_sto;

#[test]
fn test_storage_status() {
    let (s1, s2) = new_two_sto(1, 2);
    // with namespace, operation does not affect each other
    for sto in vec![s1, s2].iter() {
        let noninst = sto.get_status(&ReplicaStatus::Exec).unwrap();
        assert_eq!(None, noninst);

        let iids = instids![(1, 2), (3, 4)];
        sto.set_status(&ReplicaStatus::Exec, &iids).unwrap();

        let got = sto.get_status(&ReplicaStatus::Exec).unwrap();
        assert_eq!(Some(iids), got);
    }
}

#[test]
fn test_storage_instance() {
    let (s1, s2) = new_two_sto(1, 2);

    // with namespace, operation does not affect each other
    for sto in vec![s1, s2].iter() {
        let noninst = sto.get_instance(&instid!(1, 2)).unwrap();
        assert_eq!(None, noninst);

        let inst = inst!((1, 2), (3, _), [(x = y)]);
        sto.set_instance(&inst.instance_id.unwrap(), &inst).unwrap();

        let got = sto.get_instance(&instid!(1, 2)).unwrap();
        assert_eq!(Some(inst), got);
    }
}

#[test]
fn test_storage_record() {
    let (s1, s2) = new_two_sto(1, 2);

    // with namespace, operation does not affect each other
    for sto in vec![s1, s2].iter() {
        let none = sto.next_kv(&"init".as_bytes().to_vec(), true).unwrap();
        assert_eq!(none, None);

        let none = sto.prev_kv(&"init".as_bytes().to_vec(), true).unwrap();
        assert_eq!(none, None);

        let prefix = "k".as_bytes().to_vec();
        let kvs = vec![
            ("k0".as_bytes().to_vec(), "v0".as_bytes().to_vec()),
            ("k1".as_bytes().to_vec(), "v1".as_bytes().to_vec()),
            ("k2".as_bytes().to_vec(), "v2".as_bytes().to_vec()),
            ("k3".as_bytes().to_vec(), "v3".as_bytes().to_vec()),
        ];

        sto.set_kv(&kvs[0].0, &kvs[0].1.clone().into()).unwrap();
        let v0 = sto.get_kv(&kvs[0].0).unwrap().unwrap();
        assert_eq!(v0, kvs[0].1.clone().into());

        let next0 = sto.next_kv(&prefix, true).unwrap().unwrap();
        assert_kv(&next0, &kvs[0]);

        for (k, v) in kvs.iter() {
            sto.set_kv(k, &v.clone().into()).unwrap();
        }

        let next0 = sto.next_kv(&kvs[0].0, true).unwrap().unwrap();
        assert_kv(&next0, &kvs[0]);

        let next1 = sto.next_kv(&kvs[0].0, false).unwrap().unwrap();
        assert_kv(&next1, &kvs[1]);

        let next_last = sto.next_kv(&kvs[3].0.clone(), false).unwrap();
        assert_eq!(next_last, None);

        let prev0 = sto.prev_kv(&kvs[3].0, true).unwrap().unwrap();
        assert_kv(&prev0, &kvs[3]);

        let prev1 = sto.prev_kv(&kvs[3].0, false).unwrap().unwrap();
        assert_kv(&prev1, &kvs[2]);

        let prev2 = sto.prev_kv(&kvs[0].0, false).unwrap();
        assert_eq!(prev2, None);

        sto.delete_kv(&kvs[0].0).unwrap();
        let none = sto.get_kv(&kvs[0].0).unwrap();
        assert!(none.is_none());
    }
}

fn assert_kv(got: &(Vec<u8>, Record), want: &(Vec<u8>, Vec<u8>)) {
    assert_eq!(got.0, want.0);
    assert_eq!(got.1, want.1.clone().into());
}
