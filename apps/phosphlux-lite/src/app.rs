//! Application state management

use crate::presets::{builtin_presets, Preset};
use crate::synth::SynthState;

/// Which stage panel is currently selected in the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedStage {
    Input,
    Geometry,
    Amplitude,
    Colorize,
    Mixer,
    Feedback,
    Output,
}

impl Default for SelectedStage {
    fn default() -> Self {
        Self::Input
    }
}

/// Main application state
pub struct App {
    /// Current synthesizer state
    pub synth: SynthState,

    /// Currently selected stage panel
    pub selected_stage: SelectedStage,

    /// Available presets (built-in + user)
    pub presets: Vec<Preset>,

    /// Currently selected preset index (None = modified/custom)
    pub current_preset: Option<usize>,

    /// Frame counter
    pub frame: u32,

    /// Time accumulator
    pub time: f32,

    /// Show preset browser
    pub show_preset_browser: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            synth: SynthState::default(),
            selected_stage: SelectedStage::Input,
            presets: builtin_presets(),
            current_preset: None,
            frame: 0,
            time: 0.0,
            show_preset_browser: false,
        }
    }

    /// Update timing
    pub fn update(&mut self, dt: f32) {
        self.time += dt;
        self.frame = self.frame.wrapping_add(1);
    }

    /// Load a preset by index
    pub fn load_preset(&mut self, index: usize) {
        if let Some(preset) = self.presets.get(index) {
            self.synth = preset.state.clone();
            self.current_preset = Some(index);
        }
    }

    /// Mark state as modified (no longer matches a preset)
    pub fn mark_modified(&mut self) {
        self.current_preset = None;
    }

    /// Randomize all parameters
    pub fn randomize(&mut self) {
        use crate::synth::*;

        let rand = || -> f32 {
            // Simple LCG random - not cryptographic but fine for this
            static mut SEED: u32 = 12345;
            unsafe {
                SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
                (SEED as f32) / (u32::MAX as f32)
            }
        };

        let rand_range = |min: f32, max: f32| min + rand() * (max - min);
        let rand_int = |max: u32| (rand() * max as f32) as u32;

        // Input stage
        self.synth.input.source_a = match rand_int(11) {
            0 => InputSource::RampH,
            1 => InputSource::RampV,
            2 => InputSource::OscH,
            3 => InputSource::OscV,
            4 => InputSource::NoiseWhite,
            5 => InputSource::NoisePink,
            6 => InputSource::NoiseBrown,
            7 => InputSource::ShapeCircle,
            8 => InputSource::ShapeRect,
            9 => InputSource::ShapeDiamond,
            _ => InputSource::Checkerboard,
        };
        self.synth.input.source_b = match rand_int(11) {
            0 => InputSource::RampH,
            1 => InputSource::RampV,
            2 => InputSource::OscH,
            3 => InputSource::OscV,
            4 => InputSource::NoiseWhite,
            5 => InputSource::NoisePink,
            6 => InputSource::NoiseBrown,
            7 => InputSource::ShapeCircle,
            8 => InputSource::ShapeRect,
            9 => InputSource::ShapeDiamond,
            _ => InputSource::Checkerboard,
        };
        self.synth.input.mix = rand();
        self.synth.input.frequency = rand_range(1.0, 12.0);
        self.synth.input.phase = rand();
        self.synth.input.rotation = rand();

        // Geometry - be conservative to avoid chaos
        self.synth.geometry.wobbulate_h = rand_range(0.0, 0.3);
        self.synth.geometry.wobbulate_v = rand_range(0.0, 0.3);
        self.synth.geometry.wobble_freq = rand_range(2.0, 10.0);
        self.synth.geometry.z_displacement = rand_range(0.0, 0.2);
        self.synth.geometry.lissajous_x = rand_range(0.0, 0.3);
        self.synth.geometry.lissajous_y = rand_range(0.0, 0.3);
        self.synth.geometry.rotation = rand_range(0.0, 0.1);
        self.synth.geometry.scale = rand_range(0.8, 1.2);

        // Amplitude
        self.synth.amplitude.fold_gain = rand_range(1.0, 4.0);
        self.synth.amplitude.fold_mix = rand();
        self.synth.amplitude.quantize_levels = rand_range(4.0, 16.0);
        self.synth.amplitude.quantize_mix = rand();
        self.synth.amplitude.soft_clip = rand_range(0.0, 0.5);
        self.synth.amplitude.solarize = rand_range(0.5, 1.0);
        self.synth.amplitude.gate_threshold = rand_range(0.0, 0.3);
        self.synth.amplitude.invert = if rand() > 0.8 { 1.0 } else { 0.0 };

        // Colorize
        self.synth.colorize.mode = match rand_int(4) {
            0 => ColorMode::Spectrum,
            1 => ColorMode::Threshold,
            2 => ColorMode::Gradient,
            _ => ColorMode::Monochrome,
        };
        self.synth.colorize.hue_offset = rand();
        self.synth.colorize.saturation = rand_range(0.5, 1.5);
        self.synth.colorize.levels = rand_range(4.0, 16.0);
        self.synth.colorize.gradient_start = [rand(), rand(), rand()];
        self.synth.colorize.gradient_end = [rand(), rand(), rand()];

        // Mixer
        self.synth.mixer.feedback_mix = rand_range(0.2, 0.8);
        self.synth.mixer.blend_mode = match rand_int(8) {
            0 => BlendMode::Mix,
            1 => BlendMode::Add,
            2 => BlendMode::Multiply,
            3 => BlendMode::Screen,
            4 => BlendMode::Overlay,
            5 => BlendMode::Difference,
            6 => BlendMode::LumaKeyA,
            _ => BlendMode::LumaKeyB,
        };
        self.synth.mixer.key_threshold = rand_range(0.3, 0.7);
        self.synth.mixer.key_softness = rand_range(0.05, 0.2);
        self.synth.mixer.key_invert = rand() > 0.5;
        self.synth.mixer.layer_opacity = rand_range(0.7, 1.0);

        // Feedback - keep it stable
        self.synth.feedback.enabled = true;
        self.synth.feedback.zoom = rand_range(0.98, 1.05);
        self.synth.feedback.rotation = rand_range(-0.05, 0.05);
        self.synth.feedback.hue_shift = rand_range(0.0, 0.03);
        self.synth.feedback.decay = rand_range(0.9, 0.98);
        self.synth.feedback.offset_x = rand_range(-0.02, 0.02);
        self.synth.feedback.offset_y = rand_range(-0.02, 0.02);
        self.synth.feedback.saturation = rand_range(0.8, 1.2);

        // Output - pick a mode
        self.synth.output.mode = match rand_int(4) {
            0 => OutputMode::Clean,
            1 => OutputMode::CRT,
            2 => OutputMode::VHS,
            _ => OutputMode::Cable,
        };
        self.synth.output.scanlines = rand_range(0.0, 0.25);
        self.synth.output.curvature = rand_range(0.0, 0.2);
        self.synth.output.bloom = rand_range(0.1, 0.4);
        self.synth.output.vignette = rand_range(0.1, 0.4);
        self.synth.output.noise = rand_range(0.0, 0.1);
        self.synth.output.tracking = rand_range(0.0, 0.3);
        self.synth.output.chroma_shift = rand_range(0.0, 0.01);
        self.synth.output.tape_wobble = rand_range(0.0, 0.3);
        self.synth.output.bandwidth = rand_range(0.7, 1.0);
        self.synth.output.ghosting = rand_range(0.0, 0.15);

        self.mark_modified();
    }
}
