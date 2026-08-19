//! Allocation-free network policy boundary. Rules are evaluated before a
//! transport service admits a flow; unmatched traffic is denied.
use super::{Endpoint, Ipv4Address, NetError, Transport};
pub const MAX_RULES: usize = 32;
#[derive(Clone, Copy, Debug, Eq, PartialEq)] pub enum Action { Allow, Deny }
#[derive(Clone, Copy, Debug, Eq, PartialEq)] pub struct Rule { pub network: Ipv4Address, pub prefix_len: u8, pub port: Option<u16>, pub transport: Option<Transport>, pub action: Action }
impl Rule {
    pub fn matches(self, endpoint: Endpoint, transport: Transport) -> bool {
        if self.prefix_len > 32 { return false; }
        if let Some(port) = self.port && port != endpoint.port { return false; }
        if let Some(expected) = self.transport && expected != transport { return false; }
        let mask = if self.prefix_len == 0 { 0 } else { u32::MAX << (32 - self.prefix_len) };
        (u32::from_be_bytes(self.network.0) & mask) == (u32::from_be_bytes(endpoint.address.0) & mask)
    }
    pub const fn specificity(self) -> u16 { (self.prefix_len as u16) * 4 + if self.port.is_some() { 2 } else { 0 } + if self.transport.is_some() { 1 } else { 0 } }
}
pub struct Firewall<const N: usize = MAX_RULES> { rules: [Option<Rule>; N] }
impl<const N: usize> Default for Firewall<N> { fn default() -> Self { Self::new() } }
impl<const N: usize> Firewall<N> {
    pub const fn new() -> Self { Self { rules: [None; N] } }
    pub fn add(&mut self, rule: Rule) -> Result<usize, NetError> { if rule.prefix_len > 32 || rule.port == Some(0) { return Err(NetError::InvalidAddress); } let slot=self.rules.iter().position(Option::is_none).ok_or(NetError::WouldBlock)?; self.rules[slot]=Some(rule); Ok(slot) }
    pub fn decide(&self, endpoint: Endpoint, transport: Transport) -> Action {
        let mut best: Option<(u16, usize, Action)> = None;
        for (index, rule) in self.rules.iter().flatten().copied().enumerate() {
            if rule.matches(endpoint, transport) {
                let candidate=(rule.specificity(), index, rule.action);
                if best.is_none_or(|current| candidate.0 > current.0 || (candidate.0 == current.0 && candidate.1 < current.1)) { best=Some(candidate); }
            }
        }
        best.map_or(Action::Deny, |(_,_,action)| action)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn policy_is_deny_by_default_and_bounded() { let endpoint=Endpoint::new(Ipv4Address::LOOPBACK,443); let mut fw=Firewall::<1>::new(); assert_eq!(fw.decide(endpoint,Transport::Tcp),Action::Deny); fw.add(Rule{network:Ipv4Address::LOOPBACK,prefix_len:32,port:Some(443),transport:Some(Transport::Tcp),action:Action::Allow}).unwrap(); assert_eq!(fw.decide(endpoint,Transport::Tcp),Action::Allow); assert_eq!(fw.decide(Endpoint::new(Ipv4Address::LOOPBACK,80),Transport::Tcp),Action::Deny); }
    #[test] fn most_specific_rule_wins_independent_of_insertion_order() { let endpoint=Endpoint::new(Ipv4Address::new([10,1,2,3]),443); let mut fw=Firewall::<2>::new(); fw.add(Rule{network:Ipv4Address::new([10,0,0,0]),prefix_len:8,port:None,transport:Some(Transport::Tcp),action:Action::Deny}).unwrap(); fw.add(Rule{network:Ipv4Address::new([10,1,0,0]),prefix_len:16,port:Some(443),transport:Some(Transport::Tcp),action:Action::Allow}).unwrap(); assert_eq!(fw.decide(endpoint,Transport::Tcp),Action::Allow); }
    #[test] fn invalid_rule_and_capacity_fail_closed() { let mut fw=Firewall::<1>::new(); assert_eq!(fw.add(Rule{network:Ipv4Address::UNSPECIFIED,prefix_len:33,port:None,transport:None,action:Action::Deny}),Err(NetError::InvalidAddress)); let rule=Rule{network:Ipv4Address::UNSPECIFIED,prefix_len:0,port:None,transport:None,action:Action::Deny}; fw.add(rule).unwrap(); assert_eq!(fw.add(rule),Err(NetError::WouldBlock)); }
}
