#[macro_use]
extern crate quick_error;

mod errors;
pub use errors::*;

mod serverdata;
pub use serverdata::*;

#[cfg(test)]
mod test_serverdata;
