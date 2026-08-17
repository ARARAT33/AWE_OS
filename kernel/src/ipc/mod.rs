#![no_std]

/// A small, allocation-free IPC message used by the kernel boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IpcMessage {
    pub sender: u64,
    pub channel: u32,
    pub payload: [u64; 4],
}

impl IpcMessage {
    pub const fn new(sender: u64, channel: u32, payload: [u64; 4]) -> Self {
        Self { sender, channel, payload }
    }
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceChannel {
    Driverd = 1,
    Appd = 2,
    Asappd = 3,
    Ayuid = 4,
    Aweterminald = 5,
    Awebusd = 6,
    Aweupdated = 7,
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IpcOpcode {
    Hello = 1,
    Ping = 2,
    Start = 3,
    Stop = 4,
    Reset = 5,
    Query = 6,
    Event = 7,
    Handoff = 8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IpcEnvelope {
    pub service: ServiceChannel,
    pub opcode: IpcOpcode,
    pub request_id: u64,
    pub message: IpcMessage,
}

impl IpcEnvelope {
    pub const fn new(
        service: ServiceChannel,
        opcode: IpcOpcode,
        request_id: u64,
        message: IpcMessage,
    ) -> Self {
        Self { service, opcode, request_id, message }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IpcError {
    Full,
    Empty,
}

/// Deterministic bounded mailbox. No heap allocation and no unbounded growth.
pub struct Mailbox<const N: usize> {
    slots: [Option<IpcMessage>; N],
    head: usize,
    len: usize,
}

impl<const N: usize> Mailbox<N> {
    pub const fn new() -> Self {
        Self { slots: [None; N], head: 0, len: 0 }
    }

    pub const fn capacity(&self) -> usize { N }
    pub const fn len(&self) -> usize { self.len }
    pub const fn is_empty(&self) -> bool { self.len == 0 }
    pub const fn is_full(&self) -> bool { self.len == N }

    pub fn send(&mut self, message: IpcMessage) -> Result<(), IpcError> {
        if self.is_full() { return Err(IpcError::Full); }
        let index = (self.head + self.len) % N;
        self.slots[index] = Some(message);
        self.len += 1;
        Ok(())
    }

    pub fn recv(&mut self) -> Result<IpcMessage, IpcError> {
        if self.is_empty() { return Err(IpcError::Empty); }
        let message = self.slots[self.head].take().expect("mailbox invariant");
        self.head = (self.head + 1) % N;
        self.len -= 1;
        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_fifo_and_backpressure() {
        let mut q: Mailbox<2> = Mailbox::new();
        let a = IpcMessage::new(1, 10, [1, 0, 0, 0]);
        let b = IpcMessage::new(2, 10, [2, 0, 0, 0]);
        assert!(q.send(a).is_ok());
        assert!(q.send(b).is_ok());
        assert_eq!(q.send(a), Err(IpcError::Full));
        assert_eq!(q.recv(), Ok(a));
        assert_eq!(q.recv(), Ok(b));
        assert_eq!(q.recv(), Err(IpcError::Empty));
    }

    #[test]
    fn service_channels_and_opcodes_are_stable() {
        assert_eq!(ServiceChannel::Driverd as u16, 1);
        assert_eq!(ServiceChannel::Aweupdated as u16, 7);
        assert_eq!(IpcOpcode::Hello as u16, 1);
        assert_eq!(IpcOpcode::Handoff as u16, 8);
        let envelope = IpcEnvelope::new(
            ServiceChannel::Driverd,
            IpcOpcode::Hello,
            42,
            IpcMessage::new(7, 1, [1, 2, 3, 4]),
        );
        assert_eq!(envelope.request_id, 42);
        assert_eq!(envelope.message.payload[3], 4);
    }
}
