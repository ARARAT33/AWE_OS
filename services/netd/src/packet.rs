#![no_std]

pub const ETH_HEADER: usize = 14;
pub const IPV4_MIN_HEADER: usize = 20;
pub const UDP_HEADER: usize = 8;
pub const MAX_PACKET: usize = 1514;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PacketError { TooShort, InvalidEthernet, InvalidIpv4, InvalidChecksum, InvalidLength, UnsupportedProtocol, OutputTooSmall }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EthernetFrame { pub destination: [u8; 6], pub source: [u8; 6], pub ethertype: u16 }
impl EthernetFrame {
    pub fn parse(bytes: &[u8]) -> Result<Self, PacketError> {
        if bytes.len() < ETH_HEADER { return Err(PacketError::TooShort); }
        Ok(Self { destination: bytes[0..6].try_into().map_err(|_| PacketError::InvalidEthernet)?, source: bytes[6..12].try_into().map_err(|_| PacketError::InvalidEthernet)?, ethertype: u16::from_be_bytes([bytes[12], bytes[13]]) })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ipv4Header { pub source: [u8;4], pub destination: [u8;4], pub total_length: u16, pub protocol: u8, pub header_len: usize }
impl Ipv4Header {
    pub fn parse(bytes: &[u8]) -> Result<Self, PacketError> {
        if bytes.len() < IPV4_MIN_HEADER { return Err(PacketError::TooShort); }
        if bytes[0] >> 4 != 4 { return Err(PacketError::InvalidIpv4); }
        let header_len = ((bytes[0] & 0x0f) as usize) * 4;
        if header_len < IPV4_MIN_HEADER || header_len > bytes.len() { return Err(PacketError::InvalidIpv4); }
        let total_length = u16::from_be_bytes([bytes[2], bytes[3]]);
        if total_length as usize > bytes.len() || total_length as usize < header_len { return Err(PacketError::InvalidLength); }
        let expected = u16::from_be_bytes([bytes[10], bytes[11]]);
        let mut header = [0u8; 60];
        header[..header_len].copy_from_slice(&bytes[..header_len]);
        header[10] = 0; header[11] = 0;
        if checksum16(&header[..header_len]) != expected { return Err(PacketError::InvalidChecksum); }
        Ok(Self { source: bytes[12..16].try_into().map_err(|_| PacketError::InvalidIpv4)?, destination: bytes[16..20].try_into().map_err(|_| PacketError::InvalidIpv4)?, total_length, protocol: bytes[9], header_len })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UdpHeader { pub source_port: u16, pub destination_port: u16, pub length: u16 }
impl UdpHeader {
    pub fn parse(bytes: &[u8]) -> Result<Self, PacketError> {
        if bytes.len() < UDP_HEADER { return Err(PacketError::TooShort); }
        let length = u16::from_be_bytes([bytes[4], bytes[5]]);
        if length < UDP_HEADER as u16 || length as usize > bytes.len() { return Err(PacketError::InvalidLength); }
        Ok(Self { source_port: u16::from_be_bytes([bytes[0], bytes[1]]), destination_port: u16::from_be_bytes([bytes[2], bytes[3]]), length })
    }
}

pub fn checksum16(bytes: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < bytes.len() { sum = sum.wrapping_add(u16::from_be_bytes([bytes[i], bytes[i + 1]]) as u32); i += 2; }
    if i < bytes.len() { sum = sum.wrapping_add((bytes[i] as u32) << 8); }
    while (sum >> 16) != 0 { sum = (sum & 0xffff) + (sum >> 16); }
    !(sum as u16)
}

pub fn build_udp_ipv4(out: &mut [u8], source_mac: [u8; 6], destination_mac: [u8; 6], source_ip: [u8; 4], destination_ip: [u8; 4], source_port: u16, destination_port: u16, payload: &[u8]) -> Result<usize, PacketError> {
    let total = ETH_HEADER.checked_add(IPV4_MIN_HEADER).and_then(|v| v.checked_add(UDP_HEADER)).and_then(|v| v.checked_add(payload.len())).ok_or(PacketError::InvalidLength)?;
    if total > MAX_PACKET || out.len() < total { return Err(PacketError::OutputTooSmall); }
    out[0..6].copy_from_slice(&destination_mac); out[6..12].copy_from_slice(&source_mac); out[12..14].copy_from_slice(&0x0800u16.to_be_bytes());
    let ip = ETH_HEADER;
    out[ip] = 0x45; out[ip+1] = 0;
    out[ip+2..ip+4].copy_from_slice(&((total - ETH_HEADER) as u16).to_be_bytes());
    out[ip+4..ip+6].fill(0); out[ip+6..ip+8].copy_from_slice(&0x4000u16.to_be_bytes()); out[ip+8] = 64; out[ip+9] = 17; out[ip+10..ip+12].fill(0);
    out[ip+12..ip+16].copy_from_slice(&source_ip); out[ip+16..ip+20].copy_from_slice(&destination_ip);
    let c = checksum16(&out[ip..ip+20]); out[ip+10..ip+12].copy_from_slice(&c.to_be_bytes());
    let udp = ip + IPV4_MIN_HEADER;
    out[udp..udp+2].copy_from_slice(&source_port.to_be_bytes()); out[udp+2..udp+4].copy_from_slice(&destination_port.to_be_bytes());
    let udp_len = u16::try_from(UDP_HEADER + payload.len()).map_err(|_| PacketError::InvalidLength)?;
    out[udp+4..udp+6].copy_from_slice(&udp_len.to_be_bytes()); out[udp+6..udp+8].fill(0); out[udp+8..udp+8+payload.len()].copy_from_slice(payload);
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn udp_ipv4_packet_build_and_parse() {
        let mut packet = [0u8; MAX_PACKET];
        let n = build_udp_ipv4(&mut packet, [1,2,3,4,5,6], [6,5,4,3,2,1], [10,0,0,1], [10,0,0,2], 1000, 2000, b"hello").unwrap();
        assert_eq!(EthernetFrame::parse(&packet[..n]).unwrap().ethertype, 0x0800);
        let ip = Ipv4Header::parse(&packet[ETH_HEADER..n]).unwrap();
        assert_eq!(ip.protocol, 17); assert_eq!(ip.source, [10,0,0,1]);
        let udp = UdpHeader::parse(&packet[ETH_HEADER + ip.header_len..n]).unwrap();
        assert_eq!(udp.destination_port, 2000); assert_eq!(udp.length, 13);
    }
    #[test] fn rejects_bad_ipv4_checksum() {
        let mut packet = [0u8; MAX_PACKET];
        let n = build_udp_ipv4(&mut packet, [1;6], [2;6], [10,0,0,1], [10,0,0,2], 1, 2, b"x").unwrap();
        packet[ETH_HEADER + 8] ^= 1;
        assert_eq!(Ipv4Header::parse(&packet[ETH_HEADER..n]), Err(PacketError::InvalidChecksum));
    }
}