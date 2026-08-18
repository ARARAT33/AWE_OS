//! Bounded IPv6 packet primitives.
//! The parser validates the fixed header, payload length and next-header chain
//! boundary without allocation or exposing bytes outside the packet.

use super::{Ipv6Address, NetError};

pub const IPV6_HEADER_LEN: usize = 40;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Header {
    pub traffic_class: u8,
    pub flow_label: u32,
    pub payload_len: u16,
    pub next_header: u8,
    pub hop_limit: u8,
    pub source: Ipv6Address,
    pub destination: Ipv6Address,
}

impl Header {
    pub fn parse(packet: &[u8]) -> Result<Self, NetError> {
        if packet.len() < IPV6_HEADER_LEN {
            return Err(NetError::BufferTooSmall);
        }
        let first = u32::from_be_bytes([packet[0], packet[1], packet[2], packet[3]]);
        if first >> 28 != 6 {
            return Err(NetError::InvalidAddress);
        }
        let payload_len = u16::from_be_bytes([packet[4], packet[5]]) as usize;
        let total = IPV6_HEADER_LEN
            .checked_add(payload_len)
            .ok_or(NetError::BufferTooSmall)?;
        if total > packet.len() {
            return Err(NetError::BufferTooSmall);
        }
        let mut source = [0u8; 16];
        let mut destination = [0u8; 16];
        source.copy_from_slice(&packet[8..24]);
        destination.copy_from_slice(&packet[24..40]);
        Ok(Self {
            traffic_class: ((first >> 20) & 0xff) as u8,
            flow_label: first & 0x000f_ffff,
            payload_len: payload_len as u16,
            next_header: packet[6],
            hop_limit: packet[7],
            source: Ipv6Address(source),
            destination: Ipv6Address(destination),
        })
    }

    pub fn payload<'a>(self, packet: &'a [u8]) -> Result<&'a [u8], NetError> {
        let end = IPV6_HEADER_LEN
            .checked_add(self.payload_len as usize)
            .ok_or(NetError::BufferTooSmall)?;
        if end > packet.len() {
            return Err(NetError::BufferTooSmall);
        }
        Ok(&packet[IPV6_HEADER_LEN..end])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bounded_ipv6_header() {
        let mut packet = [0u8; IPV6_HEADER_LEN + 4];
        packet[0] = 0x60;
        packet[4..6].copy_from_slice(&4u16.to_be_bytes());
        packet[6] = 17;
        packet[7] = 64;
        packet[40..44].copy_from_slice(&[1, 2, 3, 4]);
        let header = Header::parse(&packet).unwrap();
        assert_eq!(header.next_header, 17);
        assert_eq!(header.hop_limit, 64);
        assert_eq!(header.payload(&packet).unwrap(), &[1, 2, 3, 4]);
    }

    #[test]
    fn rejects_invalid_version_and_truncated_payload() {
        let mut packet = [0u8; IPV6_HEADER_LEN];
        packet[0] = 0x40;
        assert_eq!(Header::parse(&packet), Err(NetError::InvalidAddress));
        packet[0] = 0x60;
        packet[4..6].copy_from_slice(&1u16.to_be_bytes());
        assert_eq!(Header::parse(&packet), Err(NetError::BufferTooSmall));
    }
}
