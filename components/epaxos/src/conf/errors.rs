quick_error! {
    #[derive(Debug)]
    pub enum ConfError {
        Error{s: String} {
            from(err: std::io::Error) -> {s: format!("IO Error: {}", err)}
            from(err: serde_yaml::Error) -> {s: format!("Yaml Error: {}", err)}
        }
    }
}
//from(err: std::io::Error) -> (format!("IO Error: {:?}", err))
