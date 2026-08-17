#![no_std]

pub mod address;
pub mod allocator;
pub mod boot_mapper;
pub mod frame;
pub mod mapper;
pub mod paging;
pub mod physical;
pub mod region;
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

pub use address::{PhysicalAddress, VirtualAddress};
pub use allocator::BumpAllocator;
pub use boot_mapper::{BootMapError, BootMapper};
pub use frame::{frame_at, frame_end, Frame, PhysicalFrameAllocator, PAGE_SIZE};
pub use mapper::{MapError, PageMapper};
pub use paging::{indices as page_indices, page_offset, valid_mapping, PageFlags, PageTable, PageTableEntry};
pub use physical::{FrameBitmap, PhysicalMemoryError};
#[cfg(target_arch = "x86_64")]
pub use x86_64::activate_bootstrap_identity_map;
pub use region::{Region, RegionKind};
