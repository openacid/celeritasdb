use std::fmt;
use std::sync::Arc;

use prost::Message;

use crate::StorageError;

/// DBColumnFamily defines several `table`:
/// Record stores a key-value record, e.g., x=3
/// Instance stores replication instances.
/// Status stores status, such as executed instance ids or max instance ids.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DBColumnFamily {
    Record,
    Instance,
    Status,
}

impl DBColumnFamily {
    pub fn all() -> Vec<DBColumnFamily> {
        vec![
            DBColumnFamily::Record,
            DBColumnFamily::Instance,
            DBColumnFamily::Status,
        ]
    }
}

impl From<&DBColumnFamily> for &str {
    fn from(cf: &DBColumnFamily) -> Self {
        match cf {
            DBColumnFamily::Record => return "record",
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

#[derive(Debug)]
pub enum WriteEntry {
    Nil,
    Set(DBColumnFamily, Vec<u8>, Vec<u8>),
    Delete(DBColumnFamily, Vec<u8>),
}

/// AsStorageKey defines API to convert a struct into/from a storage key in byte stream.
pub trait AsStorageKey {
    /// into_key converts a struct into bytes.
    fn into_key(&self) -> Vec<u8>;

    /// key_len returns the length of the result key in bytes. This would be used to pre-alloc mem.
    /// An unoptimized default impl just builds the key and returns the length.
    fn key_len(&self) -> usize {
        // default impl
        self.into_key().len()
    }

    /// from_key converts back from bytes into a struct.
    fn from_key(_buf: &[u8]) -> Self
    where
        Self: std::marker::Sized,
    {
        unimplemented!()
    }
}

impl AsStorageKey for Vec<u8> {
    fn into_key(&self) -> Vec<u8> {
        self.clone()
    }

    fn key_len(&self) -> usize {
        self.len()
    }

    fn from_key(buf: &[u8]) -> Vec<u8> {
        buf.into()
    }
}

impl AsStorageKey for [u8] {
    fn into_key(&self) -> Vec<u8> {
        self.into()
    }

    fn key_len(&self) -> usize {
        self.len()
    }
}

/// WithNameSpace wraps a key with a prefix namespace.
/// E.g.: key: "abc" -> key with namespace "my_namespace/abc";
///
/// It must guarantee that different namespace never generate identical output.
pub trait WithNameSpace {
    /// prepend_ns wraps a key with namespace string, e.g.:
    /// key: "foo" with ns:NameSpace = 5i64: "5/foo".
    fn prepend_ns<K: AsStorageKey + ?Sized>(&self, key: &K) -> Vec<u8>;

    /// strip_ns strip namespace prefix from key, If the key belongs to another namespace, it
    /// returns None.
    fn strip_ns<'a>(&self, key: &'a [u8]) -> Option<&'a [u8]>;
}

#[derive(Clone)]
pub struct NameSpace {
    pub ns: Vec<u8>,
}

impl<T: fmt::Display> From<T> for NameSpace {
    fn from(v: T) -> Self {
        NameSpace {
            ns: format!("{}/", v).into(),
        }
    }
}

/// RawKV defines access API for raw key-value in byte stream.
pub trait RawKV: Send + Sync {
    /// set a new key-value
    fn set_raw(&self, cf: DBColumnFamily, key: &[u8], value: &[u8]) -> Result<(), StorageError>;

    /// get an existing value with key
    fn get_raw(&self, cf: DBColumnFamily, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;

    /// delete a key
    fn delete_raw(&self, cf: DBColumnFamily, key: &[u8]) -> Result<(), StorageError>;

    fn next_raw(
        &self,
        cf: DBColumnFamily,
        key: &[u8],
        forward: bool,
        include: bool,
    ) -> Result<Option<(Vec<u8>, Vec<u8>)>, StorageError>;

    fn write_batch(&self, entrys: &Vec<WriteEntry>) -> Result<(), StorageError>;
}

/// ObjectKV defines access API to access object like KV and provides namespace in order to share a storage with several user.
pub trait ObjectKV: RawKV + WithNameSpace {
    /// set a new key-value
    fn set<OK: AsStorageKey + ?Sized, OV: Message + Default>(
        &self,
        cf: DBColumnFamily,
        key: &OK,
        value: &OV,
    ) -> Result<(), StorageError> {
        let kbytes = self.prepend_ns(key);

        let mut vbytes = vec![];
        value.encode(&mut vbytes)?;

        self.set_raw(cf, &kbytes, &vbytes)
    }

    fn get<OK: AsStorageKey + ?Sized, OV: Message + Default>(
        &self,
        cf: DBColumnFamily,
        key: &OK,
    ) -> Result<Option<OV>, StorageError> {
        let kbytes = self.prepend_ns(key);

        let vbytes = self.get_raw(cf, &kbytes)?;

        let r = match vbytes {
            Some(v) => OV::decode(v.as_slice())?,
            None => return Ok(None),
        };

        Ok(Some(r))
    }

    /// delete a key
    fn delete<OK: AsStorageKey + ?Sized>(
        &self,
        cf: DBColumnFamily,
        key: &OK,
    ) -> Result<(), StorageError> {
        let kbytes = self.prepend_ns(key);
        self.delete_raw(cf, &kbytes)
    }

    /// next returns a key-value pair greater than the given one(include=false),
    /// or greater or equal the given one(include=true)
    fn next<OK: AsStorageKey, OV: Message + Default>(
        &self,
        cf: DBColumnFamily,
        key: &OK,
        forward: bool,
        include: bool,
    ) -> Result<Option<(OK, OV)>, StorageError> {
        // TODO RawKV::next()  should return a Result with StorageError.
        let kbytes = self.prepend_ns(key);
        let nxt = self.next_raw(cf, &kbytes, forward, include)?;
        let (k, v) = match nxt {
            None => return Ok(None),
            Some((k, v)) => (k, v),
        };

        let kbytes = self.strip_ns(&k);
        let kbs = match kbytes {
            None => return Ok(None),
            Some(kbs) => kbs,
        };

        let rstk: OK = OK::from_key(kbs);
        let rstv: OV = OV::decode(v.as_slice())?;
        return Ok(Some((rstk, rstv)));
    }
}

/// Storage exports the storage APIs.
///
/// Rust does not support method with generic types for a trait object.
/// ```ignore
/// trait Typ {
///     fn get<T>() {}
/// }
///
/// Arc<dyn Typ> // illegal
/// ```
/// See https://stackoverflow.com/questions/30938499/why-is-the-sized-bound-necessary-in-this-trait#:~:text=In%20Rust%20all%20generic%20type,%3E%20T%20%7B%20...%20%7D
///
/// Thus in order to define a storage with various underlying engine and
/// generic typed method, we need to separate these two part:
/// ```ignore
/// struct Storage {            // Add method get<T>() etc to this outside struct.
///     inner: Arc<dyn Engine>, // impl various engine at this level.
/// }
/// ```
#[derive(Clone)]
pub struct Storage {
    inner: Arc<dyn RawKV>,
    ns: NameSpace,
}

impl Storage {
    pub fn new<T: Into<NameSpace>>(namespace: T, rawkv: Arc<dyn RawKV>) -> Self {
        Storage {
            ns: namespace.into(),
            inner: rawkv,
        }
    }
    pub fn get_inner(&self) -> &Arc<dyn RawKV> {
        &self.inner
    }
}

impl WithNameSpace for Storage {
    fn prepend_ns<K: AsStorageKey + ?Sized>(&self, key: &K) -> Vec<u8> {
        let pref = &self.ns.ns;
        let mut k: Vec<u8> = Vec::with_capacity(pref.len() + key.key_len());
        k.extend(pref);
        k.extend(&key.into_key());
        k
    }

    fn strip_ns<'a>(&self, key: &'a [u8]) -> Option<&'a [u8]> {
        let pref = &self.ns.ns;
        let got = &key[0..pref.len()];
        if got == pref.as_slice() {
            Some(&key[pref.len()..])
        } else {
            None
        }
    }
}

// Pass raw-kv api to inner storage engine.
impl RawKV for Storage {
    fn set_raw(&self, cf: DBColumnFamily, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        self.get_inner().set_raw(cf, key, value)
    }

    fn get_raw(&self, cf: DBColumnFamily, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        self.get_inner().get_raw(cf, key)
    }

    fn delete_raw(&self, cf: DBColumnFamily, key: &[u8]) -> Result<(), StorageError> {
        self.get_inner().delete_raw(cf, key)
    }

    fn next_raw(
        &self,
        cf: DBColumnFamily,
        key: &[u8],
        forward: bool,
        include: bool,
    ) -> Result<Option<(Vec<u8>, Vec<u8>)>, StorageError> {
        self.get_inner().next_raw(cf, key, forward, include)
    }

    fn write_batch(&self, entrys: &Vec<WriteEntry>) -> Result<(), StorageError> {
        let inn = self.get_inner();
        inn.write_batch(&entrys)
    }
}

impl ObjectKV for Storage {}
