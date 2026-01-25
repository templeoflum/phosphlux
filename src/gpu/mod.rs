//! GPU module - rendering and shader composition
//!
//! v2 architecture: compose device shaders at compile time

pub mod shader_composer;

pub use shader_composer::{
    ComposedShader, ComposeError, Connection, DeviceInstance, DeviceParameter,
    DevicePatch, Port, ShaderComposer, ShaderDevice, SignalType,
};
