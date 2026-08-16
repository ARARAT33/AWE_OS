#![no_std]

pub mod address;
pub mod allocator;
pub mod frame;
pub mod mapper;
pub mod paging;
pub mod region;

pub use address::{PhysicalAddress, VirtualAddress};
pub use allocator::BumpAllocator;
pub use frame::{frame_at, frame_end, Frame, PAGE_SIZE};
pub use mapper::{MapError, PageMapper};
pub use paging::{indices as page_indices, page_offset, valid_mapping, PageFlags, PageTable, PageTableEntry};
pub use region::{Region, RegionKind};
