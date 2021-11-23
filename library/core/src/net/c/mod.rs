#![allow(non_camel_case_types)]
#![unstable(feature = "rustix", issue = "none")]

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct in_addr {
    pub s_addr: u32,
}

#[repr(C)]
#[cfg_attr(all(unix, libc_align), repr(align(4)))]
#[derive(Debug, Copy, Clone)]
pub struct in6_addr {
    pub s6_addr: [u8; 16],

    #[cfg(all(unix, not(libc_align)))]
    pub __align: [u32; 0],
}

// FIXME: newlib and uclibc variants of `sockaddr_in` etc.
#[cfg_attr(windows, path = "aarch64.rs")]
#[cfg_attr(target_os = "solid_asp3", path = "solid.rs")]
#[cfg_attr(target_os = "hermit", path = "hermit.rs")]
#[cfg_attr(target_os = "haiku", path = "haiku.rs")]
#[cfg_attr(any(target_os = "android", target_os = "linux"), path = "linux_like.rs")]
#[cfg_attr(any(target_os = "illumos", target_os = "solaris"), path = "solarish.rs")]
#[cfg_attr(
    any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd"
    ),
    path = "bsd.rs"
)]
#[cfg_attr(target_os = "redox", path = "redox.rs")]
#[cfg_attr(target_os = "fuchsia", path = "fuchsia.rs")]
#[cfg_attr(target_os = "vxworks", path = "vxworks.rs")]
#[cfg_attr(
    not(any(
        windows,
        target_os = "solid_asp3",
        target_os = "hermit",
        target_os = "haiku",
        any(target_os = "android", target_os = "linux"),
        any(target_os = "illumos", target_os = "solaris"),
        any(
            target_os = "macos",
            target_os = "ios",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "netbsd"
        ),
        target_os = "redox",
        target_os = "fuchsia",
        target_os = "vxworks",
    )),
    path = "unsupported.rs"
)]
mod os;
pub(crate) use os::*;
