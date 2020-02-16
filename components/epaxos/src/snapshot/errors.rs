quick_error! {
    /// Errors occur when set/get with snapshot
    #[derive(Debug)]
    pub enum Error {
        RocksDBError{msg: String} {
            from(msg: String) -> {msg: msg}
            display("rocksdb got error:{}", msg)
        }
        MemDBError {}
    }
}
