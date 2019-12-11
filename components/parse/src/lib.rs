use std::fmt;
use std::fmt::Debug;

use std::error::Error;
use std::iter;
use std::num::{ParseFloatError, ParseIntError};

use std::str::{from_utf8, Utf8Error};
use std::sync::mpsc::{Receiver};

#[derive(Debug, Clone)]
pub struct Argument {
    /// The position in the array
    pub pos: usize,
    /// The length in the array
    pub len: usize,
}

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

/// No response was issued
pub enum ResponseError {
    /// The command generated no response
    NoReply,
    /// The command generated no response yet. At a later time, a new command
    /// should be executed, or give up if a None is received.
    /// Only one message will be sent.
    Wait(Receiver<Option<OwnedParsedCommand>>),
}

impl Debug for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &ResponseError::NoReply => write!(f, "NoReply"),
            &ResponseError::Wait(_) => write!(f, "Wait"),
        }
    }
}

impl Response {
    /// Serializes the response into an array of bytes using Redis protocol.
    pub fn as_bytes(&self) -> Vec<u8> {
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
                &(a.iter().map(|el| el.as_bytes()).collect::<Vec<_>>()[..].concat())[..],
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

#[derive(Debug)]
pub struct OwnedParsedCommand {
    data: Vec<u8>,
    pub argv: Vec<Argument>,
}
impl OwnedParsedCommand {
    pub fn new(data: Vec<u8>, argv: Vec<Argument>) -> Self {
        OwnedParsedCommand {
            data: data,
            argv: argv,
        }
    }

    pub fn get_command(&self) -> ParsedCommand {
        ParsedCommand::new(&*self.data, self.argv.clone())
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// The received buffer is valid but needs more data
    Incomplete,
    /// The received buffer is invalid
    BadProtocol(String),
    /// Expected one type of argument and received another
    InvalidArgument,
}

impl ParseError {
    pub fn is_incomplete(&self) -> bool {
        match *self {
            ParseError::Incomplete => true,
            _ => false,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unimpl")
        // return self.response_string().fmt(f);
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::Incomplete => "Incomplete data",
            ParseError::BadProtocol(_) => "Protocol error",
            ParseError::InvalidArgument => "Invalid argument",
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

impl From<Utf8Error> for ParseError {
    fn from(_: Utf8Error) -> ParseError {
        ParseError::InvalidArgument
    }
}

impl From<ParseIntError> for ParseError {
    fn from(_: ParseIntError) -> ParseError {
        ParseError::InvalidArgument
    }
}

impl From<ParseFloatError> for ParseError {
    fn from(_: ParseFloatError) -> ParseError {
        ParseError::InvalidArgument
    }
}

/// A protocol parser
pub struct ParsedCommand<'a> {
    /// The data itself
    data: &'a [u8],
    /// The arguments location and length
    pub argv: Vec<Argument>,
}

impl<'a> ParsedCommand<'a> {
    /// Creates a new parser with the data and arguments provided
    pub fn new(data: &[u8], argv: Vec<Argument>) -> ParsedCommand {
        return ParsedCommand {
            data: data,
            argv: argv,
        };
    }

    /// Gets an i64 from a parameter
    ///
    /// # Examples
    ///
    /// ```
    /// # use parse::{ParsedCommand, Argument};
    /// let parser = ParsedCommand::new(b"-123", vec![Argument { pos: 0, len: 4 }]);
    /// assert_eq!(parser.get_i64(0).unwrap(), -123);
    /// ```
    pub fn get_i64(&self, pos: usize) -> Result<i64, ParseError> {
        let s = self.get_str(pos)?;
        return Ok(s.parse::<i64>()?);
    }

    /// Gets an str from a parameter
    ///
    /// # Examples
    ///
    /// ```
    /// # use parse::{ParsedCommand, Argument};
    /// let parser = ParsedCommand::new(b"foo", vec![Argument { pos: 0, len: 3 }]);
    /// assert_eq!(parser.get_str(0).unwrap(), "foo");
    /// ```
    pub fn get_str(&self, pos: usize) -> Result<&str, ParseError> {
        let data = self.get_slice(pos)?;
        Ok(from_utf8(&data)?)
    }

    /// Gets a &[u8] from a parameter
    ///
    /// # Examples
    ///
    /// ```
    /// # use parse::{ParsedCommand, Argument};
    /// let parser = ParsedCommand::new(b"foo", vec![Argument { pos: 0, len: 3 }]);
    /// assert_eq!(parser.get_slice(0).unwrap(), b"foo");
    /// ```
    pub fn get_slice(&self, pos: usize) -> Result<&[u8], ParseError> {
        if pos >= self.argv.len() {
            return Err(ParseError::InvalidArgument);
        }
        let arg = &self.argv[pos];
        return Ok(&self.data[arg.pos..arg.pos + arg.len]);
    }
}

fn is_print(c: char) -> bool {
    (c as u8).wrapping_sub(b' ') < 0x5F
}

pub fn format_repr(f: &mut fmt::Formatter, s: &[u8]) -> Result<(), fmt::Error> {
    f.write_str("\"")?;
    for c in s {
        match *c {
            0x07 => {
                f.write_str("\\a")?;
                continue;
            }
            0x08 => {
                f.write_str("\\b")?;
                continue;
            }
            _ => (),
        };
        match *c as char {
            '\\' => f.write_str("\\\\"),
            '\"' => f.write_str("\\\""),
            '\n' => f.write_str("\\n"),
            '\r' => f.write_str("\\r"),
            '\t' => f.write_str("\\t"),
            x => {
                if is_print(x) {
                    write!(f, "{}", x)
                } else {
                    write!(f, "\\x{:02x}", x as u8)
                }
            }
        }?
    }
    f.write_str("\"")
}

impl<'a> fmt::Debug for ParsedCommand<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for a in self.argv.iter() {
            format_repr(f, &self.data[a.pos..(a.pos + a.len)])?;
            f.write_str(" ")?;
        }
        Ok(())
    }
}

pub struct Parser {
    data: Vec<u8>,
    pub position: usize,
    pub written: usize,
}

/// Parses the length of the paramenter in the slice
/// Upon success, it returns a tuple with the length of the argument and the
/// length of the parsed length.
fn parse_int(input: &[u8], len: usize, name: &str) -> Result<(Option<usize>, usize), ParseError> {
    if input.len() == 0 {
        return Err(ParseError::Incomplete);
    }
    let mut i = 0;
    let mut argc = 0;
    let mut argco = None;
    while input[i] as char != '\r' {
        let c = input[i] as char;
        if argc == 0 && c == '-' {
            while input[i] as char != '\r' {
                i += 1;
            }
            argco = None;
            break;
        } else if c < '0' || c > '9' {
            return Err(ParseError::BadProtocol(format!("invalid {} length", name)));
        }
        argc *= 10;
        argc += input[i] as usize - '0' as usize;
        i += 1;
        if i == len {
            return Err(ParseError::Incomplete);
        }
        argco = Some(argc);
    }
    i += 1;
    if i == len {
        return Err(ParseError::Incomplete);
    }
    if input[i] as char != '\n' {
        return Err(ParseError::BadProtocol(format!(
            "expected \\r\\n separator, got \\r{}",
            input[i] as char
        )));
    }
    return Ok((argco, i + 1));
}

/// Creates a parser from a buffer.
///
/// # Examples
///
/// ```
/// # use parse::parse;
/// let s = b"*3\r\n$3\r\nSET\r\n$5\r\nmykey\r\n$2\r\n10\r\n";
/// let (parser, len) = parse(s).unwrap();
/// assert_eq!(len, 32);
/// assert_eq!(parser.get_str(0).unwrap(), "SET");
/// assert_eq!(parser.get_str(1).unwrap(), "mykey");
/// assert_eq!(parser.get_i64(2).unwrap(), 10);
/// ```
pub fn parse(input: &[u8]) -> Result<(ParsedCommand, usize), ParseError> {
    let mut pos = 0;
    while input.len() > pos && input[pos] as char == '\r' {
        if pos + 1 < input.len() {
            if input[pos + 1] as char != '\n' {
                return Err(ParseError::BadProtocol(format!(
                    "expected \\r\\n separator, got \
                     \\r{}",
                    input[pos + 1] as char
                )));
            }
            pos += 2;
        } else {
            return Err(ParseError::Incomplete);
        }
    }
    if pos >= input.len() {
        return Err(ParseError::Incomplete);
    }
    if input[pos] as char != '*' {
        return Err(ParseError::BadProtocol(format!(
            "expected '*', got '{}'",
            input[pos] as char
        )));
    }
    pos += 1;
    let len = input.len();
    let (argco, intlen) = parse_int(&input[pos..len], len - pos, "multibulk")?;
    let argc = match argco {
        Some(i) => i,
        None => 0,
    };
    pos += intlen;
    if argc > 1024 * 1024 {
        return Err(ParseError::BadProtocol(
            "invalid multibulk length".to_owned(),
        ));
    }
    let mut argv = Vec::new();
    for i in 0..argc {
        if input.len() == pos {
            return Err(ParseError::Incomplete);
        }
        if input[pos] as char != '$' {
            return Err(ParseError::BadProtocol(format!(
                "expected '$', got '{}'",
                input[pos] as char
            )));
        }
        pos += 1;
        let (argleno, arglenlen) = parse_int(&input[pos..len], len - pos, "bulk")?;
        let arglen = match argleno {
            Some(i) => i,
            None => return Err(ParseError::BadProtocol("invalid bulk length".to_owned())),
        };
        if arglen > 512 * 1024 * 1024 {
            return Err(ParseError::BadProtocol("invalid bulk length".to_owned()));
        }
        pos += arglenlen;
        let arg = Argument {
            pos: pos,
            len: arglen,
        };
        argv.push(arg);
        pos += arglen + 2;
        if pos > len || (pos == len && i != argc - 1) {
            return Err(ParseError::Incomplete);
        }
    }
    Ok((ParsedCommand::new(input, argv), pos))
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            data: vec![],
            position: 0,
            written: 0,
        }
    }

    pub fn allocate(&mut self) {
        if self.position > 0 && self.written == self.position {
            self.written = 0;
            self.position = 0;
        }

        let len = self.data.len();
        let add = if len == 0 {
            16
        } else if self.written * 2 > len {
            len
        } else {
            0
        };

        if add > 0 {
            self.data.extend(iter::repeat(0).take(add));
        }
    }

    pub fn get_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    pub fn is_incomplete(&self) -> bool {
        let data = &(&*self.data)[self.position..self.written];
        match parse(data) {
            Ok(_) => false,
            Err(e) => e.is_incomplete(),
        }
    }

    pub fn next(&mut self) -> Result<ParsedCommand, ParseError> {
        let data = &(&*self.data)[self.position..self.written];
        let (r, len) = parse(data)?;
        self.position += len;
        Ok(r)
    }
}
