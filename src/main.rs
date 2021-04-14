use libc;

type Filter = i16;
type UData = libc::intptr_t;

/// Helper macro to execute a system call that returns an `io::Result`.
#[allow(unused_macros)]
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

#[allow(unused_macros)]
macro_rules! kevent {
    ($id: expr, $filter: expr, $flags: expr, $data: expr) => {
        libc::kevent {
            ident: $id as libc::uintptr_t,
            filter: $filter as Filter,
            flags: $flags,
            fflags: 0,
            data: 0,
            udata: $data as UData,
        }
    };
}

fn main() {
    // 获取 epoll 实例
    syscall!(kqueue())
        .and_then(|kq| syscall!(fcntl(kq, libc::F_SETFD, libc::FD_CLOEXEC)).map(|_| kq))
        .map(|kq| {
            // do something with kq
        });
}
