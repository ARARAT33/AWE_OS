#![no_std]

use super::frame::{Frame, PAGE_SIZE};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PhysicalMemoryError {
    OutOfMemory,
    InvalidFrame,
    DoubleFree,
}

pub struct FrameBitmap<'a> {
    bits: &'a mut [u64],
    base: u64,
    frame_count: usize,
    free: usize,
}

impl<'a> FrameBitmap<'a> {
    pub fn new(
        bits: &'a mut [u64],
        base: u64,
        frame_count: usize,
    ) -> Result<Self, PhysicalMemoryError> {
        if base & (PAGE_SIZE - 1) != 0 || frame_count > bits.len().saturating_mul(64) {
            return Err(PhysicalMemoryError::InvalidFrame);
        }
        for word in bits.iter_mut() {
            *word = u64::MAX;
        }
        Ok(Self {
            bits,
            base,
            frame_count,
            free: 0,
        })
    }

    fn index(&self, frame: Frame) -> Result<(usize, u64), PhysicalMemoryError> {
        if frame.start < self.base || frame.start & (PAGE_SIZE - 1) != 0 {
            return Err(PhysicalMemoryError::InvalidFrame);
        }
        let number = ((frame.start - self.base) / PAGE_SIZE) as usize;
        if number >= self.frame_count {
            return Err(PhysicalMemoryError::InvalidFrame);
        }
        Ok((number / 64, 1u64 << (number % 64)))
    }

    pub fn release(&mut self, frame: Frame) -> Result<(), PhysicalMemoryError> {
        let (word, mask) = self.index(frame)?;
        if self.bits[word] & mask == 0 {
            return Err(PhysicalMemoryError::DoubleFree);
        }
        self.bits[word] &= !mask;
        self.free += 1;
        Ok(())
    }

    pub fn allocate(&mut self) -> Result<Frame, PhysicalMemoryError> {
        for (word_index, word) in self.bits.iter_mut().enumerate() {
            let available = !*word;
            if available == 0 {
                continue;
            }
            let bit = available.trailing_zeros() as usize;
            let number = word_index * 64 + bit;
            if number >= self.frame_count {
                continue;
            }
            *word |= 1u64 << bit;
            self.free = self.free.saturating_sub(1);
            return Ok(Frame {
                start: self.base + number as u64 * PAGE_SIZE,
            });
        }
        Err(PhysicalMemoryError::OutOfMemory)
    }

    pub const fn free_frames(&self) -> usize {
        self.free
    }
    pub const fn total_frames(&self) -> usize {
        self.frame_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn release_allocate_and_double_free_are_deterministic() {
        let mut bits = [0u64; 1];
        let mut allocator = FrameBitmap::new(&mut bits, 0x1000, 64).unwrap();
        let frame = Frame { start: 0x1000 };
        allocator.release(frame).unwrap();
        assert_eq!(allocator.free_frames(), 1);
        assert_eq!(allocator.allocate().unwrap().start, 0x1000);
        assert_eq!(allocator.free_frames(), 0);
        allocator.release(frame).unwrap();
        assert_eq!(
            allocator.release(frame),
            Err(PhysicalMemoryError::DoubleFree)
        );
    }
    #[test]
    fn invalid_frame_is_rejected() {
        let mut bits = [0u64; 1];
        let mut allocator = FrameBitmap::new(&mut bits, 0x1000, 2).unwrap();
        assert_eq!(
            allocator.release(Frame { start: 0x3000 }),
            Err(PhysicalMemoryError::InvalidFrame)
        );
        assert_eq!(
            allocator.release(Frame { start: 0x1001 }),
            Err(PhysicalMemoryError::InvalidFrame)
        );
    }
}
