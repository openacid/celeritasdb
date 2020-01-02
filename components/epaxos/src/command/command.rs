#[cfg(test)]
#[path = "./tests/command_tests.rs"]
mod tests;

// FIXME(lsl): for simplicity, string type for both key and value
pub type Key = String;
pub type Value = String;

pub enum OpCode {
    NoOp,
    Set,
    Put,
    Get,
}

pub struct Command {
    pub op: OpCode,
    pub key: Key,
    pub value: Value, // TODO(lsl): must consider type, value and max
}

pub struct Propose {
    command_id: i64,
    command:   Command,
    timestamp: i64,
}

// TODO(lsl): test private function in unit test, should be removed after review
fn lsl_private() {
    println!("lsl-debug: private func");
}
