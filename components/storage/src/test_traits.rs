use crate::AsStorageKey;
#[test]
fn test_as_storage_key_impls() {
    // Vec<u8>
    let v: Vec<u8> = "foo".as_bytes().into();
    let got = v.into_key();
    assert_eq!(got, Vec::<u8>::from("foo".as_bytes()));
    assert_eq!(3, v.key_len());
    assert_eq!(Vec::<u8>::from("foo".as_bytes()), Vec::<u8>::from_key(&v));

    // [u8]
    let v: &[u8] = &"foo".as_bytes();
    let got = v.into_key();
    assert_eq!(Vec::<u8>::from("foo".as_bytes()), got);
    assert_eq!(3, v.key_len());
    // can not convert back
    // assert_eq!("foo".as_bytes().into(), Vec<u8>::from_key(v));
}
