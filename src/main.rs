use libc;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::AsRawFd;

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

fn check_flags(flags: u16) {
    let fs = vec![
        libc::EV_CLEAR,
        libc::EV_RECEIPT,
        libc::EV_ADD,
        libc::EV_EOF,
        libc::EV_POLL,
        libc::EV_DISABLE,
        libc::EV_DISPATCH,
        libc::EV_ENABLE,
        libc::EV_ERROR,
    ];
    fs.iter().for_each(|f| {
        eprintln!("{}", *f & flags == *f);
    })
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8888")?;
    // 这是我们要监听的资源
    // let mut stream = TcpStream::connect(listener.local_addr()?)?;

    // 获取 epoll 实例
    let kq = syscall!(kqueue())
        .and_then(|kq| syscall!(fcntl(kq, libc::F_SETFD, libc::FD_CLOEXEC)).map(|_| kq))?;

    let flag = libc::EV_CLEAR | libc::EV_RECEIPT | libc::EV_ADD;

    let ev = libc::kevent {
        ident: listener.as_raw_fd() as libc::uintptr_t,
        filter: libc::EVFILT_READ,
        flags: flag,
        fflags: 0,
        data: 0,
        udata: 0 as *mut libc::c_void,
    };

    let mut changelist = vec![ev];

    eprintln!("正在注册事件偏好，不会阻塞...");
    syscall!(kevent(
        kq,
        changelist.as_ptr(),
        1,
        changelist.as_mut_ptr(),
        1,
        std::ptr::null(),
    ))
    .map(|n| {
        eprintln!("成功注册 {} 个事件！", n);
        changelist.iter().for_each(|e| {
            if e.flags & libc::EV_ERROR == libc::EV_ERROR {
                eprintln!("错误 {}", std::io::Error::from_raw_os_error(e.data as i32));
            }
        });
        ()
    })
    .or_else(|err| {
        if err.raw_os_error() == Some(libc::EINTR) {
            Ok(())
        } else {
            Err(err)
        }
    })?;

    let mut events: Vec<libc::kevent> = Vec::with_capacity(10);

    loop {
        // t += 1;
        eprintln!("正在检查是否有事件发生");
        let n = syscall!(kevent(
            kq,
            std::ptr::null(),
            0,
            events.as_mut_ptr(),
            events.capacity() as libc::c_int,
            &libc::timespec {
                tv_sec: 5,
                tv_nsec: 5000
            }
        ))?;
        if n > 0 {
            eprintln!("正在处理 {} 个事件", n);
            unsafe {
                events.set_len(n as usize);
            };
            events.iter().for_each(|e| {
                eprintln!("正在返回数据...");
                let (mut stream, addr) = listener.accept().unwrap();
                stream.write(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
                stream.flush().unwrap();
            })
        }
    }

    // Ok(())
}
