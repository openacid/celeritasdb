use crate::Value;
#[test]
fn test_value_from() {
    let a = "foo";
    let r = Value::from(a);
    assert_eq!(Value::Vbytes("foo".as_bytes().into()), r);

    let a: Vec<u8> = "foo".as_bytes().into();
    let r = Value::from(a);
    assert_eq!(Value::Vbytes("foo".as_bytes().into()), r);

    let a: i64 = 3;
    let r = Value::from(a);
    assert_eq!(Value::Vi64(3), r);
}
