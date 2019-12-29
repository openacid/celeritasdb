#[macro_use]
extern crate quick_error;
extern crate rocksdb;

pub mod command;
pub mod conf;
pub mod data;
pub mod instance;
pub mod message;
pub mod replica;
pub mod snapshot;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
