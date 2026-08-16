#![no_std]

use super::bus::DeviceId;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DriverOs { Linux, Android, Windows, Bsd, AweNative, Generic }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DriverAbi { Native, LinuxKmod, LinuxUserMode, AndroidHal, WindowsCompat, Generic }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DriverAction { Probe, Bind, Start, Stop, Remove }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DriverError { UnsupportedAbi, InvalidDevice, NotSigned, VersionMismatch, PolicyDenied }

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DriverRequest {
    pub device: DeviceId,
    pub os: DriverOs,
    pub abi: DriverAbi,
    pub action: DriverAction,
    pub version: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DriverResult { pub accepted: bool, pub score: u16, pub error: Option<DriverError> }

/// Universal driver boundary. It intentionally does not execute foreign
/// kernel code: foreign drivers must be adapted into this contract or run in
/// an isolated compatibility runtime.
pub const fn validate_request(request: &DriverRequest) -> DriverResult {
    if request.version == 0 {
        return DriverResult { accepted: false, score: 0, error: Some(DriverError::VersionMismatch) };
    }
    if matches!(request.abi, DriverAbi::Native) && !matches!(request.os, DriverOs::AweNative) {
        return DriverResult { accepted: false, score: 0, error: Some(DriverError::UnsupportedAbi) };
    }
    DriverResult { accepted: true, score: 100, error: None }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn device() -> DeviceId { DeviceId { vendor: 0x8086, device: 0x100e, class: 0x0200, revision: 1 } }
    #[test]
    fn accepts_versioned_compatibility_request() {
        let r = DriverRequest { device: device(), os: DriverOs::Linux, abi: DriverAbi::LinuxUserMode, action: DriverAction::Probe, version: 1 };
        assert!(validate_request(&r).accepted);
    }
    #[test]
    fn rejects_native_abi_for_foreign_os() {
        let r = DriverRequest { device: device(), os: DriverOs::Linux, abi: DriverAbi::Native, action: DriverAction::Probe, version: 1 };
        assert_eq!(validate_request(&r).error, Some(DriverError::UnsupportedAbi));
    }
}
