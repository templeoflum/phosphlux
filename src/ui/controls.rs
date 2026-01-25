//! Control Panel UI
//!
//! Global parameters for CRT emulation effects.
//! All signal generation and processing is now handled by the node graph.

use crate::gpu::renderer::FeedbackUniforms;

/// Global synthesis parameters (CRT effects only)
/// Signal generation is handled by the node graph
#[derive(Clone, Debug)]
pub struct SynthParams {
    /// Feedback parameters (still global for now - will become node-based in Phase 4)
    pub feedback: FeedbackUniforms,

    /// CRT scanline intensity
    pub scanline_intensity: f32,
    /// CRT screen curvature
    pub curvature: f32,
    /// CRT vignette darkening
    pub vignette_amount: f32,
}

impl Default for SynthParams {
    fn default() -> Self {
        Self {
            feedback: FeedbackUniforms::default(),
            scanline_intensity: 0.12,
            curvature: 0.1,
            vignette_amount: 0.4,
        }
    }
}

/// Main control panel (now minimal - most controls are in node graph)
pub struct ControlPanel {
    pub params: SynthParams,
}

impl Default for ControlPanel {
    fn default() -> Self {
        Self {
            params: SynthParams::default(),
        }
    }
}

impl ControlPanel {
    pub fn new() -> Self {
        Self::default()
    }
}
