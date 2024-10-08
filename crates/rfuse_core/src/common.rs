pub const BLOCK_SIZE: u32 = 4096;
pub const DEFAULT_HARD_LINKS: u32 = 1;
pub const RDEV: u32 = 0;
pub const FLAGS: u32 = 0;
pub const DEFAULT_PERMISSIONS: u16 = 600;
pub const MAX_NAME_LENGTH: u32 = 255;
pub const FMODE_EXEC: i32 = 0x20;

#[cfg(target_os = "linux")]
pub const RFUSE_S_ISVTX: u16 = libc::S_ISVTX as u16;
#[cfg(target_os = "macos")]
pub const RFUSE_S_ISVTX: u16 = libc::S_ISVTX;
