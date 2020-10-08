use crate::AccessInstance;
use crate::AccessRecord;
use crate::AccessStatus;
use crate::DBColumnFamily;
use crate::FromKey;
use crate::ObjectKV;
use crate::RawKV;
use crate::Storage;
use crate::ToKey;
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

impl ToKey for TestId {
    fn to_key(&self) -> Vec<u8> {
        format!("key: {:?}", self.id).as_bytes().to_vec()
    }
}
impl FromKey for TestId {
    fn from_key(&mut self, buf: &[u8]) {
        let s = String::from_utf8_lossy(buf);
        self.id = match u64::from_str_radix(&s, 16) {
            Ok(v) => v as i64,
            Err(_) => 0,
        };
    }
}

impl ToKey for TestInstance {
    fn to_key(&self) -> Vec<u8> {
        format!("key: {:?}", self.id).as_bytes().to_vec()
    }
}

fn new_inst() -> TestInstance {
    TestInstance { id: 0, foo: 1 }
}

pub fn test_base_trait(eng: &dyn RawKV) {
    let none = eng.next_raw(DBColumnFamily::Record, &"init".as_bytes().to_vec(), true);
    assert_eq!(none, None);
    let none = eng.next_raw(DBColumnFamily::Instance, &"init".as_bytes().to_vec(), true);
    assert_eq!(none, None);

    let none = eng.prev_raw(DBColumnFamily::Record, &"init".as_bytes().to_vec(), true);
    assert_eq!(none, None);

    let none = eng.prev_raw(DBColumnFamily::Instance, &"init".as_bytes().to_vec(), true);
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

    let next = eng.next_raw(DBColumnFamily::Record, &kvs[0].0, true);
    assert!(next.is_none());

    let next = eng.next_raw(DBColumnFamily::Status, &kvs[0].0, true);
    assert_eq!(next, Some(kvs[0].clone()));

    let next = eng.next_raw(DBColumnFamily::Status, &kvs[0].0, false);
    assert_eq!(next, Some(kvs[1].clone()));

    let next = eng.next_raw(DBColumnFamily::Status, &kvs[3].0, false);
    assert!(next.is_none());

    let prev = eng.prev_raw(DBColumnFamily::Record, &kvs[0].0, true);
    assert!(prev.is_none());

    let prev = eng.prev_raw(DBColumnFamily::Status, &kvs[0].0, true);
    assert_eq!(prev, Some(kvs[0].clone()));

    let prev = eng.prev_raw(DBColumnFamily::Status, &kvs[0].0, false);
    assert!(prev.is_none());

    let prev = eng.prev_raw(DBColumnFamily::Status, &kvs[3].0, true);
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

pub fn test_record_trait(eng: &dyn AccessRecord) {
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

pub fn test_instance_trait(eng: &dyn AccessInstance<TestId, TestInstance>) {
    let noninst = eng.get_instance(TestId { id: 0 }).unwrap();
    assert_eq!(None, noninst);

    let inst = new_inst();
    eng.set_instance(&inst).unwrap();

    let got = eng.get_instance(TestId { id: 0 }).unwrap();
    assert_eq!(Some(inst), got);
}

pub fn test_status_trait(eng: &dyn AccessStatus<TestId, TestInstance>) {
    let noninst = eng.get_status(&TestId { id: 0 }).unwrap();
    assert_eq!(None, noninst);

    let inst = new_inst();
    eng.set_status(&TestId { id: 0 }, &inst).unwrap();

    let got = eng.get_status(&TestId { id: 0 }).unwrap();
    assert_eq!(Some(inst), got);
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
