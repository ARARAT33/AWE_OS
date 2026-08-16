#![no_std]

use super::linux_dependency::Dependency;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FaultImpactError {
    Full,
    MissingNode,
}

/// Fixed-capacity dependency impact analyzer for driver fault isolation.
/// If `required_hash -> driver_hash` exists, a fault in the required driver
/// impacts the dependent driver as well.
pub struct FaultImpact<const N: usize> {
    pub nodes: [u64; N],
    pub count: usize,
    pub affected: [u64; N],
    pub affected_len: usize,
}

impl<const N: usize> FaultImpact<N> {
    pub const fn new() -> Self {
        Self { nodes: [0; N], count: 0, affected: [0; N], affected_len: 0 }
    }

    pub fn add_node(&mut self, id: u64) -> Result<(), FaultImpactError> {
        if self.count == N { return Err(FaultImpactError::Full); }
        let mut i = 0;
        while i < self.count {
            if self.nodes[i] == id { return Ok(()); }
            i += 1;
        }
        self.nodes[self.count] = id;
        self.count += 1;
        Ok(())
    }

    /// Computes the transitive set of drivers that depend on `fault_id`.
    /// The result is ordered from direct dependents toward farther dependents.
    pub fn analyze(&mut self, fault_id: u64, deps: &[Dependency]) -> Result<usize, FaultImpactError> {
        let mut known = false;
        let mut i = 0;
        while i < self.count {
            if self.nodes[i] == fault_id { known = true; break; }
            i += 1;
        }
        if !known { return Err(FaultImpactError::MissingNode); }

        self.affected_len = 0;
        let mut frontier = [0u64; N];
        let mut frontier_len = 1;
        frontier[0] = fault_id;

        while frontier_len != 0 {
            let current = frontier[0];
            let mut shift = 1;
            while shift < frontier_len {
                frontier[shift - 1] = frontier[shift];
                shift += 1;
            }
            frontier_len -= 1;

            let mut d = 0;
            while d < deps.len() {
                if deps[d].required_hash == current {
                    let dependent = deps[d].driver_hash;
                    if dependent != fault_id && !self.contains_affected(dependent) {
                        if self.affected_len == N { return Err(FaultImpactError::Full); }
                        self.affected[self.affected_len] = dependent;
                        self.affected_len += 1;
                        if frontier_len < N {
                            frontier[frontier_len] = dependent;
                            frontier_len += 1;
                        }
                    }
                }
                d += 1;
            }
        }
        Ok(self.affected_len)
    }

    fn contains_affected(&self, id: u64) -> bool {
        let mut i = 0;
        while i < self.affected_len {
            if self.affected[i] == id { return true; }
            i += 1;
        }
        false
    }

    /// Produces the safest cleanup order: farthest dependents first.
    pub fn rollback_order(&mut self) {
        let mut i = 0;
        while i < self.affected_len / 2 {
            let j = self.affected_len - 1 - i;
            let tmp = self.affected[i];
            self.affected[i] = self.affected[j];
            self.affected[j] = tmp;
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::linux_dependency::Dependency;

    #[test]
    fn computes_transitive_dependents() {
        let mut impact = FaultImpact::<5>::new();
        impact.add_node(10).unwrap();
        impact.add_node(20).unwrap();
        impact.add_node(30).unwrap();
        impact.add_node(40).unwrap();
        let deps = [
            Dependency { driver_hash: 20, required_hash: 10 },
            Dependency { driver_hash: 30, required_hash: 20 },
            Dependency { driver_hash: 40, required_hash: 30 },
        ];
        assert_eq!(impact.analyze(10, &deps).unwrap(), 3);
        assert_eq!(&impact.affected[..impact.affected_len], &[20, 30, 40]);
        impact.rollback_order();
        assert_eq!(&impact.affected[..impact.affected_len], &[40, 30, 20]);
    }

    #[test]
    fn ignores_unrelated_branches() {
        let mut impact = FaultImpact::<6>::new();
        for id in [10, 20, 30, 50, 60, 70] { impact.add_node(id).unwrap(); }
        let deps = [
            Dependency { driver_hash: 20, required_hash: 10 },
            Dependency { driver_hash: 30, required_hash: 20 },
            Dependency { driver_hash: 60, required_hash: 50 },
            Dependency { driver_hash: 70, required_hash: 60 },
        ];
        assert_eq!(impact.analyze(10, &deps).unwrap(), 2);
        assert_eq!(&impact.affected[..impact.affected_len], &[20, 30]);
    }
}
