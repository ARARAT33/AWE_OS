#![no_std]

use super::linux_dependency::Dependency;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GraphError { MissingNode, Cycle }

/// Allocation-free bounded dependency graph validation.
/// Nodes are represented by module hashes and edges by Dependency records.
pub fn validate_graph(nodes: &[u64], edges: &[Dependency]) -> Result<(), GraphError> {
    for edge in edges {
        if !nodes.iter().any(|n| *n == edge.driver_hash) || !nodes.iter().any(|n| *n == edge.required_hash) {
            return Err(GraphError::MissingNode);
        }
    }
    for start in nodes {
        let mut current = *start;
        let mut steps = 0usize;
        while steps <= nodes.len() {
            let next = edges.iter().find(|e| e.driver_hash == current);
            match next {
                None => break,
                Some(edge) if edge.required_hash == *start => return Err(GraphError::Cycle),
                Some(edge) => current = edge.required_hash,
            }
            steps += 1;
        }
        if steps > nodes.len() { return Err(GraphError::Cycle); }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_chain() { assert!(validate_graph(&[1,2,3], &[Dependency{driver_hash:1,required_hash:2},Dependency{driver_hash:2,required_hash:3}]).is_ok()); }
    #[test]
    fn detects_cycle() { assert_eq!(validate_graph(&[1,2,3], &[Dependency{driver_hash:1,required_hash:2},Dependency{driver_hash:2,required_hash:3},Dependency{driver_hash:3,required_hash:1}]), Err(GraphError::Cycle)); }
    #[test]
    fn detects_missing_node() { assert_eq!(validate_graph(&[1], &[Dependency{driver_hash:1,required_hash:9}]), Err(GraphError::MissingNode)); }
}
