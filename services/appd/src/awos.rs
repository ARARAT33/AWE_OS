//! Bounded `.awos` native application package lifecycle engine.
//! Provides package validation, publisher cryptographic verification, sandbox
//! isolation policy, dependency graph resolution, repository index management,
//! install, uninstall, update, rollback, and verification.

pub const AWOS_MAGIC: [u8; 4] = *b"AWOS";
pub const AWOS_VERSION: u16 = 1;
pub const AWOS_HEADER_LEN: usize = 32;
pub const AWOS_MAX_MANIFEST: usize = 64 * 1024;
pub const AWOS_MAX_CODE: usize = 256 * 1024 * 1024;
pub const AWOS_MAX_DATA: usize = 256 * 1024 * 1024;
pub const AWOS_MIN_SIGNATURE: usize = 64;
pub const AWOS_FLAG_GUI: u32 = 1 << 0;
pub const AWOS_FLAG_SERVICE: u32 = 1 << 1;
pub const AWOS_KNOWN_FLAGS: u32 = AWOS_FLAG_GUI | AWOS_FLAG_SERVICE;
pub const MAX_PACKAGE_DEPS: usize = 16;
pub const MAX_INDEX_ENTRIES: usize = 32;
pub const MAX_INSTALLED_PACKAGES: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AwosHeader { pub version: u16, pub abi_major: u16, pub abi_minor: u16, pub manifest_len: u32, pub code_len: u32, pub data_len: u32, pub signature_len: u16, pub entry_offset: u32, pub flags: u32 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AwosError { TooShort, BadMagic, UnsupportedVersion, OversizedManifest, OversizedCode, OversizedData, MissingSignature, InvalidLength, InvalidEntry, UnknownFlags, InvalidSignature, PublisherUntrusted, DependencyMissing, DependencyCycle, PackageNotFound, AlreadyInstalled, SandboxViolation, RollbackFailed, StorageFull }

pub fn validate_awos(bytes: &[u8]) -> Result<AwosHeader, AwosError> {
    if bytes.len() < AWOS_HEADER_LEN { return Err(AwosError::TooShort); }
    if bytes[..4] != AWOS_MAGIC { return Err(AwosError::BadMagic); }
    let u16_at = |o: usize| u16::from_le_bytes([bytes[o], bytes[o + 1]]);
    let u32_at = |o: usize| u32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);
    let header = AwosHeader { version:u16_at(4),abi_major:u16_at(6),abi_minor:u16_at(8),manifest_len:u32_at(10),code_len:u32_at(14),data_len:u32_at(18),signature_len:u16_at(22),entry_offset:u32_at(24),flags:u32_at(28) };
    if header.version != AWOS_VERSION { return Err(AwosError::UnsupportedVersion); }
    if header.manifest_len as usize > AWOS_MAX_MANIFEST { return Err(AwosError::OversizedManifest); }
    if header.code_len as usize > AWOS_MAX_CODE { return Err(AwosError::OversizedCode); }
    if header.data_len as usize > AWOS_MAX_DATA { return Err(AwosError::OversizedData); }
    if (header.signature_len as usize) < AWOS_MIN_SIGNATURE { return Err(AwosError::MissingSignature); }
    if header.flags & !AWOS_KNOWN_FLAGS != 0 { return Err(AwosError::UnknownFlags); }
    if header.entry_offset >= header.code_len || header.code_len == 0 { return Err(AwosError::InvalidEntry); }
    let expected = AWOS_HEADER_LEN.checked_add(header.manifest_len as usize).and_then(|v|v.checked_add(header.code_len as usize)).and_then(|v|v.checked_add(header.data_len as usize)).and_then(|v|v.checked_add(header.signature_len as usize)).ok_or(AwosError::InvalidLength)?;
    if expected != bytes.len() { return Err(AwosError::InvalidLength); }
    Ok(header)
}

pub fn package_parts<'a>(bytes:&'a [u8],header:AwosHeader)->Result<(&'a [u8],&'a [u8],&'a [u8],&'a [u8]),AwosError>{let ms=AWOS_HEADER_LEN;let cs=ms.checked_add(header.manifest_len as usize).ok_or(AwosError::InvalidLength)?;let ds=cs.checked_add(header.code_len as usize).ok_or(AwosError::InvalidLength)?;let ss=ds.checked_add(header.data_len as usize).ok_or(AwosError::InvalidLength)?;let end=ss.checked_add(header.signature_len as usize).ok_or(AwosError::InvalidLength)?;if end!=bytes.len(){return Err(AwosError::InvalidLength)}Ok((&bytes[ms..cs],&bytes[cs..ds],&bytes[ds..ss],&bytes[ss..end]))}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublisherIdentity { pub publisher_id:u64,pub public_key:[u8;32],pub is_official:bool }
impl PublisherIdentity{pub fn verify_signature(&self,payload:&[u8],signature:&[u8])->Result<(),AwosError>{if signature.len()<AWOS_MIN_SIGNATURE{return Err(AwosError::MissingSignature)}let sig:&[u8;64]=signature[..64].try_into().map_err(|_|AwosError::MissingSignature)?;if awe_securityd::ed25519_verify(&self.public_key,payload,sig){Ok(())}else{Err(AwosError::InvalidSignature)}}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct SandboxProfile{pub package_id:u64,pub capability_mask:u64,pub max_memory_pages:u32,pub max_fds:u32,pub allow_raw_sockets:bool}
impl SandboxProfile{pub const fn strict_default(package_id:u64)->Self{Self{package_id,capability_mask:0x0003,max_memory_pages:1024,max_fds:16,allow_raw_sockets:false}}pub fn validate_access(&self,required_cap:u64,pages_requested:u32)->Result<(),AwosError>{if self.capability_mask&required_cap!=required_cap||pages_requested>self.max_memory_pages{return Err(AwosError::SandboxViolation)}Ok(())}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct PackageDependency{pub dep_package_id:u64,pub min_version:u16}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct PackageMeta{pub package_id:u64,pub version:u16,pub publisher:PublisherIdentity,pub sandbox:SandboxProfile,pub dependencies:[Option<PackageDependency>;MAX_PACKAGE_DEPS],pub dep_count:usize}
pub struct RepositoryIndex{entries:[Option<PackageMeta>;MAX_INDEX_ENTRIES]}
impl RepositoryIndex{pub const fn new()->Self{Self{entries:[None;MAX_INDEX_ENTRIES]}}pub fn register(&mut self,meta:PackageMeta)->Result<(),AwosError>{for e in self.entries.iter_mut().flatten(){if e.package_id==meta.package_id&&e.version==meta.version{*e=meta;return Ok(())}}for s in self.entries.iter_mut(){if s.is_none(){*s=Some(meta);return Ok(())}}Err(AwosError::StorageFull)}pub fn find(&self,id:u64)->Option<PackageMeta>{self.entries.iter().flatten().find(|e|e.package_id==id).copied()}pub fn resolve_dependencies(&self,root:u64)->Result<[u64;MAX_PACKAGE_DEPS],AwosError>{let mut out=[0;MAX_PACKAGE_DEPS];let mut count=0;let mut stack=[0;MAX_PACKAGE_DEPS];let mut top=1;stack[0]=root;while top>0{top-=1;let id=stack[top];let meta=self.find(id).ok_or(AwosError::DependencyMissing)?;if out[..count].contains(&id){continue}if count>=MAX_PACKAGE_DEPS{return Err(AwosError::StorageFull)}out[count]=id;count+=1;for dep in meta.dependencies.iter().flatten(){if top>=MAX_PACKAGE_DEPS{return Err(AwosError::DependencyCycle)}stack[top]=dep.dep_package_id;top+=1}}}Ok(out)}}
impl Default for RepositoryIndex{fn default()->Self{Self::new()}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum AppPackageState{Installed,Running,Staged,Failed,Quarantined,Removed}
pub const fn package_transition(from:AppPackageState,to:AppPackageState)->bool{matches!((from,to),(AppPackageState::Installed,AppPackageState::Running)|(AppPackageState::Installed,AppPackageState::Staged)|(AppPackageState::Running,AppPackageState::Staged)|(AppPackageState::Running,AppPackageState::Failed)|(AppPackageState::Staged,AppPackageState::Running)|(AppPackageState::Staged,AppPackageState::Failed)|(AppPackageState::Failed,AppPackageState::Staged)|(AppPackageState::Failed,AppPackageState::Quarantined)|(AppPackageState::Installed,AppPackageState::Removed))}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct InstalledAppRecord{pub meta:PackageMeta,pub state:AppPackageState,pub active_version:u16,pub backup_version:Option<u16>}
pub struct AppPackageManager{pub repository:RepositoryIndex,installed:[Option<InstalledAppRecord>;MAX_INSTALLED_PACKAGES]}
impl AppPackageManager{pub const fn new()->Self{Self{repository:RepositoryIndex::new(),installed:[None;MAX_INSTALLED_PACKAGES]}}fn verify_package(&self,bytes:&[u8],meta:PackageMeta)->Result<AwosHeader,AwosError>{if meta.publisher.publisher_id==0||!meta.publisher.is_official{return Err(AwosError::PublisherUntrusted)}let h=validate_awos(bytes)?;let(_,_,_,sig)=package_parts(bytes,h)?;let end=bytes.len().checked_sub(sig.len()).ok_or(AwosError::InvalidLength)?;meta.publisher.verify_signature(&bytes[..end],sig)?;Ok(h)}pub fn install_package(&mut self,bytes:&[u8],meta:PackageMeta)->Result<u64,AwosError>{self.verify_package(bytes,meta)?;if self.installed.iter().flatten().any(|r|r.meta.package_id==meta.package_id){return Err(AwosError::AlreadyInstalled)}self.repository.register(meta)?;self.repository.resolve_dependencies(meta.package_id)?;for s in self.installed.iter_mut(){if s.is_none(){*s=Some(InstalledAppRecord{meta,state:AppPackageState::Installed,active_version:meta.version,backup_version:None});return Ok(meta.package_id)}}Err(AwosError::StorageFull)}pub fn uninstall_package(&mut self,id:u64)->Result<(),AwosError>{for s in self.installed.iter_mut(){if let Some(r)=s&&r.meta.package_id==id{if !package_transition(r.state,AppPackageState::Removed){return Err(AwosError::SandboxViolation)}*s=None;return Ok(())}}Err(AwosError::PackageNotFound)}pub fn update_package(&mut self,meta:PackageMeta,bytes:&[u8])->Result<(),AwosError>{self.verify_package(bytes,meta)?;for s in self.installed.iter_mut().flatten(){if s.meta.package_id==meta.package_id{if meta.version<=s.active_version{return Err(AwosError::InvalidEntry)}let old=s.active_version;s.backup_version=Some(old);s.active_version=meta.version;s.meta=meta;s.state=AppPackageState::Installed;self.repository.register(meta)?;return Ok(())}}Err(AwosError::PackageNotFound)}pub fn rollback_package(&mut self,id:u64)->Result<u16,AwosError>{for s in self.installed.iter_mut().flatten(){if s.meta.package_id==id{if let Some(v)=s.backup_version{s.active_version=v;s.backup_version=None;s.state=AppPackageState::Installed;return Ok(v)}return Err(AwosError::RollbackFailed)}}Err(AwosError::PackageNotFound)}pub fn get_installed_record(&self,id:u64)->Result<InstalledAppRecord,AwosError>{self.installed.iter().flatten().find(|r|r.meta.package_id==id).copied().ok_or(AwosError::PackageNotFound)}}
impl Default for AppPackageManager{fn default()->Self{Self::new()}}

#[cfg(test)]
mod tests{use super::*;extern crate std;use std::vec::Vec;fn bytes(m:usize,c:usize,d:usize)->Vec<u8>{let mut b=Vec::new();b.extend_from_slice(&AWOS_MAGIC);b.extend_from_slice(&AWOS_VERSION.to_le_bytes());b.extend_from_slice(&1u16.to_le_bytes());b.extend_from_slice(&0u16.to_le_bytes());b.extend_from_slice(&(m as u32).to_le_bytes());b.extend_from_slice(&(c as u32).to_le_bytes());b.extend_from_slice(&(d as u32).to_le_bytes());b.extend_from_slice(&64u16.to_le_bytes());b.extend_from_slice(&0u32.to_le_bytes());b.extend_from_slice(&0u32.to_le_bytes());b.extend(core::iter::repeat_n(0u8,m));b.extend(core::iter::repeat_n(0x90u8,c));b.extend(core::iter::repeat_n(0u8,d));b.extend(core::iter::repeat_n(0u8,64));b}#[test]fn signature_binds_full_payload(){let seed=[7u8;32];let(pk,sk)=awe_securityd::ed25519_keypair_from_seed(&seed);let p=PublisherIdentity{publisher_id:1,public_key:pk,is_official:true};let mut b=bytes(4,8,2);let n=b.len()-64;let sig=awe_securityd::ed25519_sign(&sk,&b[..n]);b[n..].copy_from_slice(&sig);let meta=PackageMeta{package_id:1,version:1,publisher:p,sandbox:SandboxProfile::strict_default(1),dependencies:[None;MAX_PACKAGE_DEPS],dep_count:0};let mut m=AppPackageManager::new();m.install_package(&b,meta).unwrap();assert_eq!(m.get_installed_record(1).unwrap().active_version,1);b[33]^=1;assert_eq!(m.update_package(meta,&b),Err(AwosError::InvalidSignature))}}
