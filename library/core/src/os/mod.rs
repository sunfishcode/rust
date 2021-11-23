//! Types with platform-specific representations.
//!
//! This module is for defining types, like `Ipv6Addr`, which have OS-specific
//! representations. It does not do any allocation, as that would belong in the
//! alloc library, and it does not perform any actual I/O, as that would belong
//! in the std library.

#![stable(feature = "rust1", since = "1.0.0")]

#[cfg(all(unix, not(target_os = "hermit")))]
pub mod unix;
