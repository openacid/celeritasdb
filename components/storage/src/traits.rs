use crate::StorageError;
use prost::Message;
use std::sync::Arc;

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

/// ShareByNS defines the API to impl a Storage with namespace support.
pub trait ShareByNS {
    type NS: NameSpace;
    type B: RawKV + ?Sized;

    fn get_ns(&self) -> &Self::NS;
    fn get_storage(&self) -> &Arc<Self::B>;
}

/// NsStorage is a namespace storage based on a shared storage `Base`.
/// Write and read operations are wrapped with a namespace.
pub struct NsStorage<B, NS>
where
    NS: NameSpace,
    B: RawKV + ?Sized,
{
    namespace: NS,
    shared_sto: Arc<B>,
}

impl<B, NS> ShareByNS for NsStorage<B, NS>
where
    B: RawKV + ?Sized,
    NS: NameSpace,
{
    type B = B;
    type NS = NS;

    fn get_storage(&self) -> &Arc<B> {
        &self.shared_sto
    }
    fn get_ns(&self) -> &NS {
        &self.namespace
    }
}

impl<B, NS> NsStorage<B, NS>
where
    NS: NameSpace,
    B: RawKV + ?Sized,
{
    /// new creates a NsStorage with `namespace` and a shared underlying storage `shared_sto`.
    pub fn new(namespace: NS, shared_sto: Arc<B>) -> Self {
        Self {
            namespace,
            shared_sto,
        }
    }
}

/// impl Base storage API for types that impls SharedStorage.
impl<T, B, NS> RawKV for T
where
    B: RawKV + ?Sized,
    NS: NameSpace,
    T: ShareByNS<B = B, NS = NS> + Send + Sync,
{
    fn set_raw(&self, cf: DBColumnFamily, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        self.get_storage()
            .set_raw(cf, &self.get_ns().wrap_ns(key), value)
    }

    fn get_raw(&self, cf: DBColumnFamily, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        self.get_storage().get_raw(cf, &self.get_ns().wrap_ns(key))
    }

    fn delete_raw(&self, cf: DBColumnFamily, key: &[u8]) -> Result<(), StorageError> {
        self.get_storage()
            .delete_raw(cf, &self.get_ns().wrap_ns(key))
    }

    fn next_raw(
        &self,
        cf: DBColumnFamily,
        key: &[u8],
        include: bool,
    ) -> Option<(Vec<u8>, Vec<u8>)> {
        let (k, v) = self
            .get_storage()
            .next_raw(cf, &self.get_ns().wrap_ns(key), include)?;
        let unwrapped = self.get_ns().unwrap_ns(k.as_slice())?;

        Some((unwrapped, v.to_vec()))
    }

    fn prev_raw(
        &self,
        cf: DBColumnFamily,
        key: &[u8],
        include: bool,
    ) -> Option<(Vec<u8>, Vec<u8>)> {
        let (k, v) = self
            .get_storage()
            .prev_raw(cf, &self.get_ns().wrap_ns(key), include)?;
        let unwrapped = self.get_ns().unwrap_ns(k.as_slice())?;

        Some((unwrapped, v.to_vec()))
    }

    fn write_batch(&self, entrys: &Vec<WriteEntry>) -> Result<(), StorageError> {
        let mut es = Vec::with_capacity(entrys.len());

        for en in entrys {
            let e = match en {
                WriteEntry::Nil => WriteEntry::Nil,
                WriteEntry::Set(cf, k, v) => {
                    WriteEntry::Set(*cf, self.get_ns().wrap_ns(k), v.to_vec())
                }
                WriteEntry::Delete(cf, k) => WriteEntry::Delete(*cf, self.get_ns().wrap_ns(k)),
            };
            es.push(e);
        }

        self.get_storage().write_batch(&es)
    }
}

pub trait ToKey {
    fn to_key(&self) -> Vec<u8>;
}

/// RawKV defines access API for raw key-value in byte stream.
pub trait RawKV: Send + Sync {
    /// set a new key-value
    fn set_raw(&self, cf: DBColumnFamily, key: &[u8], value: &[u8]) -> Result<(), StorageError>;

    /// get an existing value with key
    fn get_raw(&self, cf: DBColumnFamily, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;

    /// delete a key
    fn delete_raw(&self, cf: DBColumnFamily, key: &[u8]) -> Result<(), StorageError>;

    /// next_kv returns a key-value pair greater than the given one(include=false),
    /// or greater or equal the given one(include=true)
    fn next_raw(&self, cf: DBColumnFamily, key: &[u8], include: bool)
        -> Option<(Vec<u8>, Vec<u8>)>;

    /// prev_kv returns a key-value pair smaller than the given one(include=false),
    /// or smaller or equal the given one(include=true)
    fn prev_raw(&self, cf: DBColumnFamily, key: &[u8], include: bool)
        -> Option<(Vec<u8>, Vec<u8>)>;

    fn write_batch(&self, entrys: &Vec<WriteEntry>) -> Result<(), StorageError>;
}

/// AccessRecord provides API to access user key/value record.
pub trait AccessRecord: RawKV {
    fn set_kv(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        self.set_raw(DBColumnFamily::Record, key, value)
    }

    fn get_kv(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        self.get_raw(DBColumnFamily::Record, key)
    }

    fn delete_kv(&self, key: &[u8]) -> Result<(), StorageError> {
        self.delete_raw(DBColumnFamily::Record, key)
    }

    fn next_kv(&self, key: &[u8], include: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        self.next_raw(DBColumnFamily::Record, key, include)
    }

    fn prev_kv(&self, key: &[u8], include: bool) -> Option<(Vec<u8>, Vec<u8>)> {
        self.prev_raw(DBColumnFamily::Record, key, include)
    }
}

/// AccessInstance provides API to access instances
pub trait AccessInstance<IK, IV>: RawKV
where
    IK: ToKey,
    IV: Message + ToKey + Default,
{
    /// set an instance
    fn set_instance(&self, v: &IV) -> Result<(), StorageError> {
        // TODO does not guarantee in a transaction
        let iid = v.to_key();
        let mut value = vec![];
        v.encode(&mut value)?;

        self.set_raw(DBColumnFamily::Instance, &iid, &value)
    }

    /// get an instance with instance id
    fn get_instance(&self, k: IK) -> Result<Option<IV>, StorageError> {
        let key = k.to_key();
        let vbs = self.get_raw(DBColumnFamily::Instance, &key)?;
        let r = match vbs {
            Some(v) => IV::decode(v.as_slice())?,
            None => return Ok(None),
        };

        Ok(Some(r))
    }
}

/// AccessStatus provides API to access status
pub trait AccessStatus<STKEY, STVAL>: RawKV
where
    STKEY: ToKey,
    STVAL: Message + Default,
{
    /// set status
    fn set_status(&self, key: &STKEY, value: &STVAL) -> Result<(), StorageError> {
        let kbytes = key.to_key();
        let mut vbytes = vec![];
        value.encode(&mut vbytes)?;

        self.set_raw(DBColumnFamily::Status, &kbytes, &vbytes)
    }

    /// get an status with key
    fn get_status(&self, key: &STKEY) -> Result<Option<STVAL>, StorageError> {
        let kbytes = key.to_key();
        let vbytes = self.get_raw(DBColumnFamily::Status, &kbytes)?;
        let r = match vbytes {
            Some(v) => STVAL::decode(v.as_slice())?,
            None => return Ok(None),
        };

        Ok(Some(r))
    }
}

pub trait Engine<IK, IV, STKEY, STVAL>:
    AccessRecord + AccessInstance<IK, IV> + AccessStatus<STKEY, STVAL>
where
    IK: ToKey,
    IV: Message + ToKey + Default,
    STKEY: ToKey,
    STVAL: Message + Default,
{
}

impl<T> AccessRecord for T where T: RawKV {}

impl<T, IK, IV> AccessInstance<IK, IV> for T
where
    T: RawKV,
    IK: ToKey,
    IV: Message + ToKey + Default,
{
}

impl<T, IK, IV> AccessStatus<IK, IV> for T
where
    T: RawKV,
    IK: ToKey,
    IV: Message + Default,
{
}

impl<T, IK, IV, STKEY, STVAL> Engine<IK, IV, STKEY, STVAL> for T
where
    T: RawKV,
    IK: ToKey,
    IV: Message + ToKey + Default,
    STKEY: ToKey,
    STVAL: Message + Default,
{
}
