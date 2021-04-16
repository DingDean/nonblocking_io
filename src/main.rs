/// Nio: nonblocking i/o
/// ```no_run
/// ```
use libc;
use nio::{Interest, Manager};
use std::io::prelude::*;
use std::net::TcpListener;
#[cfg(unix)]
use std::os::unix::io::AsRawFd;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8888")?;
    // 这是我们要监听的资源
    // let mut stream = TcpStream::connect(listener.local_addr()?)?;

    let manager = Manager::new()?;

    manager.register(listener.as_raw_fd(), vec![Interest::READABLE], 0)?;

    let mut events: Vec<libc::kevent> = Vec::with_capacity(10);

    loop {
        manager.retrieve(&mut events)?;
        events.iter().for_each(|_| {
            eprintln!("正在返回数据...");
            let (mut stream, _) = listener.accept().unwrap();
            stream.write(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
            stream.flush().unwrap();
        })
    }
}
