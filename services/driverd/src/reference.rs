#![no_std]

//! Reference VirtIO device protocols. These are userspace driver contracts and
//! request formats; actual MMIO/DMA backend execution stays in driverd.

use crate::virtio::{VirtioKind, VirtioQueue};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockRequest {
    pub sector: u64,
    pub sectors: u32,
    pub write: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetFrame<'a> {
    pub bytes: &'a [u8],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputEvent {
    pub event_type: u16,
    pub code: u16,
    pub value: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub stride: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferenceDevice {
    pub kind: VirtioKind,
    pub queue: VirtioQueue,
}

impl ReferenceDevice {
    pub const fn block(queue: VirtioQueue) -> Self { Self { kind: VirtioKind::Block, queue } }
    pub const fn network(queue: VirtioQueue) -> Self { Self { kind: VirtioKind::Network, queue } }
    pub const fn input(queue: VirtioQueue) -> Self { Self { kind: VirtioKind::Input, queue } }
    pub const fn gpu(queue: VirtioQueue) -> Self { Self { kind: VirtioKind::Gpu, queue } }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceError {
    ZeroLength,
    InvalidRange,
}

pub const fn validate_block_request(request: BlockRequest, capacity_sectors: u64) -> Result<(), ReferenceError> {
    if request.sectors == 0 { return Err(ReferenceError::ZeroLength); }
    let end = request.sector.saturating_add(request.sectors as u64);
    if end > capacity_sectors { return Err(ReferenceError::InvalidRange); }
    Ok(())
}

pub const fn validate_display_rect(rect: DisplayRect, width: u16, height: u16) -> Result<(), ReferenceError> {
    if rect.width == 0 || rect.height == 0 { return Err(ReferenceError::ZeroLength); }
    let x_end = rect.x.saturating_add(rect.width);
    let y_end = rect.y.saturating_add(rect.height);
    if x_end > width || y_end > height { return Err(ReferenceError::InvalidRange); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::virtio::VirtioQueue;

    #[test]
    fn block_request_is_bounds_checked() {
        assert!(validate_block_request(BlockRequest { sector: 10, sectors: 2, write: false }, 20).is_ok());
        assert_eq!(validate_block_request(BlockRequest { sector: 19, sectors: 2, write: false }, 20), Err(ReferenceError::InvalidRange));
    }

    #[test]
    fn display_rect_is_bounds_checked() {
        let rect = DisplayRect { x: 10, y: 10, width: 20, height: 20, stride: 80 };
        assert!(validate_display_rect(rect, 100, 100).is_ok());
        assert_eq!(validate_display_rect(rect, 20, 20), Err(ReferenceError::InvalidRange));
    }

    #[test]
    fn reference_devices_are_typed() {
        let q = VirtioQueue::new(128).unwrap();
        assert_eq!(ReferenceDevice::block(q).kind, VirtioKind::Block);
        assert_eq!(ReferenceDevice::network(q).kind, VirtioKind::Network);
        assert_eq!(ReferenceDevice::input(q).kind, VirtioKind::Input);
        assert_eq!(ReferenceDevice::gpu(q).kind, VirtioKind::Gpu);
    }
}
