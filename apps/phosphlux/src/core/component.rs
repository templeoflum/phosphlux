//! Analog Component Models
//!
//! Authentic models of electronic components used in video synthesizers.
//! These capture the real-world behavior including nonlinearities,
//! frequency response, and temperature effects.
//!
//! Used to build device-level emulations that behave like actual hardware.

use std::f32::consts::PI;

/// Op-amp model with realistic characteristics
/// Based on common op-amps used in video synthesis: LM741, TL072, LM6172
#[derive(Clone, Debug)]
pub struct OpAmp {
    /// Open-loop gain (typically 100,000-200,000 for LM741)
    pub open_loop_gain: f32,
    /// Gain-bandwidth product in Hz (1 MHz for LM741, 100 MHz for LM6172)
    pub gbw: f32,
    /// Slew rate in V/µs (0.5 for LM741, 3000 for LM6172)
    pub slew_rate: f32,
    /// Input bias current in nA
    pub input_bias_na: f32,
    /// Input offset voltage in mV
    pub offset_mv: f32,
    /// Positive supply rail voltage
    pub v_supply_pos: f32,
    /// Negative supply rail voltage
    pub v_supply_neg: f32,
    /// Output saturation offset from rails
    pub saturation_offset: f32,
    /// Previous output (for slew rate limiting)
    pub prev_output: f32,
}

impl OpAmp {
    /// LM741 - Classic slow op-amp, used in early video synths
    /// The character of Paik/Abe and early Jones comes from this chip
    pub fn lm741() -> Self {
        Self {
            open_loop_gain: 200_000.0,
            gbw: 1_000_000.0,        // 1 MHz
            slew_rate: 0.5,          // Very slow - creates character
            input_bias_na: 80.0,
            offset_mv: 1.0,
            v_supply_pos: 15.0,
            v_supply_neg: -15.0,
            saturation_offset: 2.0,  // Can't get closer than 2V to rails
            prev_output: 0.0,
        }
    }

    /// TL072 - JFET input, better performance
    /// Common in later synths and Eurorack
    pub fn tl072() -> Self {
        Self {
            open_loop_gain: 200_000.0,
            gbw: 3_000_000.0,        // 3 MHz
            slew_rate: 13.0,
            input_bias_na: 0.065,    // JFET = very low bias
            offset_mv: 3.0,
            v_supply_pos: 15.0,
            v_supply_neg: -15.0,
            saturation_offset: 1.5,
            prev_output: 0.0,
        }
    }

    /// LM6172 - High-speed video op-amp
    /// Used in LZX Industries and modern video circuits
    pub fn lm6172() -> Self {
        Self {
            open_loop_gain: 10_000.0,  // Lower gain but much faster
            gbw: 100_000_000.0,        // 100 MHz!
            slew_rate: 3000.0,         // Extremely fast
            input_bias_na: 1500.0,     // Higher bias current
            offset_mv: 2.0,
            v_supply_pos: 15.0,
            v_supply_neg: -15.0,
            saturation_offset: 1.0,    // Can get closer to rails
            prev_output: 0.0,
        }
    }

    /// Calculate closed-loop gain at a given frequency
    pub fn gain_at_frequency(&self, closed_loop_gain: f32, freq_hz: f32) -> f32 {
        // First-order rolloff model
        let f_3db = self.gbw / closed_loop_gain;
        let ratio = freq_hz / f_3db;
        closed_loop_gain / (1.0 + ratio * ratio).sqrt()
    }

    /// Compute output with realistic behavior
    /// v_plus: non-inverting input
    /// v_minus: inverting input
    /// dt: time step in seconds
    pub fn compute(&mut self, v_plus: f32, v_minus: f32, dt: f32) -> f32 {
        // Add input offset
        let v_diff = (v_plus - v_minus) + (self.offset_mv / 1000.0);

        // Ideal output (open loop)
        let v_out_ideal = v_diff * self.open_loop_gain;

        // Clamp to supply rails (with saturation offset)
        let v_max = self.v_supply_pos - self.saturation_offset;
        let v_min = self.v_supply_neg + self.saturation_offset;
        let v_out_clamped = v_out_ideal.clamp(v_min, v_max);

        // Apply slew rate limiting
        let max_change = self.slew_rate * dt * 1_000_000.0; // Convert V/µs to V
        let delta = v_out_clamped - self.prev_output;
        let v_out_slewed = if delta.abs() > max_change {
            self.prev_output + delta.signum() * max_change
        } else {
            v_out_clamped
        };

        self.prev_output = v_out_slewed;
        v_out_slewed
    }

    /// Compute inverting amplifier configuration
    /// v_in: input voltage
    /// r_in: input resistor
    /// r_fb: feedback resistor
    pub fn inverting_amp(&mut self, v_in: f32, r_in: f32, r_fb: f32, dt: f32) -> f32 {
        // Virtual ground at inverting input
        let closed_loop_gain = -r_fb / r_in;
        let v_out_ideal = v_in * closed_loop_gain;

        // Use compute for realistic behavior
        // In inverting config, v_plus = 0 (ground), v_minus approaches virtual ground
        self.compute(0.0, -v_out_ideal / self.open_loop_gain, dt)
    }

    /// Reset state (for new patch)
    pub fn reset(&mut self) {
        self.prev_output = 0.0;
    }
}

/// Bipolar transistor model (NPN or PNP)
/// Used for discrete stages in early video synths
#[derive(Clone, Debug)]
pub struct BjTransistor {
    /// Current gain (hFE / beta)
    pub beta: f32,
    /// Base-emitter voltage drop
    pub vbe: f32,
    /// Early voltage (for output impedance)
    pub v_early: f32,
    /// Is PNP (inverted polarity)
    pub is_pnp: bool,
    /// Saturation voltage (Vce_sat)
    pub v_sat: f32,
}

impl BjTransistor {
    /// 2N3904 - Common NPN, used everywhere
    pub fn n2n3904() -> Self {
        Self {
            beta: 150.0,         // Typical, varies 100-300
            vbe: 0.65,
            v_early: 100.0,
            is_pnp: false,
            v_sat: 0.2,
        }
    }

    /// 2N3906 - Common PNP complement
    pub fn n2n3906() -> Self {
        Self {
            beta: 150.0,
            vbe: 0.65,
            v_early: 100.0,
            is_pnp: true,
            v_sat: 0.2,
        }
    }

    /// Calculate collector current in active region
    /// v_be: base-emitter voltage
    /// v_ce: collector-emitter voltage
    pub fn collector_current(&self, v_be: f32, v_ce: f32) -> f32 {
        if v_be < self.vbe * 0.9 {
            // Cutoff
            0.0
        } else if v_ce < self.v_sat {
            // Saturation
            let i_sat = (v_be - self.vbe).max(0.0) * 0.1; // Simplified
            i_sat * (v_ce / self.v_sat)
        } else {
            // Active region - exponential relationship
            let vt = 0.026; // Thermal voltage at 25°C
            let is = 1e-14; // Saturation current
            let ib = is * ((v_be / vt).exp() - 1.0);
            let ic = self.beta * ib;
            // Early effect
            ic * (1.0 + v_ce / self.v_early)
        }
    }
}

/// Diode model
#[derive(Clone, Debug)]
pub struct Diode {
    /// Forward voltage drop
    pub v_forward: f32,
    /// Saturation current
    pub i_sat: f32,
    /// Ideality factor (1.0-2.0)
    pub n: f32,
}

impl Diode {
    /// 1N4148 - Fast signal diode
    pub fn n1n4148() -> Self {
        Self {
            v_forward: 0.7,
            i_sat: 2.52e-9,
            n: 1.752,
        }
    }

    /// LED (red)
    pub fn led_red() -> Self {
        Self {
            v_forward: 1.8,
            i_sat: 1e-18,
            n: 1.5,
        }
    }

    /// Calculate forward current
    pub fn forward_current(&self, v_applied: f32) -> f32 {
        let vt = 0.026; // Thermal voltage
        if v_applied < 0.0 {
            -self.i_sat // Reverse bias
        } else {
            self.i_sat * ((v_applied / (self.n * vt)).exp() - 1.0)
        }
    }
}

/// RC time constant behavior
/// Essential for capacitor effects in feedback loops
#[derive(Clone, Debug)]
pub struct RcFilter {
    /// Resistance in ohms
    pub r: f32,
    /// Capacitance in farads
    pub c: f32,
    /// Current capacitor voltage (state)
    pub v_cap: f32,
}

impl RcFilter {
    pub fn new(r: f32, c: f32) -> Self {
        Self { r, c, v_cap: 0.0 }
    }

    /// Time constant in seconds
    pub fn tau(&self) -> f32 {
        self.r * self.c
    }

    /// Cutoff frequency in Hz
    pub fn f_cutoff(&self) -> f32 {
        1.0 / (2.0 * PI * self.tau())
    }

    /// Low-pass filter step
    /// Returns filtered output
    pub fn lowpass(&mut self, v_in: f32, dt: f32) -> f32 {
        let alpha = dt / (self.tau() + dt);
        self.v_cap = self.v_cap + alpha * (v_in - self.v_cap);
        self.v_cap
    }

    /// High-pass filter step
    /// Returns filtered output
    pub fn highpass(&mut self, v_in: f32, dt: f32) -> f32 {
        let v_out = v_in - self.v_cap;
        self.lowpass(v_in, dt);
        v_out
    }

    /// Reset state
    pub fn reset(&mut self) {
        self.v_cap = 0.0;
    }
}

/// Component tolerance for realistic variation
#[derive(Clone, Copy, Debug)]
pub struct Tolerance {
    /// Nominal value
    pub nominal: f32,
    /// Tolerance as fraction (0.05 = 5%)
    pub tolerance: f32,
    /// Temperature coefficient (ppm/°C)
    pub temp_coeff: f32,
}

impl Tolerance {
    /// Standard 5% resistor
    pub fn resistor_5pct(nominal: f32) -> Self {
        Self {
            nominal,
            tolerance: 0.05,
            temp_coeff: 100.0, // 100 ppm/°C typical
        }
    }

    /// Standard 10% capacitor
    pub fn capacitor_10pct(nominal: f32) -> Self {
        Self {
            nominal,
            tolerance: 0.10,
            temp_coeff: 200.0,
        }
    }

    /// Calculate actual value with random variation
    /// seed: 0-1 random value for this component instance
    pub fn actual_value(&self, seed: f32) -> f32 {
        let variation = (seed * 2.0 - 1.0) * self.tolerance;
        self.nominal * (1.0 + variation)
    }

    /// Calculate value at temperature
    /// temp_c: temperature in celsius
    /// ref_temp: reference temperature (usually 25°C)
    pub fn at_temperature(&self, temp_c: f32, ref_temp: f32) -> f32 {
        let delta_t = temp_c - ref_temp;
        let temp_factor = 1.0 + (self.temp_coeff * 1e-6 * delta_t);
        self.nominal * temp_factor
    }
}

/// Circuit bend modification types
#[derive(Clone, Debug)]
pub enum BendType {
    /// Component removed (open circuit)
    Open,
    /// Component shorted
    Short,
    /// Variable resistance (body contact, pot)
    Variable { min: f32, max: f32 },
    /// Wrong value substituted
    WrongValue(f32),
    /// Cross-connected to another point
    CrossConnect { target: String },
}

/// A bend point - a place where the circuit can be modified
#[derive(Clone, Debug)]
pub struct BendPoint {
    /// Human-readable name
    pub name: String,
    /// Component reference (R1, C3, U2-pin4, etc.)
    pub component_ref: String,
    /// What this bend does sonically/visually
    pub description: String,
    /// The type of bend
    pub bend_type: BendType,
    /// Is this bend currently active?
    pub active: bool,
    /// Current value (for Variable type)
    pub current_value: f32,
}

impl BendPoint {
    /// Create a removable component bend
    pub fn removable(name: &str, component: &str, desc: &str) -> Self {
        Self {
            name: name.to_string(),
            component_ref: component.to_string(),
            description: desc.to_string(),
            bend_type: BendType::Open,
            active: false,
            current_value: 0.0,
        }
    }

    /// Create a body contact bend point
    pub fn body_contact(name: &str, component: &str, desc: &str) -> Self {
        Self {
            name: name.to_string(),
            component_ref: component.to_string(),
            description: desc.to_string(),
            bend_type: BendType::Variable {
                min: 100_000.0,   // 100k ohm (wet fingers)
                max: 10_000_000.0, // 10M ohm (dry touch)
            },
            active: false,
            current_value: 1_000_000.0, // 1M default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opamp_saturation() {
        let mut op = OpAmp::lm741();
        // Large input should saturate
        let out = op.compute(10.0, 0.0, 0.001);
        assert!(out < op.v_supply_pos);
        assert!(out > op.v_supply_pos - op.saturation_offset - 0.1);
    }

    #[test]
    fn test_rc_lowpass() {
        let mut rc = RcFilter::new(1000.0, 0.000001); // 1k, 1µF = 160 Hz
        assert!((rc.f_cutoff() - 159.15).abs() < 1.0);

        // Step response
        rc.reset();
        for _ in 0..1000 {
            rc.lowpass(1.0, 0.0001);
        }
        // Should approach 1.0
        assert!(rc.v_cap > 0.99);
    }

    #[test]
    fn test_tolerance_variation() {
        let t = Tolerance::resistor_5pct(10000.0);

        // At seed 0.5 (center), should be nominal
        let v = t.actual_value(0.5);
        assert!((v - 10000.0).abs() < 1.0);

        // At extremes, should be ±5%
        let v_low = t.actual_value(0.0);
        let v_high = t.actual_value(1.0);
        assert!((v_low - 9500.0).abs() < 1.0);
        assert!((v_high - 10500.0).abs() < 1.0);
    }
}
