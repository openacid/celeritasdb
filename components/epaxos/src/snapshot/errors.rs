use prost::{DecodeError, EncodeError};

quick_error! {
    /// Errors occur when set/get with snapshot
    #[derive(Debug, PartialEq, Eq)]
    pub enum Error {
        DBError(msg: String) {
            from(msg: String) -> (msg)
            display("got db error:{}", msg)
        }

        ProtoDecodeError(err: DecodeError) {
            from(err: DecodeError) -> (err)
            display("prost decode error: {:?}", err)
        }

        ProtoEncodeError(err: EncodeError) {
            from(err: EncodeError) -> (err)
            display("prost encode error: {:?}", err)
        }
    }
}
