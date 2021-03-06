# Nio: An Attempt at Non-Blocking I/O

## Our Tiny Billion-RMB Milestone

This repo is merely my attempt at nonblocking i/o with kqueue on macos. Often nonblocking i/o is associated with eventloop. I've been a NodeJS developer for almost 5 years and eventloop is of no stranger to me.

**Yet what I can't build is what I can't understand.**

So here it is, our tiny billion-rmb milestone would be **writing a dead simple eventloop which drives our nonblocking i/o with Rust**.

Choosing Rust is merely a personal preference, but as you may already know, Rust is beloved by communities. A little dose of Rust in you may be helpful.

It's not my goal to explain what eventloop is and why it's needed. For that, I'll reference this official [NodeJS doc](https://nodejs.org/en/docs/guides/event-loop-timers-and-nexttick/) as a nice overview of eventloop.

## kqueue 101

As a NodeJS developer, I've heard of [libuv](#) as the underlying asynchronous I/O based on event loops. The main gest of asynchronous I/O is to poll for I/O events instead of blocking on read/write. This polling functionality is provided by the system, and systems provide different yet similar family of system calls to achive the polling. For macos, this family of system calls is [kqueue](https://man.openbsd.org/kqueue.2). So my main focus would be on kqueue instead of epoll or Windows IOCP.

**What does kqueue provide?** The ability to register your interests on a specific resource and the ability to ask for any events for registered resource. The mental model of asynchronous I/O is simple: when you want to do I/O with a resource, instead of calling `read`/`write` immediately, you first tell the os what you want to do with the resource (read or write) and and then you later ask the os for wheather or not the resource you registered is available to do I/O. You only call `read`/`write` when os tells you it's ok to do so. 

The main problems to solve would be:

1. How do I register my interests with the resource?
2. How do I poll for states?
3. How do I nonblockingly poll for states?

### Wait in the line please

Kqueue, as name implies, there is a queue. If there is a queue, you wait. We can get a handle of this queue with `kqueue()` syscall???
```rust
use libc;
use nio::{Interest, Manager};
use std::io::prelude::*;
use std::net::TcpListener;
#[cfg(unix)]
use std::os::unix::io::AsRawFd;
use std::time::Duration;


fn main() -> std::io::Result<()> {
  let queue = libc::kqueue()
    .and_then(|kq| libc::fcntl(kq, libc::F_SETFD, libc::FD_CLOEXEC))
    .map(|_| kq))
    .unwrap();
}
```

To use this queue, you first need to determine what to put in the queue. For I/O, you care about 2 things: the resource and the events attached to the resource. For this demo, I'll focus on a tcp socket as the resource, and I want to know when can I read from the socket so that I can accept connection. We call this combination of resouce and event as an interest. How do we translate this interest into our kqueue? Comes `kevent`.

`kevent` syscall is used for two related purposes:

1. registered the resource and the events you want to observe.
2. retrieve any events occurs you registered when being called.

First things first, let's accquire the resouce we need:
```rust
fn main() -> std::io::Result<()> {
  // ...

  // we care about a tcp listener
  let listener = TcpListener::bind("127.0.0.1:8888")?;
}
```

Second, we defined the interest we have on this resouce with the help of `kevent` data structure provided by kqueue. (**kevent is the name of the struct and the name of the syscall**):
```rust
fn main() -> std::io::Result<()> {
  // ...

  // we care about a tcp listener
  let listener = TcpListener::bind("127.0.0.1:8888")?;

  // kevent is also a data structure
  let interest = libc::kevent {
    ident: listener.as_raw_fd() as libc::uintptr_t,
    filter: libc::EVFILT_READ, // EVFILT_READ indicates we only care about read on the ident provided,
    flags: libc::EV_CLEAR | libc::EV_RECEIPT | libc::EV_ADD
    fflags: 0,
    data: 0,
    udata: 0 as *mut libc::c_void, // this data would not be touched by kernel but return as is
  };
}
```

Then, we call `kevent` to actually register our interest:
```rust
fn main() -> std::io::Result<()> {
  // ...

  // we care about a tcp listener
  let listener = TcpListener::bind("127.0.0.1:8888")?;

  // kevent is also a data struct
  let interest = libc::kevent {
    ident: listener.as_raw_fd() as libc::uintptr_t,
    filter: libc::EVFILT_READ, // EVFILT_READ indicates we only care about read on the ident provided,
    flags: libc::EV_CLEAR | libc::EV_RECEIPT | libc::EV_ADD | libc::DISABLE
    fflags: 0,
    data: 0,
    udata: 0 as *mut libc::c_void, // this data would not be touched by kernel but return as is
  };

  // kevent function is to register kevent
  let mut changelist = vec![interest];
  let registered_num = libc::kevent(
    kq,
    changelist.as_ptr(),
    1,
    changelist.as_mut_ptr(), // You may wonder why we use the same array but as different pointers, please read one to find out.
    1,
    std::ptr::null(),
  ).unwrap();
}
```
There are two list you should pay attention to: `changelist` and `eventlist`. They correspond to the second and fourth parameter respecively. **What do they do?**. `changelist` is where we pass on our interests with the resource and `eventlist` is where the os return the events associated with your interests if any. To explain more plainly, in a single `kevent` syscall you actually can do two things, you use a `changelist` to register interests and you get events from `eventlist` if there is any. That's why we use `as_mut_ptr()` for our `eventlist` parameter because we need to mutate our array.

**TODO**: A side note of `EV_RECEIPT` flag and it's impact.

#### Hold on to your ticket and ask for information later
So now we registered our interests, let's poll for events:
```rust
// ...
fn main() -> std::io::Result<()> {
  // ...

  let mut events = Vec::with_capacity(10);
  let timespec = libc::timespec {
      tv_sec: 1, // we only wait for 1 second
      tv_nsec: 0,
  };
  let n = libc::kevent(
    kq, 
    std::ptr::null(), // #1 why null ptr
    0, 
    events.as_mut_ptr(), 
    events.capacity() as libc::c_int,
    &timespec
  ).unwrap()
  unsafe { events.set_len(n) }; // #2 why set_len anyway?

  for event in events {
    // do something with them
  }
}
```
We use `kevent` to poll for events and we want the poll would only block for 1 second. 
The things to pay attention to would be #1 and #2. In #1, we use null pointer to tell `kqueue` that we are not adding or modifying any registered interests. In #2, after we finished polling, we need to manually set the length of the `events` array. **Why**? If we not set the length, we wouldn't get any event in the `for event in events` loop because the length of the array is always 0. The length of the array is maintained by our `Rust` program, the os has no way to know how to actually modify this bit of info, os only fill in the events in the momory location pointed by our pointer. So it's our own job to actually set the length. 

**Wait, where is our eventloop?** Well, you ask I provide:

```rust
// ...
fn main() -> std::io::Result<()> {
  // ...

  let mut events = Vec::with_capacity(10);
  loop {
    let timespec = libc::timespec {
        tv_sec: 1, // we only wait for 1 second
        tv_nsec: 0,
    };
    let n = libc::kevent(
      kq, 
      std::ptr::null(), // #1 why null ptr
      0, 
      events.as_mut_ptr(), 
      events.capacity() as libc::c_int,
      &timespec
    ).unwrap()
    unsafe { events.set_len(n) }; // #2 why set_len anyway?

    for event in events {
      // do something with them
    }
  }
}
```
Just like that, we got ourself a **dead simple eventloop**.

## Time is Money

Time is also a resource we deeply care about. So **How do I schedule a timeout or repeating time interval with eventloop?**

The main strategy is actually similar to polling for i/o events. We have a **queue for timers managed by ourself** instead of the system. Then we use a BinaryHeap to keep track of the timers registered and run those pending timers before polling for i/o events. 

**How long do we poll for i/o events before we yield to run timers?** One of the solution is actually a natural consequence of `kevent`. A simple yet good enough strategy would be after running the pending timers, we calculate a timespec with which our kevent syscall would be used for timeouts. 

Let's take a loot at our timer and timer queue briefly:

First, our `Timer`, each has its calculated deadline and an associated callback.
```rust
struct Timer {
  pub deadline: SystemTime,
  pub cb: Box<dyn Fn()>,
}
```
Second, our `TimerQueue`:
```rust
pub struct Queue {
    due: Option<SystemTime>,
    inner: BinaryHeap<Reverse<Timer>>,
}

impl Queue {
  pub fn add(&mut self, t: Duration, cb: Box<dyn Fn()>) { 
    // add timer to the inner BinaryHeap
  }

  pub fn run(&mut self) -> Option<Duration> {
    // run timer
    // calculate a new due
    // return a duration for which our poll should block
  }
}
```
The main gest would be:

1. Our Queue has a explicit due timestamp, which we would compare to current time to determine if there is timer due to run.
2. after running the queue, we get a time duration indicating how long our polling should block(as the timeout in timespec) so that we could wrap back to our timer queue.

Using this queue, we would just simply substitute our fixed timespec with our timer queue calculated one:
```rust
// ...
fn main() -> std::io::Result<()> {
  // ...

  let mut events = Vec::with_capacity(10);
  loop {
    let due = timers.run();
    let timespec = libc::timespec {
        tv_sec: due.as_secs(), // this time we wait as we told from timer
        tv_nsec: due.as_subsec_nanos(),
    };
    let n = libc::kevent(
      kq, 
      std::ptr::null(), // #1 why null ptr
      0, 
      events.as_mut_ptr(), 
      events.capacity() as libc::c_int,
      &timespec
    ).unwrap()
    unsafe { events.set_len(n) }; // #2 why set_len anyway?

    for event in events {
      // do something with them
    }
  }
}
```
Here wraps up our simple implementation of `setTimeout` like function in our eventloop.

## What's with the name?

Well, the name is awefully similar with mio, but actually it's inspired by [Nio](https://www.nio.cn/), a luxury EV company in China, they are awesome.
