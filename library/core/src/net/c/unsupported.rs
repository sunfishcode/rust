#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sockaddr_in {
    pub sin_family: sa_family_t,
    pub sin_port: u16,
    pub sin_addr: in_addr,
}

#[derive(Debug, Copy, Clone)]
pub struct sockaddr_in6 {
    pub sin6_family: sa_family_t,
    pub sin6_port: u16,
    pub sin6_addr: in6_addr,
    pub sin6_flowinfo: u32,
    pub sin6_scope_id: u32,
}

pub type sa_family_t = u8;

pub const AF_INET: u8 = 0;
pub const AF_INET6: u8 = 1;
