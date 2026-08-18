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

pub struct ProvenanceLog<const N: usize> {
    events: [Option<ProvenanceEvent>; N],
    next: usize,
    count: usize,
    sequence: u64,
}
impl<const N: usize> Default for ProvenanceLog<N> { fn default() -> Self { Self::new() } }
impl<const N: usize> ProvenanceLog<N> {
    pub const fn new() -> Self { Self { events: [None; N], next: 0, count: 0, sequence: 0 } }
    pub fn record(&mut self, mut event: ProvenanceEvent) -> Option<ProvenanceId> {
        if N == 0 { return None; }
        self.sequence = self.sequence.wrapping_add(1);
        event.id = ProvenanceId(self.sequence);
        self.events[self.next] = Some(event);
        self.next = (self.next + 1) % N;
        if self.count < N { self.count += 1; }
        Some(event.id)
    }
    pub const fn len(&self) -> usize { self.count }
    pub const fn capacity(&self) -> usize { N }
    pub const fn is_empty(&self) -> bool { self.count == 0 }
    pub fn latest(&self) -> Option<ProvenanceEvent> {
        if self.count == 0 || N == 0 { return None; }
        let index = if self.next == 0 { N - 1 } else { self.next - 1 };
        self.events[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event() -> ProvenanceEvent { ProvenanceEvent { id: ProvenanceId(0), parent: ProvenanceId(0), process: 1, capability: CapabilityId(2), operation: 3, resource: 4, result: 0 } }
    #[test]
    fn journal_is_bounded_and_monotonic() {
        let mut log: ProvenanceLog<2> = ProvenanceLog::new();
        assert_eq!(log.record(event()).unwrap().0, 1);
        assert_eq!(log.record(event()).unwrap().0, 2);
        assert_eq!(log.record(event()).unwrap().0, 3);
        assert_eq!(log.len(), 2);
        assert_eq!(log.latest().unwrap().id.0, 3);
    }
    #[test]
    fn zero_capacity_fails_closed_without_modulo_or_index_panic() {
        let mut log: ProvenanceLog<0> = ProvenanceLog::new();
        assert!(log.record(event()).is_none());
        assert!(log.latest().is_none());
        assert!(log.is_empty());
    }
}
