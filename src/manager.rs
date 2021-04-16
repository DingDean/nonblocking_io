use crate::timer::Queue;
use crate::Interest;
#[cfg(unix)]
use std::os::unix::io::RawFd;
use std::time::Duration;

pub struct Manager {
    kq: i32,
    timers: Queue,
}

impl Manager {
    pub fn new() -> std::io::Result<Self> {
        // 获取 epoll 实例
        let kq = syscall!(kqueue())
            .and_then(|kq| syscall!(fcntl(kq, libc::F_SETFD, libc::FD_CLOEXEC)).map(|_| kq))?;

        Ok(Manager {
            kq,
            timers: Queue::new(),
        })
    }

    pub fn register(
        &self,
        fd: RawFd,
        interests: Vec<Interest>,
        ident: usize,
    ) -> std::io::Result<()> {
        let flags = libc::EV_CLEAR | libc::EV_RECEIPT | libc::EV_ADD;
        let kq = self.kq;

        let filter = interests.iter().fold(0, |a, b| match b {
            Interest::READABLE => a | libc::EVFILT_READ,
            Interest::WRITABLE => a | libc::EVFILT_WRITE,
        });

        let ev = libc::kevent {
            ident: fd as libc::uintptr_t,
            filter,
            flags,
            fflags: 0,
            data: 0,
            udata: ident as *mut libc::c_void,
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
        })
    }

    pub fn retrieve(
        &self,
        events: &mut Vec<libc::kevent>,
        timeout: Option<Duration>,
    ) -> std::io::Result<()> {
        let kq = self.kq;
        let timespec = match timeout {
            Some(t) => libc::timespec {
                tv_sec: t.as_secs() as i64,
                tv_nsec: t.subsec_nanos() as i64,
            },
            None => libc::timespec {
                tv_nsec: 1000,
                tv_sec: 1,
            },
        };
        eprintln!("正在检查是否有事件发生");
        let n = syscall!(kevent(
            kq,
            std::ptr::null(),
            0,
            events.as_mut_ptr(),
            events.capacity() as libc::c_int,
            &timespec
        ))?;
        if n > 0 {
            eprintln!("正在处理 {} 个事件", n);
            unsafe {
                events.set_len(n as usize);
            };
        }
        Ok(())
    }

    pub fn timeout(&mut self, t: Duration, cb: Box<dyn FnOnce()>) {
        self.timers.add(t, cb);
    }

    pub fn run_timers(&mut self) -> Option<Duration> {
        self.timers.run()
    }
}
