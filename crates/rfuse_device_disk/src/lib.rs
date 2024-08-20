#[cfg(feature = "local")]
pub mod local_disk;
#[cfg(feature = "mem")]
pub mod mem_disk;

pub mod utils;

pub enum DiskType {
    Local,
    Mem,
}
