quick_error! {
    /// RangeLookupError defines all error occurs at server level.
    /// It also wraps lower level errors.
    #[derive(Debug, PartialEq)]
    pub enum RangeLookupError {
        NoGroupForKey(k: String) {}
        NoLocalReplicaForKey(k: String) {}
    }
}
