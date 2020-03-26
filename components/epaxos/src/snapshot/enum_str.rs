/// used to define an enum with an `as_str` function to return it's value.
#[allow(unused_macros)]
macro_rules! enum_str {
    ($name:ident {
        $($key:ident($value:expr))*
    }) => {
        #[derive(Debug, PartialEq)]
        enum $name {
            $($key),*
        }

        impl $name {
            fn as_str(&self) -> &'static str {
                match self {
                    $(
                        &$name::$key => $value
                    ),*
                }
            }

            #[allow(dead_code)]
            fn from_str<'a> (val: &str) -> Result<&'a Self, String> {
                match val
                 {
                    $(
                        $value => Ok(&$name::$key)
                    ),*,
                    _ => Err(format!("{} is not a variant for {}", val, stringify!($name)))
                }
            }
        }
    };

    (pub $name:ident {
        $($key:ident($value:expr))*
    }) => {
        #[derive(Debug, PartialEq)]
        pub enum $name {
            $($key),*
        }

        impl $name {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $(
                        &$name::$key => $value
                    ),*
                }
            }

            #[allow(dead_code)]
            fn from_str<'a>(val: &str) -> Result<&'a Self, String> {
                match val
                 {
                    $(
                        $value => Ok(&$name::$key)
                    ),*,
                    _ => Err(format!("{} is not a variant for {}", val, stringify!($name)))
                }
            }
        }
    };
}

#[test]
fn test_enum_str() {
    enum_str! {
        Work {
            Civilian("work hard")
            Soldier("fight bravely")
        }
    }

    assert_eq!("work hard", Work::Civilian.as_str());
    assert_eq!("fight bravely", Work::Soldier.as_str());

    assert_eq!(Work::from_str("work hard").unwrap(), &Work::Civilian);
    assert_eq!(Work::from_str("fight bravely").unwrap(), &Work::Soldier);

    match Work::from_str("error") {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    };

    mod foo {
        enum_str! {
            pub Status {
                Rich("have lots of money")
                Poor("have no money")
            }
        }
    }

    assert_eq!("have lots of money", foo::Status::Rich.as_str());
    assert_eq!("have no money", foo::Status::Poor.as_str());

    assert_eq!(Work::from_str("work hard").unwrap(), &Work::Civilian);
    assert_eq!(Work::from_str("fight bravely").unwrap(), &Work::Soldier);

    match Work::from_str("error") {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    };
}
