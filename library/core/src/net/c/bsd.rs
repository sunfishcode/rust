#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sockaddr {
    pub sa_len: u8,
    pub sa_family: sa_family_t,
    pub sa_data: [u8; 14],
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct sockaddr_in {
    pub sin_len: u8,
    pub sin_family: sa_family_t,
    pub sin_port: u16,
    pub sin_addr: super::in_addr,
    pub sin_zero: [c_char; 8],
}

#[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
pub struct sockaddr_in {
    pub sin_len: u8,
    pub sin_family: sa_family_t,
    pub sin_port: u16,
    pub sin_addr: super::in_addr,
    pub sin_zero: [i8; 8],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sockaddr_in6 {
    pub sin6_len: u8,
    pub sin6_family: sa_family_t,
    pub sin6_port: u16,
    pub sin6_flowinfo: u32,
    pub sin6_addr: super::in6_addr,
    pub sin6_scope_id: u32,
}

#[repr(C)]
pub struct sockaddr_un {
    pub sun_len: u8,
    pub sun_family: sa_family_t,
    pub sun_path: [c_char; 104],
}

pub type socklen_t = u32;
pub type sa_family_t = u8;

pub const AF_INET: sa_family_t = 2;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub const AF_INET6: sa_family_t = 30;
#[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
pub const AF_INET6: sa_family_t = 28;
#[cfg(any(target_os = "openbsd", target_os = "netbsd"))]
pub const AF_INET6: sa_family_t = 24;
