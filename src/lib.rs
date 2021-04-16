/// Helper macro to execute a system call that returns an `io::Result`.
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
