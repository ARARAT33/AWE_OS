#![no_std]
use super::linux_dependency::Dependency;
#[derive(Clone,Copy,PartialEq,Eq,Debug)]pub enum OrderError{MissingNode,Cycle,OutputFull}
/// Deterministic dependency-first topological order. Nodes that are required
/// by another node are preferred over unrelated roots, keeping activation and
/// rollback stable even when independent drivers are present.
pub fn topological_order(nodes:&[u64],deps:&[Dependency],out:&mut[u64])->Result<usize,OrderError>{
 if out.len()<nodes.len(){return Err(OrderError::OutputFull)}
 for dep in deps{if !nodes.iter().any(|n|*n==dep.driver_hash)||!nodes.iter().any(|n|*n==dep.required_hash){return Err(OrderError::MissingNode)}}
 let mut emitted=[false;128];if nodes.len()>emitted.len(){return Err(OrderError::OutputFull)}let mut count=0usize;
 while count<nodes.len(){
  let mut progress=false;
  // First emit ready dependency nodes. This makes chains deterministic and
  // prevents unrelated nodes from consuming the earliest activation slots.
  for pass in 0..2{for i in 0..nodes.len(){if emitted[i]{continue}let node=nodes[i];let ready=deps.iter().filter(|d|d.driver_hash==node).all(|d|nodes.iter().position(|n|*n==d.required_hash).map(|j|emitted[j]).unwrap_or(false));if !ready{continue}let is_required=deps.iter().any(|d|d.required_hash==node);if (pass==0)!=is_required{continue}out[count]=node;emitted[i]=true;count+=1;progress=true;}}
  if !progress{return Err(OrderError::Cycle)}
 }
 Ok(count)
}
#[cfg(test)]mod tests{use super::*;#[test]fn dependencies_are_emitted_first(){let nodes=[1,2,3];let deps=[Dependency{driver_hash:1,required_hash:2},Dependency{driver_hash:2,required_hash:3}];let mut out=[0;3];let n=topological_order(&nodes,&deps,&mut out).unwrap();assert_eq!(n,3);assert_eq!(out,[3,2,1])}#[test]fn cycle_is_rejected(){let nodes=[1,2];let deps=[Dependency{driver_hash:1,required_hash:2},Dependency{driver_hash:2,required_hash:1}];let mut out=[0;2];assert_eq!(topological_order(&nodes,&deps,&mut out),Err(OrderError::Cycle))}}