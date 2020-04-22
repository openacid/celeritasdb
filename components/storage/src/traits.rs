use std::fmt::LowerHex;

use crate::StorageError;
use prost::Message;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DBColumnFamily {
    Default,
    Instance,
    Status,
}

impl DBColumnFamily {
    pub fn all() -> Vec<DBColumnFamily> {
        vec![
            DBColumnFamily::Default,
            DBColumnFamily::Instance,
            DBColumnFamily::Status,
        ]
    }
}

impl From<&DBColumnFamily> for &str {
    fn from(cf: &DBColumnFamily) -> Self {
        match cf {
            DBColumnFamily::Default => return "default",
            DBColumnFamily::Instance => return "instance",
            DBColumnFamily::Status => return "status",
        }
    }
}

impl From<DBColumnFamily> for &str {
    fn from(cf: DBColumnFamily) -> Self {
        (&cf).into()
    }
}

pub enum WriteEntry {
    Nil,
    Set(DBColumnFamily, Vec<u8>, Vec<u8>),
    Delete(DBColumnFamily, Vec<u8>),
}

/// NameSpace wraps a key into another key with namespace.
/// E.g.: key: "abc" -> key with namespace "my_namespace/abc";
///
/// It must guarantee that different namespace never generate identical output.
pub trait NameSpace {
    /// wrap_ns wraps a key with namespace string, e.g.:
    /// key: "foo" with ns:NameSpace = 5i64: "5/foo".
    fn wrap_ns(&self, key: &[u8]) -> Vec<u8>;

    /// unwrap_ns strip namespace part from key, If the key belongs to another namespace, it
    /// returns None.
    fn unwrap_ns(&self, key: &[u8]) -> Option<Vec<u8>>;
}

/// impl NameSpace for types with ToString:
/// E.g. for ns:i64=5, it wraps key "foo" to "5/foo"
impl<T: ToString> NameSpace for T {
    fn wrap_ns(&self, key: &[u8]) -> Vec<u8> {
        let mut pref = self.to_string().into_bytes();
        let mut k: Vec<u8> = Vec::with_capacity(pref.len() + 1 + key.len());
        k.append(&mut pref);
        k.push('/' as u8);
        k.append(&mut key.to_vec());
        k
    }
    fn unwrap_ns(&self, key: &[u8]) -> Option<Vec<u8>> {
        let mut pref = self.to_string().into_bytes();
        pref.push('/' as u8);
        let got = &key[0..pref.len()];
        if got == pref.as_slice() {
            Some(key[pref.len()..].to_vec())
        } else {
            None
        }
    }
}

pub fn make_ref_key<T>(typ: &str, id: T) -> Vec<u8>
where
    T: LowerHex,
{
    match typ {
        "max" => format!("/status/max_instance_id/{:016x}", id).into_bytes(),
        "exec" => format!("/status/max_exec_instance_id/{:016x}", id).into_bytes(),
        _ => panic!("unknown type ref"),
    }
}

pub trait ToKey {
    fn to_key(&self) -> Vec<u8>;
}

/// Base offer basic key-value access
pub trait Base: Send + Sync {
    /// set a new key-value
    fn set(&self, cf: DBColumnFamily, key: &[u8], value: &[u8]) -> Result<(), StorageError>;

    /// get an existing value with key
    fn get(&self, cf: DBColumnFamily, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;

    /// delete a key
    fn delete(&self, cf: DBColumnFamily, key: &[u8]) -> Result<(), StorageError>;

    /// next_kv returns a key-value pair greater than the given one(include=false),
    /// or greater or equal the given one(include=true)
    fn next(&self, cf: DBColumnFamily, key: &[u8], include: bool) -> Option<(Vec<u8>, Vec<u8>)>;

    /// prev_kv returns a key-value pair smaller than the given one(include=false),
    /// or smaller or equal the given one(include=true)
    fn prev(&self, cf: DBColumnFamily, key: &[u8], include: bool) -> Option<(Vec<u8>, Vec<u8>)>;

    fn write_batch(&self, entrys: &Vec<WriteEntry>) -> Result<(), StorageError>;
}

/// KV offers functions to store user key/value.
pub trait KV: Base {
    fn set_kv(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        self.set(DBColumnFamily::Default, key, value)
    }

    fn get_kv(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        self.get(DBColumnFamily::Default, key)
    }

    fn delete_kv(&self, key: &[u8]) -> Result<(), StorageError> {
        self.delete(DBColumnFamily::Default, key)
    }

    fn next_kv(&self, key: &[u8], include: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        self.next(DBColumnFamily::Default, key, include)
    }

    fn prev_kv(&self, key: &[u8], include: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        self.prev(DBColumnFamily::Default, key, include)
    }
}

/// ColumnedEngine organizes object in columns.
/// Because the underlying storage is a simple object store,
/// it introduces ColumnId to classify objects.
/// And also it provides APIs to track objects in different columns.
///
/// set_ref(type, K, V) to store V of K.
///
/// E.g.: `set_ref("max", K, V)` to set the "max" V of K.
pub trait ColumnedEngine<K, V>: Base
where
    K: LowerHex + Copy,
    V: Message + Default,
{
    fn set_ref(&self, typ: &str, k: K, v: V) -> Result<(), StorageError> {
        let key = make_ref_key(typ, k);
        let mut value = vec![];
        v.encode(&mut value)?;

        self.set(DBColumnFamily::Status, &key, &value)
    }

    fn get_ref(&self, typ: &str, k: K) -> Result<Option<V>, StorageError> {
        let key = make_ref_key(typ, k);
        let val = self.get(DBColumnFamily::Status, &key)?;

        let val = match val {
            Some(v) => v,
            None => return Ok(None),
        };

        Ok(Some(V::decode(val.as_slice())?))
    }

    /// set_ref_if set ref if the current value satisifies specified condition.
    /// The condition is a lambda takes one arguments: the current value of the ref.
    /// This method should be called with concurrency control.
    ///
    /// # Arguments:
    ///
    /// `typ`: ref type.
    /// `k`: column id of type K.
    /// `v`: object id of type V.
    /// `default`: the default value to feed to `cond` if ref is not found.
    /// `cond`: a lambda takes one argument of type V.
    fn set_ref_if<P>(&self, typ: &str, k: K, v: V, default: V, cond: P) -> Result<(), StorageError>
    where
        Self: Sized,
        P: FnOnce(V) -> bool,
    {
        let r0 = self.get_ref(typ, k)?;
        let r0 = r0.unwrap_or(default);

        if cond(r0) {
            self.set_ref(typ, k, v)
        } else {
            Ok(())
        }
    }
}

/// InstanceEngine offer functions to operate snapshot instances
pub trait InstanceEngine<K, V>: Base
where
    K: ToKey,
    V: Message + ToKey + Default,
{
    /// set an instance
    fn set_instance(&self, v: &V) -> Result<(), StorageError> {
        // TODO does not guarantee in a transaction
        let iid = v.to_key();
        let mut value = vec![];
        v.encode(&mut value)?;

        self.set(DBColumnFamily::Instance, &iid, &value)
    }

    /// get an instance with instance id
    fn get_instance(&self, k: K) -> Result<Option<V>, StorageError> {
        let key = k.to_key();
        let vbs = self.get(DBColumnFamily::Instance, &key)?;
        let r = match vbs {
            Some(v) => V::decode(v.as_slice())?,
            None => return Ok(None),
        };

        Ok(Some(r))
    }
}

pub trait Engine<CK, CV, IK, IV>: KV + ColumnedEngine<CK, CV> + InstanceEngine<IK, IV>
where
    CK: LowerHex + Copy,
    CV: Message + Default,
    IK: ToKey,
    IV: Message + ToKey + Default,
{
}

impl<T> KV for T where T: Base {}
impl<T, K, V> ColumnedEngine<K, V> for T
where
    T: Base,
    K: LowerHex + Copy,
    V: Message + Default,
{
}

impl<T, K, V> InstanceEngine<K, V> for T
where
    T: Base,
    K: ToKey,
    V: Message + ToKey + Default,
{
}

impl<T, CK, CV, IK, IV> Engine<CK, CV, IK, IV> for T
where
    T: Base,
    CK: LowerHex + Copy,
    CV: Message + Default,
    IK: ToKey,
    IV: Message + ToKey + Default,
{
}
