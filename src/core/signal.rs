//! Signal Types and Voltage Standards
//!
//! Following LZX Industries conventions:
//! - Video signals: 0-1V range, DC coupled
//! - Control voltages: 0-1V (unipolar) or -1V to +1V (bipolar)
//!
//! In our GPU implementation:
//! - Textures store signals as normalized floats [0, 1] or [-1, 1]
//! - RGB textures = 3 independent signal channels
//! - RGBA = 3 signals + 1 key/alpha channel

use bytemuck::{Pod, Zeroable};

/// Signal voltage range convention
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignalRange {
    /// 0V to 1V - video signals, unipolar CV
    Unipolar,
    /// -1V to +1V - bipolar CV, audio
    Bipolar,
}

/// Color space for signal interpretation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorSpace {
    /// Red, Green, Blue (LZX standard)
    RGB,
    /// Luminance, In-phase, Quadrature (NTSC native)
    YIQ,
    /// Luminance, Chroma-blue, Chroma-red (PAL/digital)
    YCbCr,
    /// Single channel grayscale
    Luma,
}

/// RGB color value (0-1 range per channel)
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
pub struct RgbSignal {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl RgbSignal {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }

    /// Convert to luminance using NTSC coefficients
    pub fn to_luma(&self) -> f32 {
        0.299 * self.r + 0.587 * self.g + 0.114 * self.b
    }

    /// Convert to YIQ color space (NTSC native)
    pub fn to_yiq(&self) -> YiqSignal {
        YiqSignal {
            y: 0.299 * self.r + 0.587 * self.g + 0.114 * self.b,
            i: 0.596 * self.r - 0.274 * self.g - 0.322 * self.b,
            q: 0.211 * self.r - 0.523 * self.g + 0.312 * self.b,
        }
    }

    /// Soft clip to valid range (tanh-style saturation)
    pub fn soft_clip(&self) -> Self {
        Self {
            r: soft_clip(self.r),
            g: soft_clip(self.g),
            b: soft_clip(self.b),
        }
    }

    /// Hard clip to valid range
    pub fn clamp(&self) -> Self {
        Self {
            r: self.r.clamp(0.0, 1.0),
            g: self.g.clamp(0.0, 1.0),
            b: self.b.clamp(0.0, 1.0),
        }
    }
}

/// YIQ color value (NTSC native color space)
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
pub struct YiqSignal {
    pub y: f32, // Luminance [0, 1]
    pub i: f32, // In-phase [-0.5957, 0.5957]
    pub q: f32, // Quadrature [-0.5226, 0.5226]
}

impl YiqSignal {
    pub fn new(y: f32, i: f32, q: f32) -> Self {
        Self { y, i, q }
    }

    /// Convert back to RGB
    pub fn to_rgb(&self) -> RgbSignal {
        RgbSignal {
            r: self.y + 0.956 * self.i + 0.621 * self.q,
            g: self.y - 0.272 * self.i - 0.647 * self.q,
            b: self.y - 1.106 * self.i + 1.703 * self.q,
        }
    }

    /// Rotate hue by phase angle (radians)
    /// This is how analog colorizers shift hue
    pub fn rotate_hue(&self, angle: f32) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            y: self.y,
            i: self.i * cos_a - self.q * sin_a,
            q: self.i * sin_a + self.q * cos_a,
        }
    }
}

/// Soft clipping function (tube-style saturation)
/// Maps any value smoothly to [-1, 1] or [0, 1] range
#[inline]
pub fn soft_clip(x: f32) -> f32 {
    // Fast approximation: x / (1 + |x|)
    // For [0,1] output, shift and scale
    let centered = x * 2.0 - 1.0;
    let clipped = centered / (1.0 + centered.abs());
    (clipped + 1.0) * 0.5
}

/// Hard clipping function
#[inline]
pub fn hard_clip(x: f32, min: f32, max: f32) -> f32 {
    x.clamp(min, max)
}

/// Wave folding (Paik/Abe style)
/// When signal exceeds threshold, it folds back
#[inline]
pub fn fold(x: f32, threshold: f32) -> f32 {
    if threshold <= 0.0 {
        return x.clamp(0.0, 1.0);
    }

    let mut v = x;
    let range = threshold * 2.0;

    // Fold the value back within range
    while v > threshold || v < -threshold {
        if v > threshold {
            v = threshold - (v - threshold);
        }
        if v < -threshold {
            v = -threshold - (v + threshold);
        }
    }

    // Normalize to [0, 1]
    (v / threshold + 1.0) * 0.5
}
