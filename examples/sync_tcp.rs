use std::io::prelude::*;
use std::net::TcpListener;

fn sleep(s: u64) {
    std::thread::sleep(std::time::Duration::from_secs(s));
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8888").expect("Failed to bind tcp socket");

    for incoming in listener.incoming() {
        let mut stream = incoming?;
        sleep(5);
        stream
            .write(b"HTTP/1.1 200 OK\r\n\r\n")
            .expect("Failed to write to socket");
        stream.flush().expect("Failed to flush");
    }
    Ok(())
}
