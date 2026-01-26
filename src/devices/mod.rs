//! Device Modules
//!
//! Each device is an authentic emulation of historical video synthesis hardware.
//! Devices expose their circuit behavior through WGSL shaders and can be
//! composed into patches using the shader composer.
//!
//! Device categories:
//! - **generators**: Test signal generators
//! - **paik_abe**: Paik/Abe Video Synthesizer (1969-1970)
//! - **sandin_ip**: Sandin Image Processor (1973)
//! - **jones**: Dave Jones Colorizers (1970s-1990s)
//! - **rutt_etra**: Rutt/Etra Video Synthesizer (1973)
//! - **lzx**: LZX Industries modules (2010s-present)

pub mod generators;
pub mod paik_abe;
// pub mod sandin_ip;  // TODO
// pub mod jones;       // TODO
// pub mod rutt_etra;   // TODO
// pub mod lzx;         // TODO

pub use generators::TestSignalGenerator;
pub use paik_abe::{PaikAbeColorizer, PaikAbeWobulator};
