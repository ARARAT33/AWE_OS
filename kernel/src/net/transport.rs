//! Allocation-free transport primitives. Full socket I/O stays in userspace
//! network services; the kernel validates packet metadata and endpoint identity.

use super::{Ipv4Address, NetError};

pub const UDP_HEADER: usize = 8;
pub const TCP_MIN_HEADER: usize = 20;
pub const MAX_ENDPOINTS: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Endpoint { pub address: Ipv4Address, pub port: u16 }

impl Endpoint {
    pub const fn new(address: Ipv4Address, port: u16) -> Self { Self { address, port } }
    pub fn valid(self) -> bool { self.port != 0 }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Transport { Udp, Tcp }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SocketEntry { pub local: Endpoint, pub remote: Option<Endpoint>, pub transport: Transport, pub connected: bool }

pub struct SocketTable<const N: usize = MAX_ENDPOINTS> { entries: [Option<SocketEntry>; N] }

impl<const N: usize> SocketTable<N> {
    pub const fn new() -> Self { Self { entries: [None; N] } }
    pub fn bind(&mut self, endpoint: Endpoint, transport: Transport) -> Result<usize, NetError> {
        if !endpoint.valid() || self.entries.iter().flatten().any(|e| e.local == endpoint && e.transport == transport) { return Err(NetError::InvalidAddress); }
        let slot = self.entries.iter().position(Option::is_none).ok_or(NetError::WouldBlock)?;
        self.entries[slot] = Some(SocketEntry { local: endpoint, remote: None, transport, connected: false });
        Ok(slot)
    }
    pub fn connect(&mut self, slot: usize, remote: Endpoint) -> Result<(), NetError> {
        if !remote.valid() { return Err(NetError::InvalidAddress); }
        let entry = self.entries.get_mut(slot).and_then(Option::as_mut).ok_or(NetError::InvalidAddress)?;
        entry.remote = Some(remote); entry.connected = true; Ok(())
    }
    pub fn get(&self, slot: usize) -> Option<SocketEntry> { self.entries.get(slot).copied().flatten() }
}
impl<const N: usize> Default for SocketTable<N> { fn default() -> Self { Self::new() } }

pub fn internet_checksum(data: &[u8]) -> u16 {
    let mut sum = 0u32; let mut i = 0;
    while i + 1 < data.len() { sum = sum.wrapping_add(u16::from_be_bytes([data[i], data[i + 1]]) as u32); i += 2; }
    if i < data.len() { sum = sum.wrapping_add((data[i] as u32) << 8); }
    while (sum >> 16) != 0 { sum = (sum & 0xffff) + (sum >> 16); }
    !(sum as u16)
}

pub fn udp_payload(packet: &[u8]) -> Result<(Endpoint, Endpoint, &[u8]), NetError> {
    if packet.len() < UDP_HEADER { return Err(NetError::BufferTooSmall); }
    let src = u16::from_be_bytes([packet[0], packet[1]]); let dst = u16::from_be_bytes([packet[2], packet[3]]);
    let len = u16::from_be_bytes([packet[4], packet[5]]) as usize;
    if src == 0 || dst == 0 || len < UDP_HEADER || len > packet.len() { return Err(NetError::InvalidAddress); }
    Ok((Endpoint::new(Ipv4Address::UNSPECIFIED, src), Endpoint::new(Ipv4Address::UNSPECIFIED, dst), &packet[UDP_HEADER..len]))
}

pub fn tcp_header_valid(packet: &[u8]) -> Result<usize, NetError> {
    if packet.len() < TCP_MIN_HEADER { return Err(NetError::BufferTooSmall); }
    let offset = ((packet[12] >> 4) as usize) * 4;
    if offset < TCP_MIN_HEADER || offset > packet.len() { return Err(NetError::BufferTooSmall); }
    let src = u16::from_be_bytes([packet[0], packet[1]]); let dst = u16::from_be_bytes([packet[2], packet[3]]);
    if src == 0 || dst == 0 { return Err(NetError::InvalidAddress); }
    Ok(offset)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checksum_is_deterministic() { assert_eq!(internet_checksum(&[0, 1, 0, 2]), 0xfffc); }
    #[test]
    fn socket_table_enforces_bounds_and_identity() {
        let mut t = SocketTable::<1>::new(); let e = Endpoint::new(Ipv4Address::LOOPBACK, 8080);
        let slot = t.bind(e, Transport::Tcp).unwrap(); assert!(t.bind(e, Transport::Tcp).is_err());
        t.connect(slot, Endpoint::new(Ipv4Address::LOOPBACK, 443)).unwrap(); assert!(t.get(slot).unwrap().connected);
        assert!(t.connect(4, e).is_err());
    }
    #[test]
    fn udp_and_tcp_lengths_fail_closed() {
        assert!(udp_payload(&[0; 7]).is_err());
        let mut tcp = [0u8; 20]; tcp[0..2].copy_from_slice(&1u16.to_be_bytes()); tcp[2..4].copy_from_slice(&2u16.to_be_bytes()); tcp[12] = 5 << 4;
        assert_eq!(tcp_header_valid(&tcp).unwrap(), 20);
    }
}
