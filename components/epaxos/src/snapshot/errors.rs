use std::num::ParseIntError;

quick_error! {
    /// Errors occur when dump or load with snapshot
    #[derive(Debug)]
    pub enum Error {
        DumpValueError(msg: String) {
            display("dump key value failed, message:{}", msg)
        }

        LoadValueError(msg: String) {
            display("load key failed, message:{}", msg)
        }

        DumpInstanceError(msg: String) {
            display("dump instance failed, message:{}", msg)
        }

        LoadInstanceError(msg: String) {
            display("load instance failed, message:{}", msg)
        }

        ScanInstancesError(msg: String) {
            display("scan instances failed, message:{}", msg)
        }

        DumpConfigError(msg: String) {
            display("dump config failed, message:{}", msg)
        }

        LoadConfigError(msg: String) {
            display("load config failed, message:{}", msg)
        }

        DumpConflictError(msg: String) {
            display("dump conflict failed, message:{}", msg)
        }

        LoadConflictError(msg: String) {
            display("load conflict failed, message:{}", msg)
        }

        ParseToInstanceIDError {msg: String} {
            from(err: ParseIntError) -> {
                msg: format!("parse to int failed, {}", err)
            }
            display("parse string to instance failed, message:{}", msg)
        }

        RocksDBError{msg: String} {
            from(msg: String) -> {msg: msg}
            display("rocksdb got error:{}", msg)
        }
    }
}
