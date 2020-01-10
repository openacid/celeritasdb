use super::super::data;

#[cfg(test)]
#[path = "tests/command_tests.rs"]
mod tests;

// FIXME(lsl): for simplicity, string type for both key and value
pub type Key = String;
pub type Value = String;

// re-export enum OpCode in data/command.rs
pub use data::OpCode;

// re-export struct Command in data/command.rs
pub use data::Command;

impl Command {
    pub fn new_command(op: OpCode, key: &str, value: &str) -> Command {
        let mut cmd = Command::new();
        cmd.set_op(op);
        cmd.set_key(key.to_string());
        cmd.set_value(value.to_string());

        return cmd;
    }
}
