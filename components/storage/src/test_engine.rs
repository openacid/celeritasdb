use crate::AsStorageKey;
use crate::DBColumnFamily;
use crate::ObjectKV;
use crate::RawKV;
use crate::Storage;
use crate::WriteEntry;
use prost::Message;

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
    let none = eng.next_raw(
        DBColumnFamily::Record,
        &"init".as_bytes().to_vec(),
        true,
        true,
    );
    assert_eq!(none, None);
    let none = eng.next_raw(
        DBColumnFamily::Instance,
        &"init".as_bytes().to_vec(),
        true,
        true,
    );
    assert_eq!(none, None);

    let none = eng.next_raw(
        DBColumnFamily::Record,
        &"init".as_bytes().to_vec(),
        false,
        true,
    );
    assert_eq!(none, None);

    let none = eng.next_raw(
        DBColumnFamily::Instance,
        &"init".as_bytes().to_vec(),
        false,
        true,
    );
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

    let next = eng.next_raw(DBColumnFamily::Record, &kvs[0].0, true, true);
    assert!(next.is_none());

    let next = eng.next_raw(DBColumnFamily::Status, &kvs[0].0, true, true);
    assert_eq!(next, Some(kvs[0].clone()));

    let next = eng.next_raw(DBColumnFamily::Status, &kvs[0].0, true, false);
    assert_eq!(next, Some(kvs[1].clone()));

    let next = eng.next_raw(DBColumnFamily::Status, &kvs[3].0, true, false);
    assert!(next.is_none());

    let prev = eng.next_raw(DBColumnFamily::Record, &kvs[0].0, false, true);
    assert!(prev.is_none());

    let prev = eng.next_raw(DBColumnFamily::Status, &kvs[0].0, false, true);
    assert_eq!(prev, Some(kvs[0].clone()));

    let prev = eng.next_raw(DBColumnFamily::Status, &kvs[0].0, false, false);
    assert!(prev.is_none());

    let prev = eng.next_raw(DBColumnFamily::Status, &kvs[3].0, false, true);
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
