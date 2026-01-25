//! Synthesis Modules
//!
//! Each module corresponds to a hardware module in an analog video synthesizer.
//! Currently, modules are implemented as shader functions.
//! Future: modular patching system with CPU-side parameter control.

// Module types (for future modular routing system)
pub enum ModuleType {
    // Oscillators - signal sources
    Ramp,
    VideoOscillator,
    NoiseGenerator,

    // Waveshapers - signal processors
    Folder,
    SoftClipper,
    Quantizer,
    Solarizer,

    // Colorizers - luma to RGB
    SpectrumColorizer,
    ThresholdColorizer,
    MatrixColorizer,

    // Effects
    RuttEtra,
    Wobbulator,
    Feedback,

    // Mixing
    Keyer,
    Multiplier,
    Crossfader,
}
