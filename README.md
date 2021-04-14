# Nio: Write Yourself An Eventloop

我们的目标: **在 MacOS 上用 Rust 写一个简单的 event loop**

1. 如何调用系统指令？**引入 libc 模块**
2. 如果监听系统消息？**使用 kqueue**
3. 为什么不是 epoll？**因为类 bsd 系统上只有 kqueue, epoll 是 linux 系才有的指令**
4. 为什么在调用了 `kqueue` 后，要调用 `fcntl` 复制一个 `fd` 呢？**Explaine Me!!!! 官方的解释可以在 https://man7.org/linux/man-pages/man2/open.2.html 的 O_CLOEXEC 部分找到，但需要自己的理解。**
5. 我如何监听想要关注的事件和资源？**(调用 kevent 方法)**

## 所有由代码直接调用的系统指令

1. [kqueue](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kqueue.2.html) 用于获取系统的 `kqueue` 实例。
2. [fcntl](https://man7.org/linux/man-pages/man2/fcntl.2.html) 操作文件的指令集。
