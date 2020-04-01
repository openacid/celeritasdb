use std::net::AddrParseError;

quick_error! {
    #[derive(Debug)]
    pub enum ConfError {
        IOError(e: std::io::Error) {
            from(e: std::io::Error) -> (e)
        }

        BadYaml(e: serde_yaml::Error) {
            from(e: serde_yaml::Error) -> (e)
        }

        BadReplication(e: AddrParseError) {
            from(e: AddrParseError) -> (e)
        }
    }
}
