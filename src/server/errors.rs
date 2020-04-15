quick_error! {
    #[derive(Debug)]
    pub enum ServerError {
        RxClosed {}
        NotStarted {}
    }
}
