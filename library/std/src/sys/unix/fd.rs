#![unstable(reason = "not public", issue = "none", feature = "fd")]

#[cfg(test)]
mod tests;

use crate::cmp;
use crate::io::{self, IoSlice, IoSliceMut, Read, ReadBuf};
use crate::os::unix::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, OwnedFd, RawFd};
use crate::sys::cvt;
use crate::sys_common::{AsInner, FromInner, IntoInner};

use rustix::fs::FdFlags;
#[cfg(not(target_os = "linux"))]
use rustix::fs::OFlags;

#[derive(Debug)]
pub struct FileDesc(OwnedFd);

// The maximum read limit on most POSIX-like systems is `SSIZE_MAX`,
// with the man page quoting that if the count of bytes to read is
// greater than `SSIZE_MAX` the result is "unspecified".
//
// On macOS, however, apparently the 64-bit libc is either buggy or
// intentionally showing odd behavior by rejecting any read with a size
// larger than or equal to INT_MAX. To handle both of these the read
// size is capped on both platforms.
#[cfg(target_os = "macos")]
const READ_LIMIT: usize = libc::c_int::MAX as usize - 1;
#[cfg(not(target_os = "macos"))]
const READ_LIMIT: usize = libc::ssize_t::MAX as usize;

impl FileDesc {
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = rustix::io::read(self, buf)?;
        Ok(ret)
    }

    #[cfg(not(any(target_os = "espidf", target_os = "horizon")))]
    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let ret = rustix::io::readv(self, unsafe { crate::mem::transmute(bufs) })?;
        Ok(ret)
    }

    #[cfg(any(target_os = "espidf", target_os = "horizon"))]
    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        return crate::io::default_read_vectored(|b| self.read(b), unsafe {
            crate::mem::transmute(bufs)
        });
    }

    #[inline]
    pub fn is_read_vectored(&self) -> bool {
        cfg!(not(any(target_os = "espidf", target_os = "horizon")))
    }

    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut me = self;
        (&mut me).read_to_end(buf)
    }

    pub fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        let ret = rustix::io::pread(self, buf, offset)?;
        Ok(ret)
    }

    pub fn read_buf(&self, buf: &mut ReadBuf<'_>) -> io::Result<()> {
        let ret = cvt(unsafe {
            libc::read(
                self.as_raw_fd(),
                buf.unfilled_mut().as_mut_ptr() as *mut libc::c_void,
                cmp::min(buf.remaining(), READ_LIMIT),
            )
        })?;

        // Safety: `ret` bytes were written to the initialized portion of the buffer
        unsafe {
            buf.assume_init(ret as usize);
        }
        buf.add_filled(ret as usize);
        Ok(())
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let ret = rustix::io::write(self, buf)?;
        Ok(ret)
    }

    #[cfg(not(any(target_os = "espidf", target_os = "horizon")))]
    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let ret = rustix::io::writev(self, unsafe { crate::mem::transmute(bufs) })?;
        Ok(ret)
    }

    #[cfg(any(target_os = "espidf", target_os = "horizon"))]
    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        return crate::io::default_write_vectored(|b| self.write(b), unsafe {
            crate::mem::transmute(bufs)
        });
    }

    #[inline]
    pub fn is_write_vectored(&self) -> bool {
        cfg!(not(any(target_os = "espidf", target_os = "horizon")))
    }

    pub fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        let ret = rustix::io::pwrite(self, buf, offset)?;
        Ok(ret)
    }

    #[cfg(target_os = "linux")]
    pub fn get_cloexec(&self) -> io::Result<bool> {
        Ok(rustix::fs::fcntl_getfd(self)?.contains(FdFlags::CLOEXEC))
    }

    #[cfg(not(any(
        target_env = "newlib",
        target_os = "solaris",
        target_os = "illumos",
        target_os = "emscripten",
        target_os = "fuchsia",
        target_os = "l4re",
        target_os = "linux",
        target_os = "haiku",
        target_os = "redox",
        target_os = "vxworks"
    )))]
    pub fn set_cloexec(&self) -> io::Result<()> {
        rustix::io::ioctl_fioclex(self)?;
        Ok(())
    }
    #[cfg(any(
        all(target_env = "newlib", not(any(target_os = "espidf", target_os = "horizon"))),
        target_os = "solaris",
        target_os = "illumos",
        target_os = "emscripten",
        target_os = "fuchsia",
        target_os = "l4re",
        target_os = "linux",
        target_os = "haiku",
        target_os = "redox",
        target_os = "vxworks"
    ))]
    pub fn set_cloexec(&self) -> io::Result<()> {
        let previous = rustix::fs::fcntl_getfd(self)?;
        let new = previous | FdFlags::CLOEXEC;
        if new != previous {
            rustix::fs::fcntl_setfd(self, new)?;
        }
        Ok(())
    }
    #[cfg(any(target_os = "espidf", target_os = "horizon"))]
    pub fn set_cloexec(&self) -> io::Result<()> {
        // FD_CLOEXEC is not supported in ESP-IDF and Horizon OS but there's no need to,
        // because neither supports spawning processes.
        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        rustix::io::ioctl_fionbio(self, nonblocking)?;
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        unsafe {
            let previous = rustix::fs::fcntl_getfl(self)?;
            let new = if nonblocking {
                previous | OFlags::NONBLOCK;
            } else {
                previous & !OFlags::NONBLOCK;
            };
            if new != previous {
                rustix::fcntl_setfl(self, new)?;
            }
            Ok(())
        }
    }

    #[inline]
    pub fn duplicate(&self) -> io::Result<FileDesc> {
        Ok(Self(self.0.try_clone()?))
    }
}

impl<'a> Read for &'a FileDesc {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }
}

impl AsInner<OwnedFd> for FileDesc {
    fn as_inner(&self) -> &OwnedFd {
        &self.0
    }
}

impl IntoInner<OwnedFd> for FileDesc {
    fn into_inner(self) -> OwnedFd {
        self.0
    }
}

impl FromInner<OwnedFd> for FileDesc {
    fn from_inner(owned_fd: OwnedFd) -> Self {
        Self(owned_fd)
    }
}

impl AsFd for FileDesc {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for FileDesc {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl IntoRawFd for FileDesc {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl FromRawFd for FileDesc {
    unsafe fn from_raw_fd(raw_fd: RawFd) -> Self {
        Self(FromRawFd::from_raw_fd(raw_fd))
    }
}
