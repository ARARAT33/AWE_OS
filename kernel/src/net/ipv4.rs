//! Dependency-free IPv4 packet primitives.
//! Parsing is bounded, allocation-free and never exposes bytes outside the packet.

use super::{Ipv4Address, NetError};

pub const IPV4_MIN_HEADER: usize = 20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Header {
    pub version: u8,
    pub ihl: u8,
    pub dscp_ecn: u8,
    pub total_len: u16,
    pub identification: u16,
    pub flags_fragment: u16,
    pub ttl: u8,
    pub protocol: u8,
    pub checksum: u16,
    pub source: Ipv4Address,
    pub destination: Ipv4Address,
}

impl Header {
    pub fn parse(packet: &[u8]) -> Result<Self, NetError> {
        if packet.len() < IPV4_MIN_HEADER {
            return Err(NetError::BufferTooSmall);
        }
        let first = packet[0];
        let version = first >> 4;
        let ihl = first & 0x0f;
        if version != 4 || ihl < 5 {
            return Err(NetError::InvalidAddress);
        }
        let header_len = (ihl as usize) * 4;
        if packet.len() < header_len {
            return Err(NetError::BufferTooSmall);
        }
        let total_len = u16::from_be_bytes([packet[2], packet[3]]);
        if (total_len as usize) < header_len || (total_len as usize) > packet.len() {
            return Err(NetError::BufferTooSmall);
        }
        Ok(Self {
            version,
            ihl,
            dscp_ecn: packet[1],
            total_len,
            identification: u16::from_be_bytes([packet[4], packet[5]]),
            flags_fragment: u16::from_be_bytes([packet[6], packet[7]]),
            ttl: packet[8],
            protocol: packet[9],
            checksum: u16::from_be_bytes([packet[10], packet[11]]),
            source: Ipv4Address([packet[12], packet[13], packet[14], packet[15]]),
            destination: Ipv4Address([packet[16], packet[17], packet[18], packet[19]]),
        })
    }

    pub const fn header_len(self) -> usize {
        self.ihl as usize * 4
    }

    pub fn payload(self, packet: &[u8]) -> Result<&[u8], NetError> {
        let h = self.header_len();
        let end = self.total_len as usize;
        if end < h || end > packet.len() {
            return Err(NetError::BufferTooSmall);
        }
        Ok(&packet[h..end])
    }

    pub fn checksum_valid(packet: &[u8]) -> Result<bool, NetError> {
        let h = Self::parse(packet)?;
        let len = h.header_len();
        let mut sum = 0u32;
        let mut i = 0usize;
        while i < len {
            sum = sum.wrapping_add(u16::from_be_bytes([packet[i], packet[i + 1]]) as u32);
            i += 2;
        }
        while (sum >> 16) != 0 {
            sum = (sum & 0xffff) + (sum >> 16);
        }
        Ok((sum as u16) == 0xffff)
    }
}
