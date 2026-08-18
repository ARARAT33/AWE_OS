//! Bounded ARP cache and Ethernet/IPv4 address resolution primitives.
//! This module owns validation and deterministic cache semantics; actual frame I/O
//! remains the responsibility of the NetworkDevice implementation.

use super::{Ipv4Address, MacAddress};

pub const ARP_ETHERNET_IPV4: u16 = 0x0800;
pub const ARP_REQUEST: u16 = 1;
pub const ARP_REPLY: u16 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArpError {
    InvalidPacket,
    UnsupportedHardware,
    UnsupportedProtocol,
    InvalidLength,
    EmptyAddress,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArpPacket {
    pub operation: u16,
    pub sender_mac: MacAddress,
    pub sender_ip: Ipv4Address,
    pub target_mac: MacAddress,
    pub target_ip: Ipv4Address,
}

impl ArpPacket {
    pub const WIRE_LEN: usize = 28;

    pub fn parse(bytes: &[u8]) -> Result<Self, ArpError> {
        if bytes.len() < Self::WIRE_LEN {
            return Err(ArpError::InvalidLength);
        }
        if u16::from_be_bytes([bytes[0], bytes[1]]) != 1 {
            return Err(ArpError::UnsupportedHardware);
        }
        if u16::from_be_bytes([bytes[2], bytes[3]]) != ARP_ETHERNET_IPV4 {
            return Err(ArpError::UnsupportedProtocol);
        }
        if bytes[4] != 6 || bytes[5] != 4 {
            return Err(ArpError::InvalidLength);
        }

        let operation = u16::from_be_bytes([bytes[6], bytes[7]]);
        if operation != ARP_REQUEST && operation != ARP_REPLY {
            return Err(ArpError::InvalidPacket);
        }

        let sender_mac = MacAddress([bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13]]);
        let sender_ip = Ipv4Address([bytes[14], bytes[15], bytes[16], bytes[17]]);
        let target_mac = MacAddress([bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23]]);
        let target_ip = Ipv4Address([bytes[24], bytes[25], bytes[26], bytes[27]]);

        if sender_mac == MacAddress::ZERO || sender_ip == Ipv4Address::UNSPECIFIED {
            return Err(ArpError::EmptyAddress);
        }
        Ok(Self { operation, sender_mac, sender_ip, target_mac, target_ip })
    }

    pub fn encode(self, out: &mut [u8]) -> Result<(), ArpError> {
        if out.len() < Self::WIRE_LEN {
            return Err(ArpError::InvalidLength);
        }
        if self.sender_mac == MacAddress::ZERO || self.sender_ip == Ipv4Address::UNSPECIFIED {
            return Err(ArpError::EmptyAddress);
        }
        if self.operation != ARP_REQUEST && self.operation != ARP_REPLY {
            return Err(ArpError::InvalidPacket);
        }

        out[..Self::WIRE_LEN].fill(0);
        out[0..2].copy_from_slice(&1u16.to_be_bytes());
        out[2..4].copy_from_slice(&ARP_ETHERNET_IPV4.to_be_bytes());
        out[4] = 6;
        out[5] = 4;
        out[6..8].copy_from_slice(&self.operation.to_be_bytes());
        out[8..14].copy_from_slice(&self.sender_mac.0);
        out[14..18].copy_from_slice(&self.sender_ip.0);
        out[18..24].copy_from_slice(&self.target_mac.0);
        out[24..28].copy_from_slice(&self.target_ip.0);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Entry {
    ip: Ipv4Address,
    mac: MacAddress,
    age: u64,
}

/// Fixed-capacity ARP cache. Replacement is deterministic: the oldest entry wins.
pub struct ArpCache<const N: usize> {
    entries: [Option<Entry>; N],
    clock: u64,
}

impl<const N: usize> Default for ArpCache<N> {
    fn default() -> Self {
        Self { entries: [None; N], clock: 0 }
    }
}

impl<const N: usize> ArpCache<N> {
    pub fn learn(&mut self, ip: Ipv4Address, mac: MacAddress) -> Result<(), ArpError> {
        if ip == Ipv4Address::UNSPECIFIED || mac == MacAddress::ZERO || !mac.is_unicast() {
            return Err(ArpError::EmptyAddress);
        }
        self.clock = self.clock.saturating_add(1);
        if let Some(entry) = self.entries.iter_mut().flatten().find(|entry| entry.ip == ip) {
            entry.mac = mac;
            entry.age = self.clock;
            return Ok(());
        }
        if let Some(slot) = self.entries.iter_mut().find(|entry| entry.is_none()) {
            *slot = Some(Entry { ip, mac, age: self.clock });
            return Ok(());
        }
        let oldest = self
            .entries
            .iter()
            .enumerate()
            .min_by_key(|(_, entry)| entry.map_or(u64::MAX, |entry| entry.age))
            .map(|(index, _)| index)
            .ok_or(ArpError::InvalidPacket)?;
        self.entries[oldest] = Some(Entry { ip, mac, age: self.clock });
        Ok(())
    }

    pub fn resolve(&self, ip: Ipv4Address) -> Option<MacAddress> {
        self.entries.iter().flatten().find(|entry| entry.ip == ip).map(|entry| entry.mac)
    }

    pub fn len(&self) -> usize {
        self.entries.iter().flatten().count()
    }

    pub const fn capacity(&self) -> usize {
        N
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet() -> ArpPacket {
        ArpPacket {
            operation: ARP_REPLY,
            sender_mac: MacAddress([2, 0, 0, 0, 0, 1]),
            sender_ip: Ipv4Address([192, 168, 1, 10]),
            target_mac: MacAddress([2, 0, 0, 0, 0, 2]),
            target_ip: Ipv4Address([192, 168, 1, 2]),
        }
    }

    #[test]
    fn arp_packet_round_trips() {
        let expected = packet();
        let mut bytes = [0u8; ArpPacket::WIRE_LEN];
        expected.encode(&mut bytes).expect("encode");
        assert_eq!(ArpPacket::parse(&bytes).expect("parse"), expected);
    }

    #[test]
    fn malformed_arp_is_rejected() {
        assert_eq!(ArpPacket::parse(&[0; 8]), Err(ArpError::InvalidLength));
        let mut bytes = [0u8; ArpPacket::WIRE_LEN];
        packet().encode(&mut bytes).expect("encode");
        bytes[0] = 0;
        assert_eq!(ArpPacket::parse(&bytes), Err(ArpError::UnsupportedHardware));
    }

    #[test]
    fn cache_is_bounded_and_replaces_oldest_entry() {
        let mut cache = ArpCache::<2>::default();
        let a = Ipv4Address([10, 0, 0, 1]);
        let b = Ipv4Address([10, 0, 0, 2]);
        let c = Ipv4Address([10, 0, 0, 3]);
        cache.learn(a, MacAddress([2, 0, 0, 0, 0, 1])).expect("learn a");
        cache.learn(b, MacAddress([2, 0, 0, 0, 0, 2])).expect("learn b");
        cache.learn(c, MacAddress([2, 0, 0, 0, 0, 3])).expect("learn c");
        assert_eq!(cache.len(), 2);
        assert!(cache.resolve(a).is_none());
        assert!(cache.resolve(b).is_some());
        assert!(cache.resolve(c).is_some());
    }
}
