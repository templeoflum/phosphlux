//! Core module - fundamental types and timing
//!
//! This module provides the foundation for authentic analog circuit emulation:
//! - Signal types with voltage/impedance characteristics
//! - Component models (op-amps, transistors, passives)
//! - NTSC/PAL timing generation

pub mod component;
pub mod signal;
pub mod timing;

pub use component::{BendPoint, BendType, BjTransistor, Diode, OpAmp, RcFilter, Tolerance};
pub use signal::{AnalogSignal, Impedance, ProcessingRate, RgbSignal, VoltageRange, YiqSignal};
pub use timing::{SyncGenerator, TimingUniforms, VideoStandard};
