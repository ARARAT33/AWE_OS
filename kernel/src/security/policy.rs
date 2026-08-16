#![no_std]
use super::{Capability,Impact,Intent};
#[repr(u8)]#[derive(Clone,Copy,PartialEq,Eq,Debug)]pub enum Decision{Allow=0,DenyInvalidIntent=1,DenyCapability=2,DenyImpact=3,DenyBudget=4}
#[derive(Clone,Copy)]pub struct SecurityPolicy{pub max_impact:Impact,pub max_budget:u64}
impl SecurityPolicy{pub const fn strict()->Self{Self{max_impact:Impact::Medium,max_budget:4096}}pub const fn permissive()->Self{Self{max_impact:Impact::Critical,max_budget:u64::MAX}}const fn impact_allowed(&self,impact:Impact)->bool{(impact as u8)<=(self.max_impact as u8)}pub const fn evaluate(&self,capability:Capability,intent:Intent)->Decision{if !intent.is_consistent(){return Decision::DenyInvalidIntent}if !capability.permits(intent.required){return Decision::DenyCapability}if !self.impact_allowed(intent.impact){return Decision::DenyImpact}if intent.expected_resource_units>self.max_budget{return Decision::DenyBudget}Decision::Allow}}
