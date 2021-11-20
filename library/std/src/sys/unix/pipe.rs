use crate::io::{self, IoSlice, IoSliceMut};
use crate::os::unix::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, RawFd};
use crate::sys::fd::FileDesc;
use crate::sys_common::{FromInner, IntoInner};

use rustix::io::{PipeFlags, PollFd, PollFlags};

////////////////////////////////////////////////////////////////////////////////
// Anonymous pipes
////////////////////////////////////////////////////////////////////////////////

pub struct AnonPipe(FileDesc);

pub fn anon_pipe() -> io::Result<(AnonPipe, AnonPipe)> {
    // The only known way right now to create atomically set the CLOEXEC flag is
    // to use the `pipe2` syscall. This was added to Linux in 2.6.27, and some
    // other targets also have it.
    cfg_if::cfg_if! {
        if #[cfg(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "linux",
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "redox"
        ))] {
            let (fds0, fds1) = rustix::io::pipe_with(PipeFlags::CLOEXEC)?;
            Ok((AnonPipe(FileDesc::from_inner(fds0)), AnonPipe(FileDesc::from_inner(fds1))))
        } else {
            let (fds0, fds1) = rustix::io::pipe()?;
            let fd0 = FileDesc::from_inner(fds0);
            let fd1 = FileDesc::from_inner(fds1);
            fd0.set_cloexec()?;
            fd1.set_cloexec()?;
            Ok((AnonPipe(fd0), AnonPipe(fd1)))
        }
    }
}

impl AnonPipe {
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }

    #[inline]
    pub fn is_read_vectored(&self) -> bool {
        self.0.is_read_vectored()
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }

    #[inline]
    pub fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }
}

impl IntoInner<FileDesc> for AnonPipe {
    fn into_inner(self) -> FileDesc {
        self.0
    }
}

pub fn read2(p1: AnonPipe, v1: &mut Vec<u8>, p2: AnonPipe, v2: &mut Vec<u8>) -> io::Result<()> {
    // Set both pipes into nonblocking mode as we're gonna be reading from both
    // in the `select` loop below, and we wouldn't want one to block the other!
    let p1 = p1.into_inner();
    let p2 = p2.into_inner();
    p1.set_nonblocking(true)?;
    p2.set_nonblocking(true)?;

    let mut fds = [PollFd::new(&p1, PollFlags::IN), PollFd::new(&p2, PollFlags::IN)];
    loop {
        // wait for either pipe to become readable using `poll`
        rustix::io::with_retrying(|| rustix::io::poll(&mut fds, -1))?;

        if !fds[0].revents().is_empty() && read(&p1, v1)? {
            p2.set_nonblocking(false)?;
            return p2.read_to_end(v2).map(drop);
        }
        if !fds[1].revents().is_empty() && read(&p2, v2)? {
            p1.set_nonblocking(false)?;
            return p1.read_to_end(v1).map(drop);
        }
    }

    // Read as much as we can from each pipe, ignoring EWOULDBLOCK or
    // EAGAIN. If we hit EOF, then this will happen because the underlying
    // reader will return Ok(0), in which case we'll see `Ok` ourselves. In
    // this case we flip the other fd back into blocking mode and read
    // whatever's leftover on that file descriptor.
    fn read(fd: &FileDesc, dst: &mut Vec<u8>) -> Result<bool, io::Error> {
        match fd.read_to_end(dst) {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.raw_os_error() == Some(rustix::io::Error::WOULDBLOCK.raw_os_error())
                    || e.raw_os_error() == Some(rustix::io::Error::AGAIN.raw_os_error())
                {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
        }
    }
}

impl AsRawFd for AnonPipe {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl AsFd for AnonPipe {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl IntoRawFd for AnonPipe {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl FromRawFd for AnonPipe {
    unsafe fn from_raw_fd(raw_fd: RawFd) -> Self {
        Self(FromRawFd::from_raw_fd(raw_fd))
    }
}
