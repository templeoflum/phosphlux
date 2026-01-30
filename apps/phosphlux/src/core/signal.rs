//! Signal Types for Analog Circuit Emulation
//!
//! Unlike v1 which treated everything as 0-1 RGB values, this module
//! models signals with proper electrical characteristics: voltage ranges,
//! impedance, and signal type metadata.
//!
//! This enables authentic circuit behavior like loading effects,
//! impedance mismatches, and proper clipping at device-specific levels.

use std::ops::{Add, Mul, Sub};

/// Voltage range specification for a signal
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VoltageRange {
    /// Video signal: 0V to 1V (composite video standard)
    Video,
    /// Bipolar CV: -1V to +1V (modulation signals)
    BipolarCV,
    /// Unipolar CV: 0V to 1V (control signals)
    UnipolarCV,
    /// Audio/AC coupled: -1V to +1V, centered at 0
    AudioAC,
    /// TTL logic: 0V or 5V (sync signals)
    TTL,
    /// Custom range for specific circuits
    Custom { min: f32, max: f32 },
}

impl VoltageRange {
    /// Get the minimum voltage for this range
    pub fn min(&self) -> f32 {
        match self {
            VoltageRange::Video => 0.0,
            VoltageRange::BipolarCV => -1.0,
            VoltageRange::UnipolarCV => 0.0,
            VoltageRange::AudioAC => -1.0,
            VoltageRange::TTL => 0.0,
            VoltageRange::Custom { min, .. } => *min,
        }
    }

    /// Get the maximum voltage for this range
    pub fn max(&self) -> f32 {
        match self {
            VoltageRange::Video => 1.0,
            VoltageRange::BipolarCV => 1.0,
            VoltageRange::UnipolarCV => 1.0,
            VoltageRange::AudioAC => 1.0,
            VoltageRange::TTL => 5.0,
            VoltageRange::Custom { max, .. } => *max,
        }
    }

    /// Normalize a voltage to 0-1 range for this voltage range
    pub fn normalize(&self, voltage: f32) -> f32 {
        (voltage - self.min()) / (self.max() - self.min())
    }

    /// Convert from normalized 0-1 to actual voltage
    pub fn denormalize(&self, normalized: f32) -> f32 {
        normalized * (self.max() - self.min()) + self.min()
    }
}

/// Impedance characteristics of a signal source or load
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Impedance {
    /// Output impedance in ohms (for sources)
    pub output: f32,
    /// Input impedance in ohms (for loads)
    pub input: f32,
}

impl Impedance {
    /// Standard video impedance (75 ohm matched)
    pub const VIDEO: Self = Self { output: 75.0, input: 75.0 };

    /// High impedance input (typical op-amp input)
    pub const HIGH_Z: Self = Self { output: 100.0, input: 1_000_000.0 };

    /// Low impedance output (buffered)
    pub const LOW_Z_OUT: Self = Self { output: 50.0, input: 10_000.0 };

    /// Calculate voltage divider effect when connecting source to load
    /// Returns the attenuation factor (1.0 = no loss)
    pub fn loading_factor(source_z: f32, load_z: f32) -> f32 {
        load_z / (source_z + load_z)
    }
}

/// A single-channel analog signal with electrical characteristics
#[derive(Clone, Copy, Debug)]
pub struct AnalogSignal {
    /// The voltage value
    pub voltage: f32,
    /// The voltage range this signal operates in
    pub range: VoltageRange,
    /// Source impedance
    pub impedance: Impedance,
}

impl AnalogSignal {
    /// Create a new signal with standard video characteristics
    pub fn video(voltage: f32) -> Self {
        Self {
            voltage: voltage.clamp(0.0, 1.0),
            range: VoltageRange::Video,
            impedance: Impedance::VIDEO,
        }
    }

    /// Create a bipolar CV signal
    pub fn cv_bipolar(voltage: f32) -> Self {
        Self {
            voltage: voltage.clamp(-1.0, 1.0),
            range: VoltageRange::BipolarCV,
            impedance: Impedance::HIGH_Z,
        }
    }

    /// Create a unipolar CV signal
    pub fn cv_unipolar(voltage: f32) -> Self {
        Self {
            voltage: voltage.clamp(0.0, 1.0),
            range: VoltageRange::UnipolarCV,
            impedance: Impedance::HIGH_Z,
        }
    }

    /// Normalize to 0-1 range
    pub fn normalized(&self) -> f32 {
        self.range.normalize(self.voltage)
    }

    /// Apply loading effect from connecting to a load impedance
    pub fn loaded_by(&self, load_z: f32) -> Self {
        let factor = Impedance::loading_factor(self.impedance.output, load_z);
        Self {
            voltage: self.voltage * factor,
            range: self.range,
            impedance: self.impedance,
        }
    }

    /// Soft clip at the signal's natural range limits
    /// Uses tanh-style saturation for analog character
    pub fn soft_clip(&self) -> Self {
        let normalized = self.normalized();
        // Soft clip around 0.5 center
        let centered = normalized * 2.0 - 1.0;
        let clipped = centered.tanh();
        let denorm = (clipped + 1.0) * 0.5;
        Self {
            voltage: self.range.denormalize(denorm),
            range: self.range,
            impedance: self.impedance,
        }
    }
}

/// RGB signal with per-channel analog characteristics
#[derive(Clone, Copy, Debug)]
pub struct RgbSignal {
    pub r: AnalogSignal,
    pub g: AnalogSignal,
    pub b: AnalogSignal,
}

impl RgbSignal {
    /// Create from normalized 0-1 RGB values (video range)
    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Self {
            r: AnalogSignal::video(r),
            g: AnalogSignal::video(g),
            b: AnalogSignal::video(b),
        }
    }

    /// Create from a single luma value (grayscale)
    pub fn from_luma(luma: f32) -> Self {
        Self::from_rgb(luma, luma, luma)
    }

    /// Convert to luma using NTSC coefficients
    pub fn to_luma(&self) -> f32 {
        0.299 * self.r.voltage + 0.587 * self.g.voltage + 0.114 * self.b.voltage
    }

    /// Get as normalized [0,1] tuple for shader upload
    pub fn as_tuple(&self) -> (f32, f32, f32) {
        (
            self.r.normalized(),
            self.g.normalized(),
            self.b.normalized(),
        )
    }

    /// Soft clip all channels
    pub fn soft_clip(&self) -> Self {
        Self {
            r: self.r.soft_clip(),
            g: self.g.soft_clip(),
            b: self.b.soft_clip(),
        }
    }
}

/// YIQ signal for NTSC color encoding
/// The IQ plane enables authentic NTSC artifact emulation
#[derive(Clone, Copy, Debug)]
pub struct YiqSignal {
    pub y: f32,  // Luminance: 0 to 1
    pub i: f32,  // In-phase (orange-cyan axis): -0.5957 to 0.5957
    pub q: f32,  // Quadrature (green-magenta axis): -0.5226 to 0.5226
}

impl YiqSignal {
    /// Convert from RGB
    pub fn from_rgb(rgb: &RgbSignal) -> Self {
        let r = rgb.r.voltage;
        let g = rgb.g.voltage;
        let b = rgb.b.voltage;

        Self {
            y: 0.299 * r + 0.587 * g + 0.114 * b,
            i: 0.5959 * r - 0.2746 * g - 0.3213 * b,
            q: 0.2115 * r - 0.5227 * g + 0.3112 * b,
        }
    }

    /// Convert to RGB
    pub fn to_rgb(&self) -> RgbSignal {
        let r = self.y + 0.956 * self.i + 0.619 * self.q;
        let g = self.y - 0.272 * self.i - 0.647 * self.q;
        let b = self.y - 1.106 * self.i + 1.703 * self.q;

        RgbSignal::from_rgb(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
    }

    /// Rotate hue by modifying the IQ plane
    pub fn rotate_hue(&self, angle_radians: f32) -> Self {
        let cos_a = angle_radians.cos();
        let sin_a = angle_radians.sin();

        Self {
            y: self.y,
            i: self.i * cos_a - self.q * sin_a,
            q: self.i * sin_a + self.q * cos_a,
        }
    }

    /// Get chroma magnitude (saturation)
    pub fn chroma(&self) -> f32 {
        (self.i * self.i + self.q * self.q).sqrt()
    }

    /// Get hue angle
    pub fn hue(&self) -> f32 {
        self.q.atan2(self.i)
    }
}

/// Processing rate for different circuit behaviors
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessingRate {
    /// Per-frame processing (60 Hz) - feedback loops, slow modulation
    PerFrame,
    /// Per-field processing (120 Hz) - interlace-aware effects
    PerField,
    /// Per-scanline processing (15.734 kHz NTSC) - horizontal effects
    PerScanline,
    /// Per-sample processing - high-rate simulation for critical components
    PerSample,
}

impl ProcessingRate {
    /// Get the rate in Hz for NTSC
    pub fn hz_ntsc(&self) -> f32 {
        match self {
            ProcessingRate::PerFrame => 29.97,
            ProcessingRate::PerField => 59.94,
            ProcessingRate::PerScanline => 15_734.0,
            ProcessingRate::PerSample => 13_500_000.0, // 13.5 MHz sample rate
        }
    }

    /// Get time step in seconds
    pub fn dt(&self) -> f32 {
        1.0 / self.hz_ntsc()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voltage_range_normalize() {
        let bipolar = VoltageRange::BipolarCV;
        assert!((bipolar.normalize(-1.0) - 0.0).abs() < 0.001);
        assert!((bipolar.normalize(0.0) - 0.5).abs() < 0.001);
        assert!((bipolar.normalize(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_loading_effect() {
        // High Z load should have minimal loading
        let factor = Impedance::loading_factor(100.0, 1_000_000.0);
        assert!(factor > 0.999);

        // Matched load should lose half
        let factor = Impedance::loading_factor(75.0, 75.0);
        assert!((factor - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_yiq_roundtrip() {
        let rgb = RgbSignal::from_rgb(0.8, 0.4, 0.2);
        let yiq = YiqSignal::from_rgb(&rgb);
        let rgb2 = yiq.to_rgb();

        assert!((rgb.r.voltage - rgb2.r.voltage).abs() < 0.01);
        assert!((rgb.g.voltage - rgb2.g.voltage).abs() < 0.01);
        assert!((rgb.b.voltage - rgb2.b.voltage).abs() < 0.01);
    }
}
