use std::cell::RefCell;
use std::io::{self, Write};
use std::str::from_utf8;

use chrono::DateTime;
use slog::{slog_error, slog_info, slog_warn, Drain};

use super::log_format::CeleFormat;
use super::log_format::TIMESTAMP_FORMAT;

thread_local! {
    static BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::new());
}

struct TestWriter;

impl Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        BUFFER.with(|buffer| buffer.borrow_mut().write(buf))
    }
    fn flush(&mut self) -> io::Result<()> {
        BUFFER.with(|buffer| buffer.borrow_mut().flush())
    }
}

#[test]
fn test_cele_format() {
    let decorator = slog_term::PlainSyncDecorator::new(TestWriter);
    let drain = CeleFormat::new(decorator).fuse();
    let logger = slog::Logger::root(drain, slog::o!());

    slog_info!(logger, "logger ready");

    slog_info!(logger, "get request from {}", "test run");
    slog_info!(logger, "get request: "; "key" => "my_key", "req_id" => "my_req_id");

    slog_warn!(logger, "client timeout: "; "timeout_ms" => 3000);

    slog_error!(logger, "failed and got: ";
                    "is_true" => true,
                    "is_none" => None as Option<u8>,
                    "errors" => ?["error1", "error2"], // `?[xxx]` is translated to `format("{:?}", [xxx])`
    );

    let expect = r#"[2020/05/03 10:13:55.035 +08:00] [INFO] [src/setup/test_format.rs:32] logger ready
[2020/05/03 10:13:55.038 +08:00] [INFO] [src/setup/test_format.rs:34] get request from test run
[2020/05/03 10:13:55.038 +08:00] [INFO] [src/setup/test_format.rs:35] get request: key: my_key, req_id: my_req_id
[2020/05/03 10:13:55.038 +08:00] [WARN] [src/setup/test_format.rs:37] client timeout: timeout_ms: 3000
[2020/05/03 10:13:55.038 +08:00] [ERRO] [src/setup/test_format.rs:39] failed and got: is_true: true, is_none: None, errors: ["error1", "error2"]
"#;

    BUFFER.with(|buffer| {
        let buffer = buffer.borrow_mut();
        let output = from_utf8(&*buffer).unwrap();

        for (output_line, expect_line) in output.lines().zip(expect.lines()) {
            let date_time = &output_line[1..31];
            assert!(valid_date_time(date_time));

            let exp_msg = &expect_line[32..];
            let out_msg = &output_line[32..];

            assert_eq!(exp_msg, out_msg);
        }
    })
}

fn valid_date_time(dt: &str) -> bool {
    DateTime::parse_from_str(dt, TIMESTAMP_FORMAT).is_ok()
}
