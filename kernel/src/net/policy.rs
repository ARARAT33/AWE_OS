//! Allocation-free network policy boundary.
//! Rules are evaluated before a transport service admits a flow.

use super::{Endpoint, Ipv4Address, NetError, Transport};

pub const MAX_RULES: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action { Allow, Deny }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rule {
    pub network: Ipv4Address,
    pub prefix_len: u8,
    pub port: Option<u16>,
    pub transport: Option<Transport>,
    pub action: Action,
}

impl Rule {
    pub fn matches(self, endpoint: Endpoint, transport: Transport) -> bool {
        if self.prefix_len > 32 { return false; }
        if let Some(port) = self.port && port != endpoint.port { return false; }
        if let Some(expected) = self.transport && expected != transport { return false; }
        let mask = if self.prefix_len == 0 { 0 } else { u32::MAX << (32 - self.prefix_len) };
        (u32::from_be_bytes(self.network.0) & mask) == (u32::from_be_bytes(endpoint.address.0) & mask)
    }
}

pub struct Firewall<const N: usize = MAX_RULES> { rules: [Option<Rule>; N] }

impl<const N: usize> Default for Firewall<N> { fn default() -> Self { Self::new() } }

impl<const N: usize> Firewall<N> {
    pub const fn new() -> Self { Self { rules: [None; N] } }

    pub fn add(&mut self, rule: Rule) -> Result<usize, NetError> {
        if rule.prefix_len > 32 || rule.port == Some(0) { return Err(NetError::InvalidAddress); }
        let slot = self.rules.iter().position(Option::is_none).ok_or(NetError::WouldBlock)?;
        self.rules[slot] = Some(rule);
        Ok(slot)
    }

    pub fn decide(&self, endpoint: Endpoint, transport: Transport) -> Action {
        for rule in self.rules.iter().flatten().copied() {
            if rule.matches(endpoint, transport) { return rule.action; }
        }
        Action::Deny
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_is_deny_by_default_and_bounded() {
        let endpoint = Endpoint::new(Ipv4Address::LOOPBACK, 443);
        let mut fw = Firewall::<1>::new();
        assert_eq!(fw.decide(endpoint, Transport::Tcp), Action::Deny);
        fw.add(Rule { network: Ipv4Address::LOOPBACK, prefix_len: 32, port: Some(443), transport: Some(Transport::Tcp), action: Action::Allow }).unwrap();
        assert_eq!(fw.decide(endpoint, Transport::Tcp), Action::Allow);
        assert_eq!(fw.decide(Endpoint::new(Ipv4Address::LOOPBACK, 80), Transport::Tcp), Action::Deny);
        assert_eq!(fw.add(Rule { network: Ipv4Address::UNSPECIFIED, prefix_len: 0, port: None, transport: None, action: Action::Deny }), Err(NetError::WouldBlock));
    }
}
