//! Native AWE binary formats.
//!
//! The formats are intentionally dependency-free and `no_std` friendly. Parsing is
//! bounded and validation is performed before any payload is exposed to callers.

pub mod asd;
pub mod awos;
