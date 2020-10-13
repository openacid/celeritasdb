use crate::Value;
use crate::ValueEnum;
#[test]
fn test_value_enum_from() {
    let a = "foo";
    let r = ValueEnum::from(a);
    assert_eq!(ValueEnum::Vbytes("foo".as_bytes().into()), r);

    let a: Vec<u8> = "foo".as_bytes().into();
    let r = ValueEnum::from(a);
    assert_eq!(ValueEnum::Vbytes("foo".as_bytes().into()), r);

    let a: i64 = 3;
    let r = ValueEnum::from(a);
    assert_eq!(ValueEnum::Vi64(3), r);
}

#[test]
fn test_value_from() {
    let a = "foo";
    let r = Value::from(a);
    assert_eq!(
        Value {
            value_enum: Some(ValueEnum::Vbytes("foo".as_bytes().into()))
        },
        r
    );

    let a: Vec<u8> = "foo".as_bytes().into();
    let r = Value::from(a);
    assert_eq!(
        Value {
            value_enum: Some(ValueEnum::Vbytes("foo".as_bytes().into()))
        },
        r
    );

    let a: i64 = 3;
    let r = Value::from(a);
    assert_eq!(
        Value {
            value_enum: Some(ValueEnum::Vi64(3))
        },
        r
    );
}
