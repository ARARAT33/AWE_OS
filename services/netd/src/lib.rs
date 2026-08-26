//! AWEOS User-Space Network Service (`netd`).
#![no_std]

pub mod packet;
pub use packet::{build_udp_ipv4, checksum16, EthernetFrame, Ipv4Header, PacketError, UdpHeader, ETH_HEADER, IPV4_MIN_HEADER, MAX_PACKET, UDP_HEADER};

pub const MAX_INTERFACES: usize = 4;
pub const MAX_SOCKETS: usize = 32;
pub const MAX_FIREWALL_RULES: usize = 16;
pub const MAX_PACKET_LEN: usize = 1514;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAddress(pub [u8; 6]);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ipv4Address(pub [u8; 4]);
impl Ipv4Address {
    pub const UNSPECIFIED: Self = Self([0, 0, 0, 0]);
    pub const LOOPBACK: Self = Self([127, 0, 0, 1]);
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self { Self([a, b, c, d]) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketProtocol { Udp, Tcp }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketState { Closed, Bound, Listening, Connected }

#[derive(Debug, Clone, Copy)]
pub struct NetworkSocket {
    pub socket_id: u32,
    pub protocol: SocketProtocol,
    pub local_port: u16,
    pub remote_port: u16,
    pub local_ip: Ipv4Address,
    pub remote_ip: Ipv4Address,
    pub state: SocketState,
    pub owner_pid: u32,
}
impl NetworkSocket {
    pub const fn new(socket_id: u32, protocol: SocketProtocol, owner_pid: u32) -> Self {
        Self { socket_id, protocol, local_port: 0, remote_port: 0, local_ip: Ipv4Address::UNSPECIFIED, remote_ip: Ipv4Address::UNSPECIFIED, state: SocketState::Closed, owner_pid }
    }
    pub fn bind(&mut self, ip: Ipv4Address, port: u16) -> Result<(), &'static str> {
        if self.state != SocketState::Closed { return Err("Socket already bound or active"); }
        self.local_ip = ip; self.local_port = port; self.state = SocketState::Bound; Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirewallAction { Allow, Deny }
#[derive(Debug, Clone, Copy)]
pub struct FirewallRule {
    pub rule_id: u32,
    pub protocol: Option<SocketProtocol>,
    pub port_range_start: u16,
    pub port_range_end: u16,
    pub action: FirewallAction,
}
impl FirewallRule {
    pub fn matches(&self, protocol: SocketProtocol, port: u16) -> bool {
        if let Some(p) = self.protocol && p != protocol { return false; }
        port >= self.port_range_start && port <= self.port_range_end
    }
}

#[derive(Debug)]
pub struct NetworkDaemon {
    interfaces: [Option<MacAddress>; MAX_INTERFACES],
    sockets: [Option<NetworkSocket>; MAX_SOCKETS],
    firewall_rules: [Option<FirewallRule>; MAX_FIREWALL_RULES],
    socket_counter: u32,
    rule_counter: u32,
}
impl NetworkDaemon {
    pub const fn new() -> Self {
        Self { interfaces: [None; MAX_INTERFACES], sockets: [None; MAX_SOCKETS], firewall_rules: [None; MAX_FIREWALL_RULES], socket_counter: 1, rule_counter: 1 }
    }
    pub fn add_interface(&mut self, mac: MacAddress) -> Result<usize, &'static str> {
        for (idx, slot) in self.interfaces.iter_mut().enumerate() { if slot.is_none() { *slot = Some(mac); return Ok(idx); } }
        Err("No free interface slots")
    }
    pub fn create_socket(&mut self, protocol: SocketProtocol, owner_pid: u32) -> Result<u32, &'static str> {
        let sid = self.socket_counter;
        for slot in self.sockets.iter_mut() { if slot.is_none() { *slot = Some(NetworkSocket::new(sid, protocol, owner_pid)); self.socket_counter = self.socket_counter.saturating_add(1); return Ok(sid); } }
        Err("Socket table full")
    }
    pub fn add_firewall_rule(&mut self, protocol: Option<SocketProtocol>, port_start: u16, port_end: u16, action: FirewallAction) -> Result<u32, &'static str> {
        if port_start > port_end { return Err("Invalid port range"); }
        let rid = self.rule_counter;
        for slot in self.firewall_rules.iter_mut() { if slot.is_none() { *slot = Some(FirewallRule { rule_id: rid, protocol, port_range_start: port_start, port_range_end: port_end, action }); self.rule_counter = self.rule_counter.saturating_add(1); return Ok(rid); } }
        Err("Firewall rule capacity reached")
    }
    pub fn evaluate_packet(&self, protocol: SocketProtocol, dst_port: u16) -> FirewallAction {
        for rule in self.firewall_rules.iter().flatten() { if rule.matches(protocol, dst_port) { return rule.action; } }
        FirewallAction::Deny
    }
    pub fn validate_ipv4_udp(&self, frame: &[u8], expected_destination: Option<u16>) -> Result<(Ipv4Header, UdpHeader), PacketError> {
        let eth = EthernetFrame::parse(frame)?;
        if eth.ethertype != 0x0800 { return Err(PacketError::UnsupportedProtocol); }
        let ip = Ipv4Header::parse(&frame[ETH_HEADER..])?;
        if ip.protocol != 17 { return Err(PacketError::UnsupportedProtocol); }
        let udp_start = ETH_HEADER + ip.header_len;
        let udp = UdpHeader::parse(&frame[udp_start..])?;
        if let Some(port) = expected_destination && udp.destination_port != port { return Err(PacketError::InvalidLength); }
        if self.evaluate_packet(SocketProtocol::Udp, udp.destination_port) != FirewallAction::Allow { return Err(PacketError::UnsupportedProtocol); }
        Ok((ip, udp))
    }
}
impl Default for NetworkDaemon { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn socket_firewall_and_packet_engine_work() {
        let mut netd = NetworkDaemon::new();
        netd.add_interface(MacAddress([0x02,0,0,0,0,1])).unwrap();
        assert_eq!(netd.create_socket(SocketProtocol::Udp, 100).unwrap(), 1);
        netd.add_firewall_rule(Some(SocketProtocol::Udp), 2000, 2000, FirewallAction::Allow).unwrap();
        let mut frame = [0u8; MAX_PACKET_LEN];
        let n = build_udp_ipv4(&mut frame, [1;6], [2;6], [10,0,0,1], [10,0,0,2], 1000, 2000, b"hello").unwrap();
        let (_, udp) = netd.validate_ipv4_udp(&frame[..n], Some(2000)).unwrap();
        assert_eq!(udp.source_port, 1000); assert_eq!(udp.destination_port, 2000);
    }
    #[test]
    fn invalid_port_range_is_rejected() {
        let mut netd = NetworkDaemon::new();
        assert!(netd.add_firewall_rule(None, 200, 100, FirewallAction::Allow).is_err());
    }
}