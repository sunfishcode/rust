//! Unix-specific networking functionality.

#![stable(feature = "unix_socket", since = "1.10.0")]

mod addr;

#[stable(feature = "unix_socket", since = "1.10.0")]
pub use self::addr::*;
