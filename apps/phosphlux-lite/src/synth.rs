//! Synthesizer stage parameters
//!
//! Defines the 7-stage fixed signal chain:
//! INPUT -> GEOMETRY -> AMPLITUDE -> COLORIZE -> MIXER -> FEEDBACK -> OUTPUT

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// Input source types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum InputSource {
    RampH = 0,
    RampV = 1,
    OscH = 2,
    OscV = 3,
    NoiseWhite = 4,
    NoisePink = 5,
    NoiseBrown = 6,
    ShapeCircle = 7,
    ShapeRect = 8,
    ShapeDiamond = 9,
    Checkerboard = 10,
}

impl Default for InputSource {
    fn default() -> Self {
        Self::RampH
    }
}

/// Colorize mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum ColorMode {
    Spectrum = 0,
    Threshold = 1,
    Gradient = 2,
    Monochrome = 3,
}

impl Default for ColorMode {
    fn default() -> Self {
        Self::Spectrum
    }
}

/// Blend mode for mixer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum BlendMode {
    Mix = 0,
    Add = 1,
    Multiply = 2,
    Screen = 3,
    Overlay = 4,
    Difference = 5,
    LumaKeyA = 6,
    LumaKeyB = 7,
}

impl Default for BlendMode {
    fn default() -> Self {
        Self::Mix
    }
}

/// Output emulation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum OutputMode {
    Clean = 0,
    CRT = 1,
    VHS = 2,
    Cable = 3,
}

impl Default for OutputMode {
    fn default() -> Self {
        Self::CRT
    }
}

/// Stage 1: Input Matrix
/// Mix and combine signal sources
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct InputStage {
    pub source_a: InputSource,
    pub source_b: InputSource,
    pub mix: f32,           // 0-1 blend between A and B
    pub frequency: f32,     // 0.5-20 cycles
    pub phase: f32,         // 0-1 phase offset
    pub rotation: f32,      // 0-1 (maps to 0-2pi)
}

impl Default for InputStage {
    fn default() -> Self {
        Self {
            source_a: InputSource::RampH,
            source_b: InputSource::OscV,
            mix: 0.0,
            frequency: 4.0,
            phase: 0.0,
            rotation: 0.0,
        }
    }
}

/// Stage 2: Geometry
/// Spatial distortions - Wobbulate, Z-displacement, Lissajous
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GeometryStage {
    pub wobbulate_h: f32,   // 0-1 horizontal wobble amount
    pub wobbulate_v: f32,   // 0-1 vertical wobble amount
    pub wobble_freq: f32,   // 1-20 wobble frequency
    pub z_displacement: f32, // 0-0.5 Rutt/Etra style displacement
    pub lissajous_x: f32,   // 0-1 lissajous X modulation
    pub lissajous_y: f32,   // 0-1 lissajous Y modulation
    pub rotation: f32,      // 0-1 (maps to 0-2pi)
    pub scale: f32,         // 0.5-2.0
}

impl Default for GeometryStage {
    fn default() -> Self {
        Self {
            wobbulate_h: 0.0,
            wobbulate_v: 0.0,
            wobble_freq: 4.0,
            z_displacement: 0.0,
            lissajous_x: 0.0,
            lissajous_y: 0.0,
            rotation: 0.0,
            scale: 1.0,
        }
    }
}

/// Stage 3: Amplitude
/// Waveform shaping - fold, quantize, clip, solarize
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AmplitudeStage {
    pub fold_gain: f32,     // 1-8 folding intensity
    pub fold_mix: f32,      // 0-1 dry/wet
    pub quantize_levels: f32, // 2-32 quantization levels
    pub quantize_mix: f32,  // 0-1 dry/wet
    pub soft_clip: f32,     // 0-1 soft clipping amount
    pub solarize: f32,      // 0-1 solarize threshold
    pub gate_threshold: f32, // 0-1 hard gate
    pub invert: f32,        // 0 or 1
}

impl Default for AmplitudeStage {
    fn default() -> Self {
        Self {
            fold_gain: 1.0,
            fold_mix: 0.0,
            quantize_levels: 8.0,
            quantize_mix: 0.0,
            soft_clip: 0.0,
            solarize: 1.0, // 1.0 = off (threshold above max)
            gate_threshold: 0.0,
            invert: 0.0,
        }
    }
}

/// Stage 4: Colorize
/// Luminance to color mapping
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ColorizeStage {
    pub mode: ColorMode,
    pub hue_offset: f32,    // 0-1 hue rotation
    pub saturation: f32,    // 0-2 saturation multiplier
    pub levels: f32,        // 2-32 for threshold mode
    pub gradient_start: [f32; 3], // RGB start color
    pub gradient_end: [f32; 3],   // RGB end color
}

impl Default for ColorizeStage {
    fn default() -> Self {
        Self {
            mode: ColorMode::Spectrum,
            hue_offset: 0.0,
            saturation: 1.0,
            levels: 8.0,
            gradient_start: [0.0, 0.0, 0.0],
            gradient_end: [1.0, 1.0, 1.0],
        }
    }
}

/// Stage 5: Mixer
/// Blend with feedback, keying
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MixerStage {
    pub feedback_mix: f32,  // 0-1 feedback amount
    pub blend_mode: BlendMode,
    pub key_threshold: f32, // 0-1 for keying modes
    pub key_softness: f32,  // 0-0.5 key edge softness
    pub key_invert: bool,
    pub layer_opacity: f32, // 0-1 overall layer opacity
}

impl Default for MixerStage {
    fn default() -> Self {
        Self {
            feedback_mix: 0.3,
            blend_mode: BlendMode::Mix,
            key_threshold: 0.5,
            key_softness: 0.1,
            key_invert: false,
            layer_opacity: 1.0,
        }
    }
}

/// Stage 6: Feedback
/// Temporal effects - zoom, rotate, decay
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FeedbackStage {
    pub enabled: bool,
    pub zoom: f32,          // 0.9-1.1 feedback zoom
    pub rotation: f32,      // 0-1 (maps to +/- pi/8)
    pub hue_shift: f32,     // 0-1 hue rotation per frame
    pub decay: f32,         // 0.8-1.0 brightness decay
    pub offset_x: f32,      // -0.1 to 0.1 horizontal drift
    pub offset_y: f32,      // -0.1 to 0.1 vertical drift
    pub saturation: f32,    // 0-2 saturation adjustment
}

impl Default for FeedbackStage {
    fn default() -> Self {
        Self {
            enabled: true,
            zoom: 1.02,
            rotation: 0.0,
            hue_shift: 0.0,
            decay: 0.95,
            offset_x: 0.0,
            offset_y: 0.0,
            saturation: 1.0,
        }
    }
}

/// Stage 7: Output
/// Display emulation - CRT, VHS, cable degradation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OutputStage {
    pub mode: OutputMode,
    pub scanlines: f32,     // 0-1 scanline intensity
    pub curvature: f32,     // 0-0.5 barrel distortion
    pub bloom: f32,         // 0-1 phosphor bloom
    pub vignette: f32,      // 0-1 edge darkening
    pub noise: f32,         // 0-0.5 signal noise
    // VHS specific
    pub tracking: f32,      // 0-1 tracking error amount
    pub chroma_shift: f32,  // 0-0.02 chroma/luma separation
    pub tape_wobble: f32,   // 0-1 horizontal instability
    // Cable specific
    pub bandwidth: f32,     // 0.5-1.0 bandwidth limiting
    pub ghosting: f32,      // 0-0.3 RF ghosting
}

impl Default for OutputStage {
    fn default() -> Self {
        Self {
            mode: OutputMode::CRT,
            scanlines: 0.15,
            curvature: 0.1,
            bloom: 0.2,
            vignette: 0.3,
            noise: 0.02,
            tracking: 0.0,
            chroma_shift: 0.0,
            tape_wobble: 0.0,
            bandwidth: 1.0,
            ghosting: 0.0,
        }
    }
}

/// Complete synthesizer state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthState {
    pub input: InputStage,
    pub geometry: GeometryStage,
    pub amplitude: AmplitudeStage,
    pub colorize: ColorizeStage,
    pub mixer: MixerStage,
    pub feedback: FeedbackStage,
    pub output: OutputStage,
}

impl Default for SynthState {
    fn default() -> Self {
        Self {
            input: InputStage::default(),
            geometry: GeometryStage::default(),
            amplitude: AmplitudeStage::default(),
            colorize: ColorizeStage::default(),
            mixer: MixerStage::default(),
            feedback: FeedbackStage::default(),
            output: OutputStage::default(),
        }
    }
}

/// GPU-friendly packed uniforms (256 bytes)
/// Aligned to 16-byte boundaries for GPU
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct SynthUniforms {
    // Input stage (16 bytes)
    pub input_source_a: u32,
    pub input_source_b: u32,
    pub input_mix: f32,
    pub input_frequency: f32,

    // Input continued (16 bytes)
    pub input_phase: f32,
    pub input_rotation: f32,
    pub _pad0: f32,
    pub _pad1: f32,

    // Geometry stage (32 bytes)
    pub geo_wobbulate_h: f32,
    pub geo_wobbulate_v: f32,
    pub geo_wobble_freq: f32,
    pub geo_z_displacement: f32,

    pub geo_lissajous_x: f32,
    pub geo_lissajous_y: f32,
    pub geo_rotation: f32,
    pub geo_scale: f32,

    // Amplitude stage (32 bytes)
    pub amp_fold_gain: f32,
    pub amp_fold_mix: f32,
    pub amp_quantize_levels: f32,
    pub amp_quantize_mix: f32,

    pub amp_soft_clip: f32,
    pub amp_solarize: f32,
    pub amp_gate_threshold: f32,
    pub amp_invert: f32,

    // Colorize stage (32 bytes)
    pub color_mode: u32,
    pub color_hue_offset: f32,
    pub color_saturation: f32,
    pub color_levels: f32,

    pub color_gradient_start: [f32; 3],
    pub _pad2: f32,

    // Colorize gradient end (16 bytes)
    pub color_gradient_end: [f32; 3],
    pub _pad3: f32,

    // Mixer stage (16 bytes)
    pub mixer_feedback_mix: f32,
    pub mixer_blend_mode: u32,
    pub mixer_key_threshold: f32,
    pub mixer_key_softness: f32,

    // Mixer continued (16 bytes)
    pub mixer_key_invert: f32,
    pub mixer_layer_opacity: f32,
    pub _pad4: f32,
    pub _pad5: f32,

    // Feedback stage (32 bytes)
    pub fb_enabled: f32,
    pub fb_zoom: f32,
    pub fb_rotation: f32,
    pub fb_hue_shift: f32,

    pub fb_decay: f32,
    pub fb_offset_x: f32,
    pub fb_offset_y: f32,
    pub fb_saturation: f32,

    // Output stage (32 bytes)
    pub out_mode: u32,
    pub out_scanlines: f32,
    pub out_curvature: f32,
    pub out_bloom: f32,

    pub out_vignette: f32,
    pub out_noise: f32,
    pub out_tracking: f32,
    pub out_chroma_shift: f32,

    // Output continued (16 bytes)
    pub out_tape_wobble: f32,
    pub out_bandwidth: f32,
    pub out_ghosting: f32,
    pub _pad6: f32,

    // Timing (16 bytes)
    pub time: f32,
    pub frame: u32,
    pub _pad7: f32,
    pub _pad8: f32,
}

impl SynthUniforms {
    pub fn from_state(state: &SynthState, time: f32, frame: u32) -> Self {
        Self {
            // Input
            input_source_a: state.input.source_a as u32,
            input_source_b: state.input.source_b as u32,
            input_mix: state.input.mix,
            input_frequency: state.input.frequency,
            input_phase: state.input.phase,
            input_rotation: state.input.rotation,
            _pad0: 0.0,
            _pad1: 0.0,

            // Geometry
            geo_wobbulate_h: state.geometry.wobbulate_h,
            geo_wobbulate_v: state.geometry.wobbulate_v,
            geo_wobble_freq: state.geometry.wobble_freq,
            geo_z_displacement: state.geometry.z_displacement,
            geo_lissajous_x: state.geometry.lissajous_x,
            geo_lissajous_y: state.geometry.lissajous_y,
            geo_rotation: state.geometry.rotation,
            geo_scale: state.geometry.scale,

            // Amplitude
            amp_fold_gain: state.amplitude.fold_gain,
            amp_fold_mix: state.amplitude.fold_mix,
            amp_quantize_levels: state.amplitude.quantize_levels,
            amp_quantize_mix: state.amplitude.quantize_mix,
            amp_soft_clip: state.amplitude.soft_clip,
            amp_solarize: state.amplitude.solarize,
            amp_gate_threshold: state.amplitude.gate_threshold,
            amp_invert: state.amplitude.invert,

            // Colorize
            color_mode: state.colorize.mode as u32,
            color_hue_offset: state.colorize.hue_offset,
            color_saturation: state.colorize.saturation,
            color_levels: state.colorize.levels,
            color_gradient_start: state.colorize.gradient_start,
            _pad2: 0.0,
            color_gradient_end: state.colorize.gradient_end,
            _pad3: 0.0,

            // Mixer
            mixer_feedback_mix: state.mixer.feedback_mix,
            mixer_blend_mode: state.mixer.blend_mode as u32,
            mixer_key_threshold: state.mixer.key_threshold,
            mixer_key_softness: state.mixer.key_softness,
            mixer_key_invert: if state.mixer.key_invert { 1.0 } else { 0.0 },
            mixer_layer_opacity: state.mixer.layer_opacity,
            _pad4: 0.0,
            _pad5: 0.0,

            // Feedback
            fb_enabled: if state.feedback.enabled { 1.0 } else { 0.0 },
            fb_zoom: state.feedback.zoom,
            fb_rotation: state.feedback.rotation,
            fb_hue_shift: state.feedback.hue_shift,
            fb_decay: state.feedback.decay,
            fb_offset_x: state.feedback.offset_x,
            fb_offset_y: state.feedback.offset_y,
            fb_saturation: state.feedback.saturation,

            // Output
            out_mode: state.output.mode as u32,
            out_scanlines: state.output.scanlines,
            out_curvature: state.output.curvature,
            out_bloom: state.output.bloom,
            out_vignette: state.output.vignette,
            out_noise: state.output.noise,
            out_tracking: state.output.tracking,
            out_chroma_shift: state.output.chroma_shift,
            out_tape_wobble: state.output.tape_wobble,
            out_bandwidth: state.output.bandwidth,
            out_ghosting: state.output.ghosting,
            _pad6: 0.0,

            // Timing
            time,
            frame,
            _pad7: 0.0,
            _pad8: 0.0,
        }
    }
}
