#![no_std]
use super::bus::DeviceId;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PackageFormat {
    AweDriver,
    LinuxModule,
    AndroidHal,
    WindowsPackage,
    Firmware,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InstallError {
    InvalidPackage,
    DeviceMismatch,
    SignatureRequired,
    UnsupportedFormat,
}
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct InstallerPackage {
    pub format: PackageFormat,
    pub vendor: u16,
    pub device: u16,
    pub version: u32,
    pub signed: bool,
}
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct InstallPlan {
    pub device: DeviceId,
    pub package: InstallerPackage,
    pub requires_adapter: bool,
    pub requires_reboot: bool,
}
pub const fn plan_install(
    device: DeviceId,
    package: InstallerPackage,
) -> Result<InstallPlan, InstallError> {
    if package.vendor != device.vendor || package.device != device.device {
        return Err(InstallError::DeviceMismatch);
    }
    if !package.signed {
        return Err(InstallError::SignatureRequired);
    }
    let requires_adapter = !matches!(package.format, PackageFormat::AweDriver);
    Ok(InstallPlan {
        device,
        package,
        requires_adapter,
        requires_reboot: true,
    })
}
