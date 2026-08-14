#![no_std]

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DetectedOs {
    AweOs,
    Windows,
    Linux,
    Android,
    Unknown,
}

/// Conservative signatures used only to refuse obvious foreign images.
/// A positive AWEOS manifest remains mandatory; these signatures can never
/// make a foreign image bootable.
pub fn detect(bytes: &[u8]) -> DetectedOs {
    if bytes.len() >= 8 && &bytes[..8] == b"AWEOS001" {
        return DetectedOs::AweOs;
    }
    if bytes.len() >= 2 && bytes[..2] == *b"MZ" {
        return DetectedOs::Windows;
    }
    if bytes.len() >= 4 && &bytes[..4] == b"\x7fELF" {
        return DetectedOs::Unknown;
    }
    if bytes.len() >= 4 && &bytes[..4] == b"ANDROID!" {
        return DetectedOs::Android;
    }
    DetectedOs::Unknown
}

pub fn should_refuse(bytes: &[u8]) -> bool {
    !matches!(detect(bytes), DetectedOs::AweOs)
}
