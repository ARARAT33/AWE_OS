//! AWE networking core contracts.
//! Protocol implementations and hardware drivers are deliberately separated
//! from transport-neutral packet ownership and endpoint identity.

#![allow(dead_code)]

pub mod ipv4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetError {
    BufferTooSmall,
    InvalidAddress,
    WouldBlock,
    NoRoute,
    Io,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MacAddress(pub [u8; 6]);

impl MacAddress {
    pub const ZERO: Self = Self([0; 6]);

    pub fn is_unicast(self) -> bool {
        (self.0[0] & 1) == 0 && self.0 != [0xff; 6]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ipv4Address(pub [u8; 4]);

impl Ipv4Address {
    pub const UNSPECIFIED: Self = Self([0, 0, 0, 0]);
    pub const LOOPBACK: Self = Self([127, 0, 0, 1]);
}

pub trait NetworkDevice {
    fn mac(&self) -> MacAddress;
    fn mtu(&self) -> usize;
    fn transmit(&mut self, frame: &[u8]) -> Result<(), NetError>;
    fn receive(&mut self, frame: &mut [u8]) -> Result<usize, NetError>;
}
