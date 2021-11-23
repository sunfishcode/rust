#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sockaddr {
    pub sa_family: sa_family_t,
    pub sa_data: [u8; 14],
}

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

#[repr(C)]
#[derive(Clone, Debug)]
pub struct sockaddr_un {
    pub sun_family: sa_family_t,
    pub sun_path: [u8; 108],
}

#[repr(C)]
#[derive(Debug)]
pub struct sockaddr_storage {
    pub ss_family: sa_family_t,
    __ss_align: usize,
    #[cfg(target_pointer_width = "32")]
    __ss_pad2: [u8; 128 - 2 * 4],
    #[cfg(target_pointer_width = "64")]
    __ss_pad2: [u8; 128 - 2 * 8],
}

#[cfg(not(all(target_os = "android", target_pointer_width = "32")))]
pub type socklen_t = u32;
#[cfg(all(target_os = "android", target_pointer_width = "32"))]
pub type socklen_t = i32;
pub type sa_family_t = u16;

pub const AF_UNIX: sa_family_t = 1;
pub const AF_INET: sa_family_t = 2;
pub const AF_INET6: sa_family_t = 10;
