/// Nio: nonblocking i/o
/// ```no_run
/// ```
use libc;
use nio::{Interest, Manager};
use std::io::prelude::*;
use std::net::TcpListener;
#[cfg(unix)]
use std::os::unix::io::AsRawFd;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8888")?;
    // 这是我们要监听的资源
    // let mut stream = TcpStream::connect(listener.local_addr()?)?;

    let mut manager = Manager::new()?;

    manager.register(listener.as_raw_fd(), vec![Interest::READABLE], 0)?;
    manager.timeout(
        Duration::from_secs(5),
        Box::new(|| {
            println!("Hello World after 5 seconds");
        }),
    );

    let mut events: Vec<libc::kevent> = Vec::with_capacity(10);

    loop {
        let timeout = manager.run_timers();
        println!("{:?}", timeout);

        // 询问时间是否发生
        manager.retrieve(&mut events, timeout)?;

        loop {
            match events.pop() {
                Some(_) => {
                    eprintln!("正在返回数据...");
                    let (mut stream, _) = listener.accept().unwrap();
                    stream.write(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
                    stream.flush().unwrap();
                }
                None => {
                    break;
                }
            }
        }
    }
}
