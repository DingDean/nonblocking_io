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

TODO: Fill in code snippets

## Time is Money

Time is also a resource we deeply care about. So **How do I schedule a timeout or repeating time interval with eventloop?**

The main strategy is actually similar to polling for i/o events. We have a **queue for timers managed by ourself** instead of the system. Then we use a BinaryHeap to keep track of the timers registered and run those pending timers before polling for i/o events. 

**How long do we poll for i/o events before we yield to run timers?** A simple yet good enough strategy would be after running the pending timers, we calculate a timespec with which our kevent syscall would be used for timeouts. 

TODO: fill in code snippets

## What's with name?

Well, the name is awefully similar with mio, but actually it's inspired by [Nio](https://www.nio.cn/), a luxury EV company in China, they are awesome.
