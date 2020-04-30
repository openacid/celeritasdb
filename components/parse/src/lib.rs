use std::fmt::Debug;

/// A command response to send to a client
#[derive(PartialEq, Debug)]
pub enum Response {
    /// No data
    Nil,
    /// A number
    Integer(i64),
    /// Binary data
    Data(Vec<u8>),
    /// A simple error string
    Error(String),
    /// A simple status string
    Status(String),
    /// An array of responses that may mix different types
    Array(Vec<Response>),
}

impl Response {
    /// Serializes the response into an array of bytes using Redis protocol.
    pub fn to_vec(&self) -> Vec<u8> {
        return match *self {
            Response::Nil => b"$-1\r\n".to_vec(),
            Response::Data(ref d) => [
                &b"$"[..],
                &format!("{}\r\n", d.len()).into_bytes()[..],
                &d[..],
                &"\r\n".to_owned().into_bytes()[..],
            ]
            .concat(),
            Response::Integer(ref i) => {
                [&b":"[..], &format!("{}\r\n", i).into_bytes()[..]].concat()
            }
            Response::Error(ref d) => [
                &b"-"[..],
                (*d).as_bytes(),
                &"\r\n".to_owned().into_bytes()[..],
            ]
            .concat(),
            Response::Status(ref d) => [
                &b"+"[..],
                (*d).as_bytes(),
                &"\r\n".to_owned().into_bytes()[..],
            ]
            .concat(),
            Response::Array(ref a) => [
                &b"*"[..],
                &format!("{}\r\n", a.len()).into_bytes()[..],
                &(a.iter().map(|el| el.to_vec()).collect::<Vec<_>>()[..].concat())[..],
            ]
            .concat(),
        };
    }

    /// Returns true if and only if the response is an error.
    pub fn is_error(&self) -> bool {
        if let Response::Error(_) = *self {
            true
        } else {
            false
        }
    }

    /// Is the response a status
    pub fn is_status(&self) -> bool {
        if let Response::Status(_) = *self {
            true
        } else {
            false
        }
    }
}
