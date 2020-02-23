use prost::Message;

include!(concat!(env!("OUT_DIR"), "/command.rs"));

#[cfg(test)]
#[path = "tests/command_tests.rs"]
mod tests;

pub type Key = [u8];
pub type Value = [u8];

impl Command {
    pub fn of(op: OpCode, key: &[u8], value: &[u8]) -> Command {
        Command {
            op: op as i32,
            key: key.to_vec(),
            value: value.to_vec(),
            ..Default::default()
        }
    }
}
