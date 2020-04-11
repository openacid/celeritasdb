use parse::Response;

quick_error! {
    /// RangeLookupError defines all error occurs at server level.
    /// It also wraps lower level errors.
    #[derive(Debug, PartialEq)]
    pub enum RangeLookupError {
        NoGroupForKey(k: String) {}
        NoLocalReplicaForKey(k: String) {}
    }
}

impl From<RangeLookupError> for Response {
    fn from(e: RangeLookupError) -> Response {
        match e {
            RangeLookupError::NoGroupForKey(k) => {
                Response::Error(format!("No gruop serves: {}", k))
            }
            RangeLookupError::NoLocalReplicaForKey(k) => {
                Response::Error(format!("No replica serve: {}", k))
            }
        }
    }
}
