//! GPU module - rendering and shader composition
//!
//! v2 architecture: compose device shaders at compile time

pub mod renderer;
pub mod shader_composer;

pub use shader_composer::{
    DeviceInstance, DeviceParameter, DevicePatch, ParamLayout, Port, ProcessingMode, RenderPlan,
    SegmentKind, SegmentShader, ShaderComposer, ShaderDevice, SignalType,
};
