#![no_std]
use super::linux_dependency::Dependency;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FaultImpactError {
    Full,
    MissingNode,
}
pub struct FaultImpact<const N: usize> {
    pub nodes: [u64; N],
    pub count: usize,
    pub affected: [u64; N],
    pub affected_len: usize,
}
impl<const N: usize> FaultImpact<N> {
    pub const fn new() -> Self {
        Self {
            nodes: [0; N],
            count: 0,
            affected: [0; N],
            affected_len: 0,
        }
    }
    pub fn add_node(&mut self, id: u64) -> Result<(), FaultImpactError> {
        if self.count == N {
            return Err(FaultImpactError::Full);
        }
        let mut i = 0;
        while i < self.count {
            if self.nodes[i] == id {
                return Ok(());
            }
            i += 1
        }
        self.nodes[self.count] = id;
        self.count += 1;
        Ok(())
    }
    pub fn analyze(
        &mut self,
        fault_id: u64,
        deps: &[Dependency],
    ) -> Result<usize, FaultImpactError> {
        let mut known = false;
        let mut i = 0;
        while i < self.count {
            if self.nodes[i] == fault_id {
                known = true;
                break;
            }
            i += 1
        }
        if !known {
            return Err(FaultImpactError::MissingNode);
        }
        self.affected_len = 0;
        let mut frontier = [0u64; N];
        let mut frontier_len = 1;
        frontier[0] = fault_id;
        while frontier_len != 0 {
            let current = frontier[0];
            let mut shift = 1;
            while shift < frontier_len {
                frontier[shift - 1] = frontier[shift];
                shift += 1
            }
            frontier_len -= 1;
            let mut d = 0;
            while d < deps.len() {
                if deps[d].required_hash == current {
                    let dependent = deps[d].driver_hash;
                    if dependent != fault_id && !self.contains_affected(dependent) {
                        if self.affected_len == N {
                            return Err(FaultImpactError::Full);
                        }
                        self.affected[self.affected_len] = dependent;
                        self.affected_len += 1;
                        if frontier_len < N {
                            frontier[frontier_len] = dependent;
                            frontier_len += 1
                        }
                    }
                }
                d += 1
            }
        }
        Ok(self.affected_len)
    }
    pub fn compute(
        &mut self,
        fault_id: u64,
        deps: &[Dependency],
    ) -> Result<usize, FaultImpactError> {
        self.analyze(fault_id, deps)
    }
    pub fn compute_pairs(
        &mut self,
        fault_id: u64,
        pairs: &[(u64, u64)],
    ) -> Result<usize, FaultImpactError> {
        self.affected_len = 0;
        self.count = 0;
        self.add_node(fault_id)?;
        let mut i = 0;
        while i < pairs.len() {
            self.add_node(pairs[i].0)?;
            self.add_node(pairs[i].1)?;
            i += 1
        }
        let mut frontier = [0u64; N];
        let mut frontier_len = 1;
        frontier[0] = fault_id;
        while frontier_len != 0 {
            let current = frontier[0];
            let mut s = 1;
            while s < frontier_len {
                frontier[s - 1] = frontier[s];
                s += 1
            }
            frontier_len -= 1;
            let mut p = 0;
            while p < pairs.len() {
                if pairs[p].0 == current {
                    let dependent = pairs[p].1;
                    if dependent != fault_id && !self.contains_affected(dependent) {
                        if self.affected_len == N {
                            return Err(FaultImpactError::Full);
                        }
                        self.affected[self.affected_len] = dependent;
                        self.affected_len += 1;
                        if frontier_len < N {
                            frontier[frontier_len] = dependent;
                            frontier_len += 1
                        }
                    }
                }
                p += 1
            }
        }
        Ok(self.affected_len)
    }
    fn contains_affected(&self, id: u64) -> bool {
        let mut i = 0;
        while i < self.affected_len {
            if self.affected[i] == id {
                return true;
            }
            i += 1
        }
        false
    }
    pub fn rollback_order(&mut self) {
        let mut i = 0;
        while i < self.affected_len / 2 {
            let j = self.affected_len - 1 - i;
            self.affected.swap(i, j);
            i += 1
        }
    }
}

impl<const N: usize> Default for FaultImpact<N> {
    fn default() -> Self {
        Self::new()
    }
}
