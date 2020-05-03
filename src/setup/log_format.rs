use std::{io, result};

use slog::{Drain, OwnedKVList, Record, KV};
use slog_term::{Decorator, RecordDecorator, Serializer};

pub const TIMESTAMP_FORMAT: &str = "%Y/%m/%d %H:%M:%S%.3f %:z";

/// CeleFormat defines a self-customized log format.
pub struct CeleFormat<D>
where
    D: Decorator,
{
    decorator: D,
}

impl<D> Drain for CeleFormat<D>
where
    D: Decorator,
{
    type Ok = ();
    type Err = io::Error;

    fn log(&self, record: &Record, values: &OwnedKVList) -> result::Result<Self::Ok, Self::Err> {
        self.format(record, values)
    }
}

impl<D> CeleFormat<D>
where
    D: Decorator,
{
    pub fn new(d: D) -> CeleFormat<D> {
        CeleFormat { decorator: d }
    }

    fn format(&self, record: &Record, values: &OwnedKVList) -> io::Result<()> {
        self.decorator.with_record(record, values, |decorator| {
            write_log_header(decorator, record)?;
            write_log_msg(decorator, record)?;
            write_log_fields(decorator, record, values)?;

            decorator.start_whitespace()?;
            writeln!(decorator)?;

            decorator.flush()
        })
    }
}

/// write cele formatted log header
fn write_log_header(rd: &mut dyn RecordDecorator, record: &Record) -> io::Result<()> {
    rd.start_timestamp()?;
    write!(rd, "[{}]", chrono::Local::now().format(TIMESTAMP_FORMAT))?;

    rd.start_whitespace()?;
    write!(rd, " ")?;

    rd.start_level()?;
    write!(rd, "[{}]", record.level().as_short_str())?;

    rd.start_whitespace()?;
    write!(rd, " ")?;

    // there is no `start_line()` or `start_file()`
    rd.start_msg()?;
    write!(rd, "[{}:{}]", record.file(), record.line())
}

/// write log msg
fn write_log_msg(rd: &mut dyn RecordDecorator, record: &Record) -> io::Result<()> {
    rd.start_whitespace()?;
    write!(rd, " ")?;

    rd.start_msg()?;
    write!(rd, "{}", record.msg())
}

/// write log record fields
fn write_log_fields(
    rd: &mut dyn RecordDecorator,
    record: &Record,
    values: &OwnedKVList,
) -> io::Result<()> {
    let mut serializer = Serializer::new(rd, false, true); // no comma, print record kvs just as what write

    record.kv().serialize(record, &mut serializer)?;
    values.serialize(record, &mut serializer)?;

    serializer.finish()
}
