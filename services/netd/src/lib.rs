//! AWEOS User-Space Network Service (`netd`)
//!
//! Manages network interfaces, socket tables, ARP cache, IPv4/UDP/TCP routing,
//! and firewall policy enforcement in user-space.

#![no_std]

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

    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self([a, b, c, d])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketProtocol {
    Udp,
    Tcp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketState {
    Closed,
    Bound,
    Listening,
    Connected,
}

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
        Self {
            socket_id,
            protocol,
            local_port: 0,
            remote_port: 0,
            local_ip: Ipv4Address::UNSPECIFIED,
            remote_ip: Ipv4Address::UNSPECIFIED,
            state: SocketState::Closed,
            owner_pid,
        }
    }

    pub fn bind(&mut self, ip: Ipv4Address, port: u16) -> Result<(), &'static str> {
        if self.state != SocketState::Closed {
            return Err("Socket already bound or active");
        }
        self.local_ip = ip;
        self.local_port = port;
        self.state = SocketState::Bound;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirewallAction {
    Allow,
    Deny,
}

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
        if let Some(p) = self.protocol
            && p != protocol
        {
            return false;
        }
        port >= self.port_range_start && port <= self.port_range_end
    }
}

/// Network Daemon Manager Instance.
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
        Self {
            interfaces: [None; MAX_INTERFACES],
            sockets: [None; MAX_SOCKETS],
            firewall_rules: [None; MAX_FIREWALL_RULES],
            socket_counter: 1,
            rule_counter: 1,
        }
    }

    pub fn add_interface(&mut self, mac: MacAddress) -> Result<usize, &'static str> {
        for (idx, slot) in self.interfaces.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(mac);
                return Ok(idx);
            }
        }
        Err("No free interface slots")
    }

    pub fn create_socket(
        &mut self,
        protocol: SocketProtocol,
        owner_pid: u32,
    ) -> Result<u32, &'static str> {
        let sid = self.socket_counter;
        for slot in self.sockets.iter_mut() {
            if slot.is_none() {
                *slot = Some(NetworkSocket::new(sid, protocol, owner_pid));
                self.socket_counter += 1;
                return Ok(sid);
            }
        }
        Err("Socket table full")
    }

    pub fn add_firewall_rule(
        &mut self,
        protocol: Option<SocketProtocol>,
        port_start: u16,
        port_end: u16,
        action: FirewallAction,
    ) -> Result<u32, &'static str> {
        let rid = self.rule_counter;
        for slot in self.firewall_rules.iter_mut() {
            if slot.is_none() {
                *slot = Some(FirewallRule {
                    rule_id: rid,
                    protocol,
                    port_range_start: port_start,
                    port_range_end: port_end,
                    action,
                });
                self.rule_counter += 1;
                return Ok(rid);
            }
        }
        Err("Firewall rule capacity reached")
    }

    pub fn evaluate_packet(&self, protocol: SocketProtocol, dst_port: u16) -> FirewallAction {
        for rule in self.firewall_rules.iter().flatten() {
            if rule.matches(protocol, dst_port) {
                return rule.action;
            }
        }
        FirewallAction::Deny // Fail-closed by default
    }
}

impl Default for NetworkDaemon {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_netd_socket_and_firewall() {
        let mut netd = NetworkDaemon::new();
        netd.add_interface(MacAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]))
            .unwrap();

        let sock_id = netd.create_socket(SocketProtocol::Udp, 100).unwrap();
        assert_eq!(sock_id, 1);

        // Deny by default
        assert_eq!(
            netd.evaluate_packet(SocketProtocol::Udp, 80),
            FirewallAction::Deny
        );

        // Add rule to allow HTTP (port 80)
        netd.add_firewall_rule(Some(SocketProtocol::Udp), 80, 80, FirewallAction::Allow)
            .unwrap();
        assert_eq!(
            netd.evaluate_packet(SocketProtocol::Udp, 80),
            FirewallAction::Allow
        );
    }
}
