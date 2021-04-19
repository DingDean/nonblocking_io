/// Helper macro to execute a system call that returns an `io::Result`.
/// Shameless borrowed from mio
#[macro_export]
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

pub mod manager;
pub mod timer;
pub use manager::Manager;
pub enum Interest {
    READABLE,
    WRITABLE,
}

pub mod prelude {
    pub use syscall;
}
