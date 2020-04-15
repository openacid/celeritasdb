use prost::{DecodeError, EncodeError};

quick_error! {
    /// Errors occur when set/get with storage
    #[derive(Debug, PartialEq, Eq)]
    pub enum StorageError {
        DBError(msg: String) {
            from(msg: String) -> (msg)
            display("got db error:{}", msg)
        }

        ProstError(err: String) {
            from(err: DecodeError) -> (format!("{:?}", err))
            from(err: EncodeError) -> (format!("{:?}", err))
            display("prost error: {:?}", err)
        }
    }
}
