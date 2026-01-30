//! Master Sync Generator
//!
//! Generates timing signals that drive the entire synthesizer, just like
//! the sync generator in real analog video hardware. All modules lock to
//! these timing signals.
//!
//! NTSC: 525 lines, 59.94 Hz, 15.734 kHz horizontal
//! PAL:  625 lines, 50 Hz, 15.625 kHz horizontal
//!
//! Note: Some methods are currently unused but provide foundational timing
//! calculations for future features (e.g., interlacing, PAL support).

#![allow(dead_code)]

use bytemuck::{Pod, Zeroable};

/// Video timing standard
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VideoStandard {
    NTSC,
    PAL,
}

impl VideoStandard {
    /// Total lines per frame (including blanking)
    pub fn total_lines(&self) -> u32 {
        match self {
            VideoStandard::NTSC => 525,
            VideoStandard::PAL => 625,
        }
    }

    /// Visible lines per frame
    pub fn visible_lines(&self) -> u32 {
        match self {
            VideoStandard::NTSC => 480,
            VideoStandard::PAL => 576,
        }
    }

    /// Field rate in Hz
    pub fn field_rate(&self) -> f32 {
        match self {
            VideoStandard::NTSC => 59.94,
            VideoStandard::PAL => 50.0,
        }
    }

    /// Horizontal line rate in Hz
    pub fn line_rate(&self) -> f32 {
        match self {
            VideoStandard::NTSC => 15_734.264,
            VideoStandard::PAL => 15_625.0,
        }
    }

    /// Color subcarrier frequency (for NTSC artifact emulation)
    pub fn color_subcarrier(&self) -> f32 {
        match self {
            VideoStandard::NTSC => 3_579_545.0, // 3.58 MHz
            VideoStandard::PAL => 4_433_618.75, // 4.43 MHz
        }
    }

    /// Microseconds per line
    pub fn line_time_us(&self) -> f32 {
        1_000_000.0 / self.line_rate()
    }

    /// Active video portion of line (microseconds)
    pub fn active_line_us(&self) -> f32 {
        match self {
            VideoStandard::NTSC => 52.6,
            VideoStandard::PAL => 52.0,
        }
    }
}

/// Timing data passed to GPU shaders (must be 64 bytes for WGSL alignment)
/// This is the "control voltage" that drives all timing-dependent modules
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct TimingUniforms {
    /// Current field number (0 = even, 1 = odd for interlaced)
    pub field: u32,
    /// Total frame count since start
    pub frame: u32,
    /// Normalized time within current frame [0, 1]
    pub frame_phase: f32,
    /// Time in seconds since start
    pub time: f32,

    /// Horizontal ramp: 0 at left edge, 1 at right edge
    pub h_phase: f32,
    /// Vertical ramp: 0 at top, 1 at bottom
    pub v_phase: f32,
    /// Lines per frame (visible)
    pub lines: f32,
    /// Field rate
    pub field_rate: f32,

    /// Color subcarrier phase (for NTSC artifact emulation)
    pub subcarrier_phase: f32,
    /// Padding to reach 64 bytes total (16 x f32)
    pub _pad: [f32; 7],
}

/// Master sync generator
/// Tracks timing state and produces uniforms for GPU
pub struct SyncGenerator {
    standard: VideoStandard,
    start_time: instant::Instant,
    field_count: u64,
    frame_count: u64,
    subcarrier_phase: f64, // High precision accumulator
}

impl SyncGenerator {
    pub fn new(standard: VideoStandard) -> Self {
        Self {
            standard,
            start_time: instant::Instant::now(),
            field_count: 0,
            frame_count: 0,
            subcarrier_phase: 0.0,
        }
    }

    /// Advance by one field (call this at field rate)
    pub fn tick(&mut self) {
        self.field_count += 1;
        if self.field_count % 2 == 0 {
            self.frame_count += 1;
        }

        // Accumulate subcarrier phase
        // This creates the characteristic NTSC color cycling
        let subcarrier_cycles_per_field =
            self.standard.color_subcarrier() / self.standard.field_rate();
        self.subcarrier_phase += subcarrier_cycles_per_field as f64;
        self.subcarrier_phase %= 1_000_000.0; // Prevent overflow
    }

    /// Get current timing uniforms for GPU
    pub fn get_timing(&self) -> TimingUniforms {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let frame_phase = (self.field_count % 2) as f32 * 0.5;

        TimingUniforms {
            field: (self.field_count % 2) as u32,
            frame: self.frame_count as u32,
            frame_phase,
            time: elapsed,
            h_phase: 0.0, // Derived in shader from UV
            v_phase: 0.0, // Derived in shader from UV
            lines: self.standard.visible_lines() as f32,
            field_rate: self.standard.field_rate(),
            subcarrier_phase: (self.subcarrier_phase % 1.0) as f32,
            _pad: [0.0; 7],
        }
    }

    pub fn field_count(&self) -> u64 {
        self.field_count
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn standard(&self) -> VideoStandard {
        self.standard
    }
}
