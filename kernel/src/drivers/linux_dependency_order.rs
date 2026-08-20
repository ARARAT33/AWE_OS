#![no_std]
use super::linux_dependency::Dependency;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OrderError {
    MissingNode,
    Cycle,
    OutputFull,
}
/// Deterministic dependency-first topological order. Required nodes are
/// preferred one-at-a-time over unrelated roots, so independent drivers never
/// jump ahead of an activation chain.
pub fn topological_order(
    nodes: &[u64],
    deps: &[Dependency],
    out: &mut [u64],
) -> Result<usize, OrderError> {
    if out.len() < nodes.len() {
        return Err(OrderError::OutputFull);
    }
    for dep in deps {
        if !nodes.contains(&dep.driver_hash) || !nodes.contains(&dep.required_hash) {
            return Err(OrderError::MissingNode);
        }
    }
    let mut emitted = [false; 128];
    if nodes.len() > emitted.len() {
        return Err(OrderError::OutputFull);
    }
    let mut count = 0usize;
    while count < nodes.len() {
        let mut selected = None;
        for i in 0..nodes.len() {
            if emitted[i] {
                continue;
            }
            let node = nodes[i];
            let ready = deps.iter().filter(|d| d.driver_hash == node).all(|d| {
                nodes
                    .iter()
                    .position(|n| *n == d.required_hash)
                    .map(|j| emitted[j])
                    .unwrap_or(false)
            });
            if ready && deps.iter().any(|d| d.required_hash == node) {
                selected = Some(i);
                break;
            }
        }
        if selected.is_none() {
            for i in 0..nodes.len() {
                if emitted[i] {
                    continue;
                }
                let node = nodes[i];
                let ready = deps.iter().filter(|d| d.driver_hash == node).all(|d| {
                    nodes
                        .iter()
                        .position(|n| *n == d.required_hash)
                        .map(|j| emitted[j])
                        .unwrap_or(false)
                });
                if ready {
                    selected = Some(i);
                    break;
                }
            }
        }
        let i = selected.ok_or(OrderError::Cycle)?;
        out[count] = nodes[i];
        emitted[i] = true;
        count += 1;
    }
    Ok(count)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dependencies_are_emitted_first() {
        let nodes = [1, 2, 3];
        let deps = [
            Dependency {
                driver_hash: 1,
                required_hash: 2,
            },
            Dependency {
                driver_hash: 2,
                required_hash: 3,
            },
        ];
        let mut out = [0; 3];
        let n = topological_order(&nodes, &deps, &mut out).unwrap();
        assert_eq!(n, 3);
        assert_eq!(out, [3, 2, 1])
    }
    #[test]
    fn independent_nodes_do_not_precede_chain() {
        let nodes = [10, 20, 30, 40];
        let deps = [
            Dependency {
                driver_hash: 10,
                required_hash: 20,
            },
            Dependency {
                driver_hash: 20,
                required_hash: 30,
            },
        ];
        let mut out = [0; 4];
        topological_order(&nodes, &deps, &mut out).unwrap();
        assert_eq!(out, [30, 20, 10, 40])
    }
    #[test]
    fn cycle_is_rejected() {
        let nodes = [1, 2];
        let deps = [
            Dependency {
                driver_hash: 1,
                required_hash: 2,
            },
            Dependency {
                driver_hash: 2,
                required_hash: 1,
            },
        ];
        let mut out = [0; 2];
        assert_eq!(
            topological_order(&nodes, &deps, &mut out),
            Err(OrderError::Cycle)
        )
    }
}
