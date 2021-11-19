use crate::borrow::Cow;
use crate::ffi::{OsStr, OsString};
use crate::mem;
use crate::sealed::Sealed;
use crate::sys::os_str::Buf;
use crate::sys_common::{AsInner, FromInner, IntoInner};

// Note: this file is currently reused in other `std::os::{platform}::ffi` modules to reduce duplication.
// Keep this in mind when applying changes to this file that only apply to `unix`.

/// Platform-specific extensions to [`OsString`].
///
/// This trait is sealed: it cannot be implemented outside the standard library.
/// This is so that future additional methods are not breaking changes.
#[stable(feature = "rust1", since = "1.0.0")]
pub trait OsStringExt: Sealed {
    /// Creates an [`OsString`] from a byte vector.
    ///
    /// See the module documentation for an example.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn from_vec(vec: Vec<u8>) -> Self;

    /// Yields the underlying byte vector of this [`OsString`].
    ///
    /// See the module documentation for an example.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn into_vec(self) -> Vec<u8>;
}

#[stable(feature = "rust1", since = "1.0.0")]
impl OsStringExt for OsString {
    #[inline]
    fn from_vec(vec: Vec<u8>) -> OsString {
        FromInner::from_inner(Buf { inner: vec })
    }
    #[inline]
    fn into_vec(self) -> Vec<u8> {
        self.into_inner().inner
    }
}

/// Platform-specific extensions to [`OsStr`].
///
/// This trait is sealed: it cannot be implemented outside the standard library.
/// This is so that future additional methods are not breaking changes.
#[stable(feature = "rust1", since = "1.0.0")]
pub trait OsStrExt: Sealed {
    #[stable(feature = "rust1", since = "1.0.0")]
    /// Creates an [`OsStr`] from a byte slice.
    ///
    /// See the module documentation for an example.
    fn from_bytes(slice: &[u8]) -> &Self;

    /// Gets the underlying byte view of the [`OsStr`] slice.
    ///
    /// See the module documentation for an example.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn as_bytes(&self) -> &[u8];
}

#[stable(feature = "rust1", since = "1.0.0")]
impl OsStrExt for OsStr {
    #[inline]
    fn from_bytes(slice: &[u8]) -> &OsStr {
        unsafe { mem::transmute(slice) }
    }
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        &self.as_inner().inner
    }
}

#[unstable(feature = "rustix", issue = "none")]
impl rustix::path::Arg for &OsStr {
    #[inline]
    fn as_str(&self) -> rustix::io::Result<&str> {
        self.to_str().ok_or(rustix::io::Error::INVAL)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        OsStr::to_string_lossy(self)
    }

    #[inline]
    fn as_cow_z_str(&self) -> rustix::io::Result<Cow<'_, rustix::ffi::ZStr>> {
        Ok(Cow::Owned(
            rustix::ffi::ZString::new(self.as_bytes())
                .map_err(|_cstr_err| rustix::io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_z_str<'b>(self) -> rustix::io::Result<Cow<'b, rustix::ffi::ZStr>>
    where
        Self: 'b,
    {
        self.as_bytes().into_z_str()
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> rustix::io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&rustix::ffi::ZStr) -> rustix::io::Result<T>,
    {
        self.as_bytes().into_with_z_str(f)
    }
}

#[unstable(feature = "rustix", issue = "none")]
impl rustix::path::Arg for &crate::path::Path {
    #[inline]
    fn as_str(&self) -> rustix::io::Result<&str> {
        self.as_os_str().to_str().ok_or(rustix::io::Error::INVAL)
    }

    #[inline]
    fn to_string_lossy(&self) -> Cow<'_, str> {
        crate::path::Path::to_string_lossy(self)
    }

    #[inline]
    fn as_cow_z_str(&self) -> rustix::io::Result<Cow<'_, rustix::ffi::ZStr>> {
        Ok(Cow::Owned(
            rustix::ffi::ZString::new(self.as_os_str().as_bytes())
                .map_err(|_cstr_err| rustix::io::Error::INVAL)?,
        ))
    }

    #[inline]
    fn into_z_str<'b>(self) -> rustix::io::Result<Cow<'b, rustix::ffi::ZStr>>
    where
        Self: 'b,
    {
        self.as_os_str().into_z_str()
    }

    #[inline]
    fn into_with_z_str<T, F>(self, f: F) -> rustix::io::Result<T>
    where
        Self: Sized,
        F: FnOnce(&rustix::ffi::ZStr) -> rustix::io::Result<T>,
    {
        self.as_os_str().into_with_z_str(f)
    }
}
