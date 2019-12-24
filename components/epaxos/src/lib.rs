pub mod command;
pub mod instance;
pub mod replica;
pub mod message;
pub mod conf;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
