#![no_std]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogRecord {
    pub timestamp: u64,
    pub level: Level,
    pub subsystem: u16,
    pub code: u32,
    pub value: u64,
}

/// Bounded diagnostic ring. Logging must never allocate or block the kernel.
pub struct LogRing<const N: usize> {
    entries: [Option<LogRecord>; N],
    next: usize,
    len: usize,
    dropped: u64,
}

impl<const N: usize> LogRing<N> {
    pub const fn new() -> Self {
        Self {
            entries: [None; N],
            next: 0,
            len: 0,
            dropped: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }
    pub const fn dropped(&self) -> u64 {
        self.dropped
    }

    pub fn push(&mut self, record: LogRecord) {
        if N == 0 {
            self.dropped = self.dropped.saturating_add(1);
            return;
        }
        if self.len == N {
            self.dropped = self.dropped.saturating_add(1);
        }
        self.entries[self.next] = Some(record);
        self.next = (self.next + 1) % N;
        if self.len < N {
            self.len += 1;
        }
    }

    /// Returns records in chronological order into a caller-provided buffer.
    pub fn snapshot(&self, out: &mut [Option<LogRecord>]) -> usize {
        let count = core::cmp::min(out.len(), self.len);
        if count == 0 {
            return 0;
        }
        let start = if self.len == N { self.next } else { 0 };
        let mut i = 0;
        while i < count {
            out[i] = self.entries[(start + i) % N];
            i += 1;
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_is_bounded_and_ordered() {
        let mut log: LogRing<2> = LogRing::new();
        let a = LogRecord {
            timestamp: 1,
            level: Level::Info,
            subsystem: 1,
            code: 10,
            value: 1,
        };
        let b = LogRecord {
            timestamp: 2,
            level: Level::Warn,
            subsystem: 1,
            code: 11,
            value: 2,
        };
        let c = LogRecord {
            timestamp: 3,
            level: Level::Error,
            subsystem: 1,
            code: 12,
            value: 3,
        };
        log.push(a);
        log.push(b);
        log.push(c);
        let mut out = [None; 2];
        assert_eq!(log.snapshot(&mut out), 2);
        assert_eq!(out[0], Some(b));
        assert_eq!(out[1], Some(c));
        assert_eq!(log.dropped(), 1);
    }
}
