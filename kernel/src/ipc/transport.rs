#![no_std]

//! 60.6-61.0 service transport boundary.
//! Fixed-capacity, allocation-free service registration, handshake, shared
//! rings and asynchronous request/event primitives.

use super::{IpcEnvelope, IpcOpcode, ServiceChannel};
use crate::system_contract::ServiceId;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapabilityHandle(pub u64);

impl CapabilityHandle {
    pub const INVALID: Self = Self(0);
    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServiceEndpoint {
    pub service: ServiceId,
    pub process: u64,
    pub endpoint: CapabilityHandle,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandshakeState {
    New = 0,
    HelloSent = 1,
    Established = 2,
    Rejected = 3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServiceHello {
    pub service: ServiceId,
    pub abi_major: u16,
    pub abi_minor: u16,
    pub endpoint: CapabilityHandle,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServiceHandshake {
    pub state: HandshakeState,
    pub expected: ServiceHello,
}

impl ServiceHandshake {
    pub const fn new(expected: ServiceHello) -> Self {
        Self {
            state: HandshakeState::New,
            expected,
        }
    }

    pub fn hello_matches(&self, hello: ServiceHello) -> bool {
        hello.service as u16 == self.expected.service as u16
            && hello.abi_major == self.expected.abi_major
            && hello.abi_minor <= self.expected.abi_minor
            && hello.endpoint == self.expected.endpoint
    }

    pub fn accept(&mut self, hello: ServiceHello) -> Result<(), TransportError> {
        if !self.hello_matches(hello) {
            self.state = HandshakeState::Rejected;
            return Err(TransportError::HandshakeRejected);
        }
        self.state = HandshakeState::Established;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportError {
    Full,
    Empty,
    InvalidEndpoint,
    HandshakeRejected,
}

pub struct SharedRing<const N: usize> {
    slots: [Option<IpcEnvelope>; N],
    head: usize,
    len: usize,
}

impl<const N: usize> SharedRing<N> {
    pub const fn new() -> Self {
        Self {
            slots: [None; N],
            head: 0,
            len: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push(&mut self, message: IpcEnvelope) -> Result<(), TransportError> {
        if self.is_full() {
            return Err(TransportError::Full);
        }
        let index = (self.head + self.len) % N;
        self.slots[index] = Some(message);
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<IpcEnvelope, TransportError> {
        if self.is_empty() {
            return Err(TransportError::Empty);
        }
        let value = self.slots[self.head]
            .take()
            .expect("shared ring invariant");
        self.head = (self.head + 1) % N;
        self.len -= 1;
        Ok(value)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingRequest {
    pub request_id: u64,
    pub service: ServiceId,
    pub endpoint: CapabilityHandle,
}

pub struct AsyncRequests<const N: usize> {
    slots: [Option<PendingRequest>; N],
    len: usize,
}

impl<const N: usize> AsyncRequests<N> {
    pub const fn new() -> Self {
        Self {
            slots: [None; N],
            len: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub fn submit(&mut self, request: PendingRequest) -> Result<(), TransportError> {
        if !request.endpoint.is_valid() {
            return Err(TransportError::InvalidEndpoint);
        }
        if self.len == N {
            return Err(TransportError::Full);
        }
        self.slots[self.len] = Some(request);
        self.len += 1;
        Ok(())
    }

    pub fn complete(&mut self, request_id: u64) -> bool {
        let mut i = 0;
        while i < self.len {
            if self.slots[i].map(|request| request.request_id) == Some(request_id) {
                let last = self.len - 1;
                self.slots[i] = self.slots[last];
                self.slots[last] = None;
                self.len -= 1;
                return true;
            }
            i += 1;
        }
        false
    }
}

pub type EventQueue<const N: usize> = SharedRing<N>;

pub const fn service_channel(service: ServiceId) -> ServiceChannel {
    match service {
        ServiceId::Driverd => ServiceChannel::Driverd,
        ServiceId::Appd => ServiceChannel::Appd,
        ServiceId::Asappd => ServiceChannel::Asappd,
        ServiceId::Ayuid => ServiceChannel::Ayuid,
        ServiceId::Aweterminald => ServiceChannel::Aweterminald,
        ServiceId::Awebusd => ServiceChannel::Awebusd,
        ServiceId::Aweupdated => ServiceChannel::Aweupdated,
    }
}

pub const fn opcode_allowed(opcode: IpcOpcode) -> bool {
    matches!(
        opcode,
        IpcOpcode::Hello
            | IpcOpcode::Ping
            | IpcOpcode::Start
            | IpcOpcode::Stop
            | IpcOpcode::Reset
            | IpcOpcode::Query
            | IpcOpcode::Event
            | IpcOpcode::Handoff
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::IpcMessage;

    #[test]
    fn handshake_accepts_compatible_service() {
        let endpoint = CapabilityHandle(9);
        let expected = ServiceHello {
            service: ServiceId::Appd,
            abi_major: 1,
            abi_minor: 2,
            endpoint,
        };
        let mut hs = ServiceHandshake::new(expected);
        assert_eq!(
            hs.accept(ServiceHello {
                service: ServiceId::Appd,
                abi_major: 1,
                abi_minor: 1,
                endpoint,
            }),
            Ok(())
        );
        assert_eq!(hs.state, HandshakeState::Established);
    }

    #[test]
    fn handshake_rejects_wrong_endpoint() {
        let mut hs = ServiceHandshake::new(ServiceHello {
            service: ServiceId::Driverd,
            abi_major: 1,
            abi_minor: 2,
            endpoint: CapabilityHandle(2),
        });
        assert_eq!(
            hs.accept(ServiceHello {
                service: ServiceId::Driverd,
                abi_major: 1,
                abi_minor: 2,
                endpoint: CapabilityHandle(3),
            }),
            Err(TransportError::HandshakeRejected)
        );
    }

    #[test]
    fn ring_and_async_requests_are_bounded() {
        let mut ring: SharedRing<2> = SharedRing::new();
        let message = IpcEnvelope::new(
            ServiceChannel::Appd,
            IpcOpcode::Ping,
            7,
            IpcMessage::new(1, 1, [0, 0, 0, 0]),
        );
        assert_eq!(ring.push(message), Ok(()));
        assert_eq!(ring.pop(), Ok(message));
        let mut pending: AsyncRequests<1> = AsyncRequests::new();
        assert_eq!(
            pending.submit(PendingRequest {
                request_id: 7,
                service: ServiceId::Appd,
                endpoint: CapabilityHandle(1),
            }),
            Ok(())
        );
        assert!(!pending.complete(8));
        assert!(pending.complete(7));
    }
}
