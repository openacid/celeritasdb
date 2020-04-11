#![feature(is_sorted)]

#[macro_use]
extern crate quick_error;

#[macro_use]
mod testutil;

pub mod conf;
mod serverdata;
mod service;

#[macro_use]
pub mod qpaxos;
pub mod replica;
pub mod replication;
pub mod snapshot;
pub mod tokey;

pub use serverdata::*;
pub use service::*;
