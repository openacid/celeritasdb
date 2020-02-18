use super::super::data;

#[cfg(test)]
#[path = "tests/command_tests.rs"]
mod tests;

pub type Key = [u8];
pub type Value = [u8];

// re-export enum OpCode in data/command.rs
pub use data::OpCode;

// re-export struct Command in data/command.rs
pub use data::Command;

impl Command {
    pub fn of(op: OpCode, key: &[u8], value: &[u8]) -> Command {
        Command {
            op: op,
            key: key.to_vec(),
            value: value.to_vec(),
            ..Default::default()
        }
    }
}
