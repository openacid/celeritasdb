#[macro_use]
extern crate quick_error;

#[macro_use]
mod testutil;

pub mod command;
pub mod conf;
pub mod instance;
pub mod message;
pub mod qpaxos;
pub mod replica;
pub mod snapshot;
pub mod tokey;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
