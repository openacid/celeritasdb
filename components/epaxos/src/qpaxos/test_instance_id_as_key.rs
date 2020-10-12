use crate::instid;

use crate::InstanceId;
use storage::AsStorageKey;

#[test]
fn test_instance_id_as_key() {
    let cases = vec![(instid!(1, 2), "/instance/0000000000000001/0000000000000002")];
    for (iid, want) in cases.iter() {
        let key = iid.into_key();
        assert_eq!(want.as_bytes(), key.as_slice());
        assert_eq!(43, iid.key_len());

        let iidgot = InstanceId::from_key(&key);
        assert_eq!(*iid, iidgot);
    }
}
