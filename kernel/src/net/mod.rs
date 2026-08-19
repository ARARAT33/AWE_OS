//! AWE networking core contracts.
//! Protocol implementations and hardware drivers are deliberately separated
//! from transport-neutral packet ownership and endpoint identity.

#![allow(dead_code)]

pub mod arp;
pub mod ipv4;
pub mod policy;
pub mod transport;

pub use arp::{ArpCache, ArpError, ArpPacket};
pub use policy::{Action as FirewallAction, Firewall, Rule as FirewallRule};
pub use transport::{
    Endpoint, SocketEntry, SocketTable, Transport, internet_checksum, tcp_header_valid, udp_payload,
};

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
    pub const BROADCAST: Self = Self([0xff; 6]);
    pub const fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }
    pub fn is_unicast(self) -> bool {
        (self.0[0] & 1) == 0 && self.0 != Self::BROADCAST.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ipv4Address(pub [u8; 4]);

impl Ipv4Address {
    pub const UNSPECIFIED: Self = Self([0, 0, 0, 0]);
    pub const LOOPBACK: Self = Self([127, 0, 0, 1]);
    pub const fn new(bytes: [u8; 4]) -> Self {
        Self(bytes)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EthernetFrame<'a> {
    pub destination: MacAddress,
    pub source: MacAddress,
    pub ethertype: u16,
    pub payload: &'a [u8],
}

impl<'a> EthernetFrame<'a> {
    pub const HEADER_LEN: usize = 14;
    pub fn parse(frame: &'a [u8]) -> Result<Self, NetError> {
        if frame.len() < Self::HEADER_LEN {
            return Err(NetError::BufferTooSmall);
        }
        let destination = MacAddress(
            frame[0..6]
                .try_into()
                .map_err(|_| NetError::BufferTooSmall)?,
        );
        let source = MacAddress(
            frame[6..12]
                .try_into()
                .map_err(|_| NetError::BufferTooSmall)?,
        );
        if !source.is_unicast() {
            return Err(NetError::InvalidAddress);
        }
        let ethertype = u16::from_be_bytes([frame[12], frame[13]]);
        if ethertype == 0 {
            return Err(NetError::Unsupported);
        }
        Ok(Self {
            destination,
            source,
            ethertype,
            payload: &frame[Self::HEADER_LEN..],
        })
    }
    pub fn is_ipv4(self) -> bool {
        self.ethertype == 0x0800
    }
    pub fn is_arp(self) -> bool {
        self.ethertype == 0x0806
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Route {
    pub network: Ipv4Address,
    pub prefix_len: u8,
    pub gateway: Ipv4Address,
    pub interface: u8,
}
impl Route {
    pub fn matches(self, address: Ipv4Address) -> bool {
        if self.prefix_len > 32 {
            return false;
        }
        let mask = if self.prefix_len == 0 {
            0
        } else {
            u32::MAX << (32 - self.prefix_len)
        };
        (u32::from_be_bytes(self.network.0) & mask) == (u32::from_be_bytes(address.0) & mask)
    }
}

pub struct RouteTable<const N: usize> {
    routes: [Option<Route>; N],
}
impl<const N: usize> Default for RouteTable<N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<const N: usize> RouteTable<N> {
    pub const fn new() -> Self {
        Self { routes: [None; N] }
    }
    pub fn add(&mut self, route: Route) -> Result<usize, NetError> {
        if route.prefix_len > 32 {
            return Err(NetError::InvalidAddress);
        }
        if let Some(index) = self.routes.iter().position(Option::is_none) {
            self.routes[index] = Some(route);
            Ok(index)
        } else {
            Err(NetError::WouldBlock)
        }
    }
    pub fn lookup(&self, address: Ipv4Address) -> Result<Route, NetError> {
        let mut best = None;
        for route in self.routes.iter().flatten().copied() {
            if route.matches(address)
                && best.is_none_or(|current: Route| route.prefix_len > current.prefix_len)
            {
                best = Some(route);
            }
        }
        best.ok_or(NetError::NoRoute)
    }
}

pub trait NetworkDevice {
    fn mac(&self) -> MacAddress;
    fn mtu(&self) -> usize;
    fn transmit(&mut self, frame: &[u8]) -> Result<(), NetError>;
    fn receive(&mut self, frame: &mut [u8]) -> Result<usize, NetError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ethernet_frame_is_bounded_and_typed() {
        assert_eq!(
            EthernetFrame::parse(&[0; 13]),
            Err(NetError::BufferTooSmall)
        );
        let mut frame = [0u8; 18];
        frame[0..6].copy_from_slice(&MacAddress::BROADCAST.0);
        frame[6..12].copy_from_slice(&[0x02, 0, 0, 0, 0, 1]);
        frame[12..14].copy_from_slice(&0x0800u16.to_be_bytes());
        frame[14..].copy_from_slice(&[1, 2, 3, 4]);
        let parsed = EthernetFrame::parse(&frame).expect("valid ethernet frame");
        assert!(parsed.is_ipv4());
        assert!(!parsed.is_arp());
        assert_eq!(parsed.payload, &[1, 2, 3, 4]);
    }
    #[test]
    fn route_lookup_uses_longest_prefix() {
        let mut table = RouteTable::<4>::new();
        table
            .add(Route {
                network: Ipv4Address::new([10, 0, 0, 0]),
                prefix_len: 8,
                gateway: Ipv4Address::UNSPECIFIED,
                interface: 1,
            })
            .unwrap();
        table
            .add(Route {
                network: Ipv4Address::new([10, 1, 0, 0]),
                prefix_len: 16,
                gateway: Ipv4Address::new([10, 1, 0, 1]),
                interface: 2,
            })
            .unwrap();
        let route = table.lookup(Ipv4Address::new([10, 1, 2, 3])).unwrap();
        assert_eq!(route.prefix_len, 16);
        assert_eq!(route.interface, 2);
        assert_eq!(
            table.lookup(Ipv4Address::new([192, 0, 2, 1])),
            Err(NetError::NoRoute)
        );
    }
    #[test]
    fn route_table_rejects_invalid_prefix_and_capacity_overflow() {
        let mut table = RouteTable::<1>::new();
        assert_eq!(
            table.add(Route {
                network: Ipv4Address::UNSPECIFIED,
                prefix_len: 33,
                gateway: Ipv4Address::UNSPECIFIED,
                interface: 0
            }),
            Err(NetError::InvalidAddress)
        );
        let route = Route {
            network: Ipv4Address::UNSPECIFIED,
            prefix_len: 0,
            gateway: Ipv4Address::UNSPECIFIED,
            interface: 0,
        };
        table.add(route).unwrap();
        assert_eq!(table.add(route), Err(NetError::WouldBlock));
    }
}
