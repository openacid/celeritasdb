use crate::snapshot::Error as SnapError;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        EngineError(s: String) {
            from(err: SnapError) -> (format!("{:?}", err))
        }
        CmdNotSupport(s: String)
        SystemError(s: String) {
            from(err: std::time::SystemTimeError) -> (format!("{:?}", err))
        }
    }
}
