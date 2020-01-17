/// used to define an enum with an `as_str` function to return it's value.
macro_rules! enum_str {
    ($name:ident {
        $($key:ident($value:expr))*
    }) => {
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
        }
    };

    (pub $name:ident {
        $($key:ident($value:expr))*
    }) => {
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
}
