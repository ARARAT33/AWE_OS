#![no_std]

use super::capability::CapabilityId;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ProvenanceId(pub u64);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ProvenanceEvent {
    pub id: ProvenanceId,
    pub parent: ProvenanceId,
    pub process: u64,
    pub capability: CapabilityId,
    pub operation: u32,
    pub resource: u64,
    pub result: i64,
}

/// Fixed-size, allocation-free provenance journal suitable for the trusted
/// path. Overflow intentionally drops the oldest event rather than blocking
/// the kernel or allocating memory.
pub struct ProvenanceLog<const N: usize> {
    events: [Option<ProvenanceEvent>; N],
    next: usize,
    count: usize,
    sequence: u64,
}

impl<const N: usize> ProvenanceLog<N> {
    pub const fn new() -> Self {
        Self { events: [None; N], next: 0, count: 0, sequence: 0 }
    }

    pub fn record(&mut self, mut event: ProvenanceEvent) -> ProvenanceId {
        self.sequence = self.sequence.wrapping_add(1);
        event.id = ProvenanceId(self.sequence);
        self.events[self.next] = Some(event);
        self.next = (self.next + 1) % N;
        if self.count < N { self.count += 1; }
        event.id
    }

    pub const fn len(&self) -> usize { self.count }
    pub const fn is_empty(&self) -> bool { self.count == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn journal_is_bounded_and_monotonic() {
        let mut log: ProvenanceLog<2> = ProvenanceLog::new();
        let base = ProvenanceEvent { id: ProvenanceId(0), parent: ProvenanceId(0), process: 1, capability: CapabilityId(2), operation: 3, resource: 4, result: 0 };
        assert_eq!(log.record(base).0, 1);
        assert_eq!(log.record(base).0, 2);
        assert_eq!(log.record(base).0, 3);
        assert_eq!(log.len(), 2);
    }
}
