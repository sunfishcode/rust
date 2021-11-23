use crate::ffi::{c_void, ZStr};
use crate::{ascii, fmt, iter, mem};

use crate::net::c as libc;

fn sun_path_offset(addr: &libc::sockaddr_un) -> usize {
    // Work with an actual instance of the type since using a null pointer is UB
    let base = addr as *const _ as usize;
    let path = &addr.sun_path as *const _ as usize;
    path - base
}

pub(super) fn sockaddr_un(path: &ZStr) -> Result<SocketAddr, &'static &'static str> {
    let mut addr: libc::sockaddr_un = unsafe { mem::zeroed() };
    addr.sun_family = libc::AF_UNIX as libc::sa_family_t;

    let bytes = path.to_bytes();

    if bytes.contains(&0) {
        return Err(&"paths must not contain interior null bytes");
    }

    if bytes.len() >= addr.sun_path.len() {
        return Err(&"path must be shorter than SUN_LEN");
    }
    for (dst, src) in iter::zip(&mut addr.sun_path, bytes) {
        *dst = *src as u8;
    }
    // null byte for pathname addresses is already there because we zeroed the
    // struct

    let mut len = sun_path_offset(&addr) + bytes.len();
    match bytes.get(0) {
        Some(&0) | None => {}
        Some(_) => len += 1,
    }
    SocketAddr::from_parts(addr, len as libc::socklen_t)
}

enum AddressKind<'a> {
    Unnamed,
    Pathname(&'a ZStr),
    Abstract(&'a [u8]),
}

struct AsciiEscaped<'a>(&'a [u8]);

impl<'a> fmt::Display for AsciiEscaped<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "\"")?;
        for byte in self.0.iter().cloned().flat_map(ascii::escape_default) {
            write!(fmt, "{}", byte as char)?;
        }
        write!(fmt, "\"")
    }
}

/// An address associated with a Unix socket.
///
/// # Examples
///
/// ```
/// use std::os::unix::net::UnixListener;
///
/// let socket = match UnixListener::bind("/tmp/sock") {
///     Ok(sock) => sock,
///     Err(e) => {
///         println!("Couldn't bind: {:?}", e);
///         return
///     }
/// };
/// let addr = socket.local_addr().expect("Couldn't get local address");
/// ```
#[derive(Clone)]
#[stable(feature = "unix_socket", since = "1.10.0")]
pub struct SocketAddr {
    pub(super) addr: libc::sockaddr_un,
    pub(super) len: libc::socklen_t,
}

impl SocketAddr {
    /// Constructs a `SocketAddr` representing the given path.
    #[unstable(feature = "rustix", issue = "none")]
    #[inline]
    pub fn from_path(path: &ZStr) -> Result<Self, &'static &'static str> {
        sockaddr_un(path)
    }

    pub(super) fn from_parts(
        addr: libc::sockaddr_un,
        mut len: libc::socklen_t,
    ) -> Result<SocketAddr, &'static &'static str> {
        if len == 0 {
            // When there is a datagram from unnamed unix socket
            // linux returns zero bytes of address
            len = sun_path_offset(&addr) as libc::socklen_t; // i.e., zero-length address
        } else if addr.sun_family != libc::AF_UNIX as libc::sa_family_t {
            return Err(&"file descriptor did not correspond to a Unix socket");
        }

        Ok(SocketAddr { addr, len })
    }

    /// Returns `true` if the address is unnamed.
    ///
    /// # Examples
    ///
    /// A named address:
    ///
    /// ```no_run
    /// use std::os::unix::net::UnixListener;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let socket = UnixListener::bind("/tmp/sock")?;
    ///     let addr = socket.local_addr().expect("Couldn't get local address");
    ///     assert_eq!(addr.is_unnamed(), false);
    ///     Ok(())
    /// }
    /// ```
    ///
    /// An unnamed address:
    ///
    /// ```
    /// use std::os::unix::net::UnixDatagram;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let socket = UnixDatagram::unbound()?;
    ///     let addr = socket.local_addr().expect("Couldn't get local address");
    ///     assert_eq!(addr.is_unnamed(), true);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn is_unnamed(&self) -> bool {
        matches!(self.address(), AddressKind::Unnamed)
    }

    /// Returns the contents of this address if it is a `pathname` address.
    ///
    /// # Examples
    ///
    /// With a pathname:
    ///
    /// ```no_run
    /// use std::os::unix::net::UnixListener;
    /// use core::ffi::ZStr;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let socket = UnixListener::bind("/tmp/sock")?;
    ///     let addr = socket.local_addr().expect("Couldn't get local address");
    ///     assert_eq!(addr.as_pathname(), Some(ZStr::new("/tmp/sock")));
    ///     Ok(())
    /// }
    /// ```
    ///
    /// Without a pathname:
    ///
    /// ```
    /// use std::os::unix::net::UnixDatagram;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let socket = UnixDatagram::unbound()?;
    ///     let addr = socket.local_addr().expect("Couldn't get local address");
    ///     assert_eq!(addr.as_pathname(), None);
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "unix_socket", since = "1.10.0")]
    #[must_use]
    pub fn as_pathname(&self) -> Option<&ZStr> {
        if let AddressKind::Pathname(path) = self.address() { Some(path) } else { None }
    }

    /// Returns the contents of this address if it is an abstract namespace
    /// without the leading null byte.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// #![feature(unix_socket_abstract)]
    /// use std::os::unix::net::{UnixListener, SocketAddr};
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let namespace = b"hidden";
    ///     let namespace_addr = SocketAddr::from_abstract_namespace(&namespace[..])?;
    ///     let socket = UnixListener::bind_addr(&namespace_addr)?;
    ///     let local_addr = socket.local_addr().expect("Couldn't get local address");
    ///     assert_eq!(local_addr.as_abstract_namespace(), Some(&namespace[..]));
    ///     Ok(())
    /// }
    /// ```
    #[doc(cfg(any(target_os = "android", target_os = "linux")))]
    #[cfg(any(doc, target_os = "android", target_os = "linux",))]
    #[unstable(feature = "unix_socket_abstract", issue = "85410")]
    pub fn as_abstract_namespace(&self) -> Option<&[u8]> {
        if let AddressKind::Abstract(name) = self.address() { Some(name) } else { None }
    }

    fn address(&self) -> AddressKind<'_> {
        let len = self.len as usize - sun_path_offset(&self.addr);
        let path = unsafe { mem::transmute::<&[_], &[u8]>(&self.addr.sun_path) };

        // macOS seems to return a len of 16 and a zeroed sun_path for unnamed addresses
        if len == 0
            || (cfg!(not(any(target_os = "linux", target_os = "android")))
                && self.addr.sun_path[0] == 0)
        {
            AddressKind::Unnamed
        } else if self.addr.sun_path[0] == 0 {
            AddressKind::Abstract(&path[1..len])
        } else {
            AddressKind::Pathname(unsafe { ZStr::from_bytes_with_nul_unchecked(&path[..len]) })
        }
    }

    /// Creates an abstract domain socket address from a namespace
    ///
    /// An abstract address does not create a file unlike traditional path-based
    /// Unix sockets. The advantage of this is that the address will disappear when
    /// the socket bound to it is closed, so no filesystem clean up is required.
    ///
    /// The leading null byte for the abstract namespace is automatically added.
    ///
    /// This is a Linux-specific extension. See more at [`unix(7)`].
    ///
    /// [`unix(7)`]: https://man7.org/linux/man-pages/man7/unix.7.html
    ///
    /// # Errors
    ///
    /// This will return an error if the given namespace is too long
    ///
    /// # Examples
    ///
    /// ```no_run
    /// #![feature(unix_socket_abstract)]
    /// use std::os::unix::net::{UnixListener, SocketAddr};
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let addr = SocketAddr::from_abstract_namespace(b"hidden")?;
    ///     let listener = match UnixListener::bind_addr(&addr) {
    ///         Ok(sock) => sock,
    ///         Err(err) => {
    ///             println!("Couldn't bind: {:?}", err);
    ///             return Err(err);
    ///         }
    ///     };
    ///     Ok(())
    /// }
    /// ```
    #[doc(cfg(any(target_os = "android", target_os = "linux")))]
    #[cfg(any(doc, target_os = "android", target_os = "linux",))]
    #[unstable(feature = "unix_socket_abstract", issue = "85410")]
    pub fn from_abstract_namespace(namespace: &[u8]) -> Result<SocketAddr, &'static &'static str> {
        unsafe {
            let mut addr: libc::sockaddr_un = mem::zeroed();
            addr.sun_family = libc::AF_UNIX as libc::sa_family_t;

            if namespace.len() + 1 > addr.sun_path.len() {
                return Err(&"namespace must be shorter than SUN_LEN");
            }

            crate::ptr::copy_nonoverlapping(
                namespace.as_ptr(),
                addr.sun_path.as_mut_ptr().offset(1) as *mut u8,
                namespace.len(),
            );
            let len = (sun_path_offset(&addr) + 1 + namespace.len()) as libc::socklen_t;
            SocketAddr::from_parts(addr, len)
        }
    }
}

#[stable(feature = "unix_socket", since = "1.10.0")]
impl fmt::Debug for SocketAddr {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.address() {
            AddressKind::Unnamed => write!(fmt, "(unnamed)"),
            AddressKind::Abstract(name) => write!(fmt, "{} (abstract)", AsciiEscaped(name)),
            AddressKind::Pathname(path) => write!(fmt, "{:?} (pathname)", path),
        }
    }
}

/// Access to the platform-specific parts of of `SocketAddr`.
#[unstable(feature = "rustix", issue = "none")]
pub trait SocketAddrExt {
    /// Construct a `SocketAddr` from raw parts.
    #[unstable(feature = "rustix", issue = "none")]
    unsafe fn from_raw(
        raw: *const c_void,
        len: libc::socklen_t,
    ) -> Result<Self, &'static &'static str>
    where
        Self: Sized;

    /// View the raw parts of a `SocketAddr`.
    #[unstable(feature = "rustix", issue = "none")]
    fn as_raw(&self) -> (*const c_void, libc::socklen_t);
}

#[unstable(feature = "rustix", issue = "none")]
impl SocketAddrExt for SocketAddr {
    #[inline]
    unsafe fn from_raw(
        raw: *const c_void,
        len: libc::socklen_t,
    ) -> Result<Self, &'static &'static str> {
        SocketAddr::from_parts(unsafe { raw.cast::<libc::sockaddr_un>().read() }, len)
    }

    #[inline]
    fn as_raw(&self) -> (*const c_void, libc::socklen_t) {
        let ptr: *const _ = &self.addr;
        (ptr.cast(), self.len)
    }
}
