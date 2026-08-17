#![no_std]

//! APIC/IOAPIC discovery and routing model. Hardware register access remains
//! behind the driverd platform backend; this module owns validation/state only.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApicError {
    InvalidVector,
    InvalidGsi,
    InvalidDestination,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalApic {
    pub base: u64,
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IoApic {
    pub id: u8,
    pub base: u64,
    pub gsi_base: u32,
    pub redirection_entries: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IrqRoute {
    pub gsi: u32,
    pub vector: u8,
    pub destination_apic: u8,
    pub masked: bool,
}

impl IrqRoute {
    pub const fn new(gsi: u32, vector: u8, destination_apic: u8) -> Result<Self, ApicError> {
        if vector < 32 { return Err(ApicError::InvalidVector); }
        if destination_apic > 255 { return Err(ApicError::InvalidDestination); }
        Ok(Self { gsi, vector, destination_apic, masked: true })
    }

    pub const fn unmask(mut self) -> Self {
        self.masked = false;
        self
    }
}

impl IoApic {
    pub const fn owns_gsi(self, gsi: u32) -> bool {
        gsi >= self.gsi_base && gsi < self.gsi_base + self.redirection_entries as u32
    }

    pub const fn route(self, gsi: u32, vector: u8, destination: u8) -> Result<IrqRoute, ApicError> {
        if !self.owns_gsi(gsi) { return Err(ApicError::InvalidGsi); }
        IrqRoute::new(gsi, vector, destination)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ioapic_gsi_range_is_bounded() {
        let io = IoApic { id: 1, base: 0xFEC0_0000, gsi_base: 32, redirection_entries: 24 };
        assert!(io.owns_gsi(32));
        assert!(io.owns_gsi(55));
        assert!(!io.owns_gsi(56));
    }

    #[test]
    fn irq_vector_below_exception_range_is_rejected() {
        assert_eq!(IrqRoute::new(1, 31, 0), Err(ApicError::InvalidVector));
        assert!(IrqRoute::new(1, 32, 0).is_ok());
    }
}
