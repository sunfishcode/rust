#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sockaddr_in {
    pub sin_family: sa_family_t,
    pub sin_port: u16,
    pub sin_addr: super::in_addr,
    pub sin_zero: [u8; 8],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sockaddr_in6 {
    pub sin6_family: sa_family_t,
    pub sin6_port: u16,
    pub sin6_flowinfo: u32,
    pub sin6_addr: super::in6_addr,
    pub sin6_scope_id: u32,
}

pub type sa_family_t = u16;

pub const AF_INET: sa_family_t = 2;
pub const AF_INET6: sa_family_t = 10;
