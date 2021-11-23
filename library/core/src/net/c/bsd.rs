#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sockaddr {
    pub sa_len: u8,
    pub sa_family: sa_family_t,
    pub sa_data: [u8; 14],
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

pub type sa_family_t = u8;

pub const AF_INET: sa_family_t = 2;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub const AF_INET6: sa_family_t = 30;
#[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
pub const AF_INET6: sa_family_t = 28;
#[cfg(any(target_os = "openbsd", target_os = "netbsd"))]
pub const AF_INET6: sa_family_t = 24;
