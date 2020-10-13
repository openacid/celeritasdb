use crate::Record;
use crate::Value;
use crate::ValueEnum;
#[test]
fn test_record_from() {
    let foo: Vec<u8> = "foo".as_bytes().into();
    let a = "foo";
    let r = Record::from(a);
    assert_eq!(
        Record {
            value: Some(Value {
                value_enum: Some(ValueEnum::Vbytes(foo.clone()))
            }),
        },
        r
    );

    let a: Vec<u8> = "foo".as_bytes().into();
    let r = Record::from(a);
    assert_eq!(
        Record {
            value: Some(Value {
                value_enum: Some(ValueEnum::Vbytes(foo.clone()))
            }),
        },
        r
    );

    let a: i64 = 3;
    let r = Record::from(a);
    assert_eq!(
        Record {
            value: Some(Value {
                value_enum: Some(ValueEnum::Vi64(3))
            }),
        },
        r
    );
}

#[test]
fn test_record_to_vec() {
    let r: Record = 3i64.into();
    assert_eq!("3".as_bytes().to_vec(), r.to_vec());

    let r: Record = "foo".into();
    assert_eq!("foo".as_bytes().to_vec(), r.to_vec());
}
