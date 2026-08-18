#![no_std]

use super::{VirtioDescriptor, VirtioError, VirtioSplitQueue, VirtioTransportState};

pub const SECTOR_SIZE: u64 = 512;
pub const MAX_REQUEST_SECTORS: u32 = 128;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockOp { Read, Write, Flush }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BlockRequest {
    pub op: BlockOp,
    pub sector: u64,
    pub sectors: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockError {
    ZeroLength,
    TooLarge,
    OutOfRange,
    ArithmeticOverflow,
    Unsupported,
    Queue(VirtioError),
}

impl From<VirtioError> for BlockError {
    fn from(value: VirtioError) -> Self { Self::Queue(value) }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VirtioBlockConfig {
    pub capacity_sectors: u64,
    pub logical_block_size: u32,
}

impl VirtioBlockConfig {
    pub const fn validate(&self) -> Result<(), BlockError> {
        if self.capacity_sectors == 0 || self.logical_block_size == 0 {
            return Err(BlockError::ZeroLength);
        }
        if self.logical_block_size != SECTOR_SIZE as u32 {
            return Err(BlockError::Unsupported);
        }
        Ok(())
    }

    pub const fn validate_request(&self, request: BlockRequest) -> Result<u64, BlockError> {
        self.validate()?;
        if request.sectors == 0 { return Err(BlockError::ZeroLength); }
        if request.sectors > MAX_REQUEST_SECTORS { return Err(BlockError::TooLarge); }
        let end = match request.sector.checked_add(request.sectors as u64) {
            Some(value) => value,
            None => return Err(BlockError::ArithmeticOverflow),
        };
        if end > self.capacity_sectors { return Err(BlockError::OutOfRange); }
        request.sectors.checked_mul(SECTOR_SIZE as u32).map(|v| v as u64).ok_or(BlockError::ArithmeticOverflow)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BlockCompletion { pub request_id: u16, pub status: u8, pub bytes: u32 }

pub struct VirtioBlockQueue<const N: usize> {
    queue: VirtioSplitQueue<N>,
}

impl<const N: usize> VirtioBlockQueue<N> {
    pub const fn new() -> Self { Self { queue: VirtioSplitQueue::new() } }

    pub fn submit<const DMA_BITS: u8>(
        &mut self,
        request_id: u16,
        descriptor: VirtioDescriptor,
        config: VirtioBlockConfig,
        transport: &mut VirtioTransportState,
        mmio: &mut super::VirtioMmioRegisters,
    ) -> Result<(), BlockError> {
        if request_id as usize >= N { return Err(BlockError::Queue(VirtioError::InvalidQueueIndex)); }
        config.validate()?;
        self.queue.submit_and_notify_checked(
            request_id,
            descriptor,
            DMA_BITS,
            (MAX_REQUEST_SECTORS * SECTOR_SIZE as u32).saturating_add(16),
            transport,
            mmio,
        )?;
        Ok(())
    }

    pub fn complete(&mut self, request_id: u16, bytes: u32, mmio: &mut super::VirtioMmioRegisters) -> Result<(), BlockError> {
        self.queue.complete(request_id, bytes, mmio)?;
        Ok(())
    }

    pub fn poll_completion(&mut self) -> Result<Option<BlockCompletion>, BlockError> {
        Ok(self.queue.pop_completion()?.map(|entry| BlockCompletion { request_id: entry.id, status: 0, bytes: entry.len }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::{VirtioFeatures, VirtioMmioRegisters};

    const CFG: VirtioBlockConfig = VirtioBlockConfig { capacity_sectors: 4096, logical_block_size: 512 };

    #[test]
    fn accepts_bounded_request() {
        assert_eq!(CFG.validate_request(BlockRequest { op: BlockOp::Read, sector: 8, sectors: 2 }), Ok(1024));
    }

    #[test]
    fn rejects_end_overflow_and_capacity_escape() {
        assert_eq!(CFG.validate_request(BlockRequest { op: BlockOp::Read, sector: u64::MAX, sectors: 1 }), Err(BlockError::ArithmeticOverflow));
        assert_eq!(CFG.validate_request(BlockRequest { op: BlockOp::Read, sector: 4095, sectors: 2 }), Err(BlockError::OutOfRange));
    }

    #[test]
    fn queue_submission_is_transport_bound() {
        let mut transport = super::super::VirtioTransportState::new(VirtioFeatures::VERSION_1);
        let mut mmio = VirtioMmioRegisters::new(2, 1, VirtioFeatures::VERSION_1);
        transport.acknowledge().unwrap();
        transport.set_driver().unwrap();
        transport.negotiate(VirtioFeatures::VERSION_1).unwrap();
        transport.configure_queues(1).unwrap();
        transport.driver_ok().unwrap();
        let mut queue: VirtioBlockQueue<1> = VirtioBlockQueue::new();
        queue.submit::<32>(0, VirtioDescriptor { addr: 0x1000, len: 528, flags: 0, next: 0 }, CFG, &mut transport, &mut mmio).unwrap();
        assert_eq!(mmio.queue_notify, 0);
        queue.complete(0, 512, &mut mmio).unwrap();
        assert_eq!(queue.poll_completion().unwrap(), Some(BlockCompletion { request_id: 0, status: 0, bytes: 512 }));
    }
}
