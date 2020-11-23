extern crate libc;

use std::net::TcpStream;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use std::{io, mem, ptr, time};

pub struct FdSet(libc::fd_set);

impl FdSet {
    pub fn new() -> FdSet {
        unsafe {
            let mut raw_fd_set = mem::MaybeUninit::<libc::fd_set>::uninit().assume_init();
            libc::FD_ZERO(&mut raw_fd_set);
            FdSet(raw_fd_set)
        }
    }
    pub fn clear(&mut self, fd: RawFd) {
        unsafe { libc::FD_CLR(fd, &mut self.0) }
    }
    pub fn set(&mut self, fd: RawFd) {
        unsafe { libc::FD_SET(fd, &mut self.0) }
    }
    pub fn is_set(&mut self, fd: RawFd) -> bool {
        unsafe { libc::FD_ISSET(fd, &mut self.0) }
    }
}

fn to_fdset_ptr(opt: Option<&mut FdSet>) -> *mut libc::fd_set {
    match opt {
        None => ptr::null_mut(),
        Some(&mut FdSet(ref mut raw_fd_set)) => raw_fd_set,
    }
}
fn to_ptr<T>(opt: Option<&T>) -> *const T {
    match opt {
        None => ptr::null::<T>(),
        Some(p) => p,
    }
}

pub fn pselect(
    nfds: libc::c_int,
    readfds: Option<&mut FdSet>,
    writefds: Option<&mut FdSet>,
    errorfds: Option<&mut FdSet>,
    timeout: Option<&libc::timespec>,
    sigmask: Option<&libc::sigset_t>,
) -> io::Result<usize> {
    fn to_fdset_ptr(opt: Option<&mut FdSet>) -> *mut libc::fd_set {
        match opt {
            None => ptr::null_mut(),
            Some(&mut FdSet(ref mut raw_fd_set)) => raw_fd_set,
        }
    }
    fn to_ptr<T>(opt: Option<&T>) -> *const T {
        match opt {
            None => ptr::null::<T>(),
            Some(p) => p,
        }
    }

    match unsafe {
        libc::pselect(
            nfds,
            to_fdset_ptr(readfds),
            to_fdset_ptr(writefds),
            to_fdset_ptr(errorfds),
            to_ptr(timeout),
            to_ptr(sigmask),
        )
    } {
        -1 => Err(io::Error::last_os_error()),
        res => Ok(res as usize),
    }
}

pub fn make_timespec(duration: time::Duration) -> libc::timespec {
    libc::timespec {
        tv_sec: duration.as_secs() as i64,
        tv_nsec: duration.subsec_nanos() as i64,
    }
}

pub fn connect_to_localhost_2000() -> TcpStream {
    TcpStream::connect("localhost:2000").expect("Failed to connect to localhost 2000")
}

fn main() {
    let ten_seconds = time::Duration::new(10, 0);
    let mut fd_set = FdSet::new();

    let stream1 = connect_to_localhost_2000();
    let raw_fd1 = stream1.as_raw_fd();

    let stream2 = connect_to_localhost_2000();
    let raw_fd2 = stream2.as_raw_fd();

    let stream3 = connect_to_localhost_2000();
    let raw_fd3 = stream3.as_raw_fd();

    let max_fd = raw_fd1.max(raw_fd2.max(raw_fd3));

    println!("Socket 1: {}", raw_fd1);
    println!("Socket 2: {}", raw_fd2);
    println!("Socket 3: {}", raw_fd3);

    fd_set.set(raw_fd1);
    fd_set.set(raw_fd2);
    fd_set.set(raw_fd3);

    match pselect(
        max_fd + 1,
        Some(&mut fd_set),                 // read
        None,                              // write
        None,                              // error
        Some(&make_timespec(ten_seconds)), // timeout
        None,                              // mask
    ) {
        Ok(res) => {
            println!("select result: {}", res);

            let range = std::ops::Range {
                start: 0,
                end: max_fd + 1,
            };
            for i in range {
                if (fd_set).is_set(i) {
                    println!("Socket {} received something!", i);
                }
            }
        }
        Err(err) => {
            println!("Failed to select: {:?}", err);
        }
    }
}
