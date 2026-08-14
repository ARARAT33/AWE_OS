#![no_std]

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BootError {
    InvalidMagic = 1,
    InvalidVersion = 2,
    UnsupportedArchitecture = 3,
    InvalidBounds = 4,
    InvalidMemoryMap = 5,
    ForeignOperatingSystem = 6,
    UnsignedImage = 7,
    BadSignature = 8,
    RollbackDetected = 9,
    InvalidKernel = 10,
    InvalidEntry = 11,
    UnsupportedProtocol = 12,
}

pub trait ErrorSink {
    fn report(&mut self, error: BootError);
}

pub fn stop<E: ErrorSink>(sink: &mut E, error: BootError) -> ! {
    sink.report(error);
    loop {
        core::hint::spin_loop();
    }
}
