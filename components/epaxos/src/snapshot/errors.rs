quick_error! {
    /// Errors occur when set/get with snapshot
    #[derive(Debug, PartialEq)]
    pub enum Error {
        DBError{msg: String} {
            from(msg: String) -> {msg: msg}
            display("rocksdb got error:{}", msg)
        }
        NotFound{}
    }
}
