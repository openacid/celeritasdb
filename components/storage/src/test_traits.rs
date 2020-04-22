use crate::NameSpace;

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
