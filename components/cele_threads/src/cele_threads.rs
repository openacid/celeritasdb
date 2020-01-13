extern crate quick_error;
extern crate threads_pool;

use quick_error::quick_error;
use std::convert::From;
use std::fmt;
use threads_pool::{ExecutionError, PoolManager, ThreadPool};

quick_error! {
    pub enum CTError {
        Timeout(err: ExecutionError) {}
        InternalErr (err: ExecutionError) {}
    }
}

impl fmt::Debug for CTError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            CTError::Timeout(e) => execute_error_to_str(e),
            CTError::InternalErr(e) => execute_error_to_str(e),
        };

        write!(f, "CTError::{}", msg)
    }
}

impl From<ExecutionError> for CTError {
    fn from(e: ExecutionError) -> Self {
        match e {
            ExecutionError::Timeout => CTError::Timeout(e),
            _ => CTError::InternalErr(e),
        }
    }
}

fn execute_error_to_str(err: &ExecutionError) -> &'static str {
    match err {
        ExecutionError::Timeout => "Timeout(timeout)",
        ExecutionError::Uninitialized => "InternalErr(uninitalized)",
        ExecutionError::Disconnected => "InternalErr(disconnected)",
        ExecutionError::PoolPoisoned => "InternalErr(poolposioned)",
    }
}

pub trait CeleThreadPool {
    fn new(size: usize) -> Self;

    fn dispatch<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<(), CTError>;

    fn destory(&mut self) -> Result<(), CTError>;
}

/// # Examples
/// ```
/// extern crate cele_threads;
/// use cele_threads::{CeleThreadPool, CeleThreads};
///
/// use std::thread::sleep;
/// use std::time::Duration;
///
/// fn main() {
///     let mut pool: CeleThreads = CeleThreadPool::new(10);
///
///     for num in 0 .. 100 {
///         let rst = pool.dispatch(move || {
///             println!("get {}", num);
///             sleep(Duration::from_millis(10));
///         });
///
///         match rst {
///             Ok(_) => println!("successfully dispatch job"),
///             Err(e) => print!("failed to dispatch job: {}", e),
///         }
///     }
///
///     pool.destory().unwrap();
/// }
/// ```
pub struct CeleThreads {
    pub pool: ThreadPool,
}

impl CeleThreadPool for CeleThreads {
    fn new(size: usize) -> Self {
        let mut pool = ThreadPool::new(size);

        // multiple default settings
        pool.toggle_auto_scale(true);

        return CeleThreads { pool };
    }

    fn dispatch<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<(), CTError> {
        match self.pool.execute(f) {
            Ok(r) => Ok(r),
            Err(e) => Err(CTError::from(e)),
        }
    }

    fn destory(&mut self) -> Result<(), CTError> {
        self.pool.clear();
        self.pool.close();

        return Ok(());
    }
}
