#![feature(is_sorted)]

#[macro_use]
extern crate quick_error;

#[macro_use]
mod testutil;

pub mod conf;

#[macro_use]
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
