use crate::io::{self, IoSlice, IoSliceMut};
use crate::mem::ManuallyDrop;
use crate::os::unix::io::{AsFd, BorrowedFd};
use crate::sys::fd::FileDesc;
use crate::sys_common::FromInner;

pub struct Stdin(());
pub struct Stdout(());
pub struct Stderr(());

impl Stdin {
    pub const fn new() -> Stdin {
        Stdin(())
    }
}

impl io::Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        unsafe { ManuallyDrop::new(FileDesc::from_inner(rustix::io::take_stdin())).read(buf) }
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        unsafe {
            ManuallyDrop::new(FileDesc::from_inner(rustix::io::take_stdin())).read_vectored(bufs)
        }
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        true
    }
}

impl Stdout {
    pub const fn new() -> Stdout {
        Stdout(())
    }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unsafe { ManuallyDrop::new(FileDesc::from_inner(rustix::io::take_stdout())).write(buf) }
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        unsafe {
            ManuallyDrop::new(FileDesc::from_inner(rustix::io::take_stdout())).write_vectored(bufs)
        }
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Stderr {
    pub const fn new() -> Stderr {
        Stderr(())
    }
}

impl io::Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unsafe { ManuallyDrop::new(FileDesc::from_inner(rustix::io::take_stderr())).write(buf) }
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        unsafe {
            ManuallyDrop::new(FileDesc::from_inner(rustix::io::take_stderr())).write_vectored(bufs)
        }
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn is_ebadf(err: &io::Error) -> bool {
    err.raw_os_error() == Some(rustix::io::Errno::BADF.raw_os_error())
}

pub const STDIN_BUF_SIZE: usize = crate::sys_common::io::DEFAULT_BUF_SIZE;

pub fn panic_output() -> Option<impl io::Write> {
    Some(Stderr::new())
}

#[stable(feature = "io_safety", since = "1.63.0")]
impl AsFd for io::Stdin {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { rustix::io::stdin() }
    }
}

#[stable(feature = "io_safety", since = "1.63.0")]
impl<'a> AsFd for io::StdinLock<'a> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { rustix::io::stdin() }
    }
}

#[stable(feature = "io_safety", since = "1.63.0")]
impl AsFd for io::Stdout {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { rustix::io::stdout() }
    }
}

#[stable(feature = "io_safety", since = "1.63.0")]
impl<'a> AsFd for io::StdoutLock<'a> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { rustix::io::stdout() }
    }
}

#[stable(feature = "io_safety", since = "1.63.0")]
impl AsFd for io::Stderr {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { rustix::io::stderr() }
    }
}

#[stable(feature = "io_safety", since = "1.63.0")]
impl<'a> AsFd for io::StderrLock<'a> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { rustix::io::stderr() }
    }
}
