//! LZX Cadet IV Dual Ramp Generator (T-0374, first pass)
//!
//! Built from `research/devices/lzx_cadet_iv/CIRCUIT.md` (cited below as "stage N"), judged by
//! `research/devices/lzx_cadet_iv/FAITHFUL.md`. Six jacks and the two trimmers, nothing else: the
//! module has no front-panel controls and no signal inputs (CIRCUIT.md "The module as a whole").
//!
//! Every output is a function of time since the sync edge. A pixel's time comes from the picture
//! window FAITHFUL.md §2 assumes (LZX states neither the sync widths nor the window, stage 2a).
//! The same formulas run on the CPU (`CadetIvModel`) for tests and in WGSL for the app; the shader
//! text is generated from the constants here so the two cannot drift apart.

use crate::gpu::{DeviceParameter, Port, ShaderDevice, SignalType};

// ─── Circuit constants (CIRCUIT.md) ──────────────────────────

/// TL431 reference, typical (stage 3, datasheet TI p14).
pub const VREF: f64 = 2.495;
/// Charging current: (2·Vref)/R44 with R39 = R42 = 10K, R44 = 4.99K, Iref ignored (stage 4).
pub const I_CHARGE: f64 = 2.0 * VREF / 4.99e3;
/// Timing capacitors C29 (H) and C18 (V), nominal (stage 5).
pub const C_H: f64 = 56e-9;
pub const C_V: f64 = 15e-6;
/// −2 V reference magnitude: Vref·R41/R40 = Vref·10/12.4 (stage 3).
pub const V_NEG: f64 = VREF * 10.0 / 12.4;
/// Inverted ramp ceiling: (R46/R48)·|V(−2V)| = Vref·10/24.8 ≈ 1.006 V (stage 8).
pub const INV_CEILING: f64 = V_NEG * 10.0 / 20.0;
/// Triangle: out = (R16/R13)·|ramp − zero|, zero = |V(−2V)|·R13/R11 (stage 9).
pub const TRI_GAIN: f64 = 1000.0 / 499.0;
pub const TRI_ZERO: f64 = V_NEG * 499.0 / 2000.0;
/// Reset switch on-resistance. NOT from LZX: CIRCUIT.md Q11 says the residual after reset is not
/// stated. 2.33 Ω is fitted to the T-0278 simulation's V-ramp discharge (results/frame_*.csv,
/// τ ≈ 35 µs with C18). It leaves ~5 mV on C18 after V sync and nothing measurable on C29.
pub const R_RESET_ON: f64 = 2.33;

/// Trimmer wiper position p (0..1) to the fraction of the capacitor voltage: (4.99 + 10p)/14.99
/// (stage 6, R1/R3 10K over R2/R4 4.99K).
#[cfg_attr(not(test), allow(dead_code))] // CPU model: tests now, the GPU comparison next
pub fn trim_fraction(p: f64) -> f64 {
    (4.99 + 10.0 * p.clamp(0.0, 1.0)) / 14.99
}

// ─── Timing (assumed; FAITHFUL.md §2, sim/run_sims.py) ───────

/// Sync and picture window for one video standard. Rates are from the Cadet I (CIRCUIT.md 2a,
/// C1 p6); sync widths and the window are ASSUMED, as in the T-0278 simulation and FAITHFUL.md §2.
#[derive(Clone, Copy, Debug)]
pub struct Standard {
    pub line: f64,
    pub field: f64,
    pub h_sync: f64,
    pub v_sync: f64,
    /// Picture columns start this long after the H sync leading edge, and last `h_len`.
    pub h_start: f64,
    pub h_len: f64,
    /// Picture rows start this many lines after the V sync leading edge; `rows` rows.
    pub v_start_lines: f64,
    pub rows: f64,
}

pub const NTSC: Standard = Standard {
    line: 63.556e-6,
    field: 16.6833e-3,
    h_sync: 4.7e-6,
    v_sync: 3.0 * 63.556e-6,
    h_start: 9.4e-6,
    h_len: 52.66e-6,
    v_start_lines: 18.0,
    rows: 240.0,
};

#[cfg_attr(not(test), allow(dead_code))] // CPU model: tests now, the GPU comparison next
pub const PAL: Standard = Standard {
    line: 64.0e-6,
    field: 20.0e-3,
    h_sync: 4.7e-6,
    v_sync: 2.5 * 64.0e-6,
    h_start: 10.5e-6,
    h_len: 52.0e-6,
    v_start_lines: 23.0,
    rows: 288.0,
};

// ─── CPU model ───────────────────────────────────────────────

/// Capacitor voltage at `t` seconds after the sync leading edge (stages 4 and 5).
/// Sync holds the BS170 on: the capacitor discharges from last period's peak through R_RESET_ON.
/// After sync it charges at I/C from what is left.
#[cfg_attr(not(test), allow(dead_code))] // CPU model: tests now, the GPU comparison next
pub fn cap_voltage(t: f64, period: f64, sync: f64, c: f64) -> f64 {
    let slope = I_CHARGE / c;
    let tau = R_RESET_ON * c;
    let peak = slope * (period - sync);
    if t < sync {
        peak * (-t / tau).exp()
    } else {
        peak * (-sync / tau).exp() + slope * (t - sync)
    }
}

/// The three jacks of one channel from its scaled ramp: [ramp, inverted, triangle] (stages 7–9).
#[cfg_attr(not(test), allow(dead_code))] // CPU model: tests now, the GPU comparison next
pub fn channel_jacks(ramp: f64) -> [f64; 3] {
    [ramp, INV_CEILING - ramp, TRI_GAIN * (ramp - TRI_ZERO).abs()]
}

/// CPU model of the six jacks at a pixel, in panel order:
/// J8 H /|, J6 H |\, J10 H V, J3 V /|, J2 V |\, J4 V V (stage 12). Volts, unclamped.
#[cfg_attr(not(test), allow(dead_code))] // CPU model: tests now, the GPU comparison next
pub struct CadetIvModel {
    pub std: Standard,
    pub h_adj: f64,
    pub v_adj: f64,
}

#[cfg_attr(not(test), allow(dead_code))] // CPU model: tests now, the GPU comparison next
impl CadetIvModel {
    pub fn h_time(&self, u: f64) -> f64 {
        self.std.h_start + u * self.std.h_len
    }

    pub fn v_time(&self, v: f64) -> f64 {
        (self.std.v_start_lines + v * self.std.rows) * self.std.line
    }

    pub fn h_ramp(&self, u: f64) -> f64 {
        let s = &self.std;
        trim_fraction(self.h_adj) * cap_voltage(self.h_time(u), s.line, s.h_sync, C_H)
    }

    pub fn v_ramp(&self, v: f64) -> f64 {
        let s = &self.std;
        trim_fraction(self.v_adj) * cap_voltage(self.v_time(v), s.field, s.v_sync, C_V)
    }

    pub fn jacks(&self, u: f64, v: f64) -> [f64; 6] {
        let h = channel_jacks(self.h_ramp(u));
        let w = channel_jacks(self.v_ramp(v));
        [h[0], h[1], h[2], w[0], w[1], w[2]]
    }
}

// ─── Device ──────────────────────────────────────────────────

/// Jack names in panel order (stage 12). Port 0 is what the device shows when it is the output.
pub const JACKS: [&str; 6] = ["h_ramp", "h_inv", "h_tri", "v_ramp", "v_inv", "v_tri"];

pub struct LzxCadetIv {
    name: String,
    shader: String,
}

impl LzxCadetIv {
    pub fn new() -> Self {
        Self {
            name: "Cadet IV Ramps".to_string(),
            shader: shader_source(&NTSC),
        }
    }
}

impl Default for LzxCadetIv {
    fn default() -> Self {
        Self::new()
    }
}

fn shader_source(s: &Standard) -> String {
    let f = |x: f64| format!("{:.9e}", x);
    format!(
        r#"
// LZX Cadet IV Dual Ramp Generator (T-0374). Formulas and constants: devices/lzx/cadet_iv.rs,
// from research/devices/lzx_cadet_iv/CIRCUIT.md. NTSC timing as FAITHFUL.md section 2 assumes.

// Stages 4-5: capacitor charged at I/C after sync, discharged through the BS170 during sync.
fn lzx_cadet_iv_cap(t: f32, period: f32, sync: f32, slope: f32, tau: f32) -> f32 {{
    let peak = slope * (period - sync);
    if t < sync {{
        return peak * exp(-t / tau);
    }}
    return peak * exp(-sync / tau) + slope * (t - sync);
}}

// Stage 6: trimmer wiper p -> (4.99 + 10p) / 14.99 of the capacitor voltage.
fn lzx_cadet_iv_trim(p: f32) -> f32 {{
    return (4.99 + 10.0 * clamp(p, 0.0, 1.0)) / 14.99;
}}

// Six jacks in panel order (stage 12): 0 J8 H ramp, 1 J6 H inverted, 2 J10 H triangle,
// 3 J3 V ramp, 4 J2 V inverted, 5 J4 V triangle. Volts, unclamped; 1 V = 1.0.
fn device_lzx_cadet_iv_output(uv: vec2<f32>, time: f32, port: u32, h_adj: f32, v_adj: f32) -> vec4<f32> {{
    var ramp: f32;
    if port < 3u {{
        // Stage 7 (J8): H ramp at this column's time after H sync.
        let t = {h_start} + uv.x * {h_len};
        ramp = lzx_cadet_iv_trim(h_adj) * lzx_cadet_iv_cap(t, {line}, {h_sync}, {slope_h}, {tau_h});
    }} else {{
        // Stage 7 (J3): V ramp at this row's time after V sync.
        let t = ({v_start} + uv.y * {rows}) * {line};
        ramp = lzx_cadet_iv_trim(v_adj) * lzx_cadet_iv_cap(t, {field}, {v_sync}, {slope_v}, {tau_v});
    }}
    let jack = port % 3u;
    var v = ramp;
    if jack == 1u {{
        // Stage 8: inverted ramp, Vref*10/24.8 - ramp.
        v = {inv} - ramp;
    }} else if jack == 2u {{
        // Stage 9: triangle, (1K/499R) * |ramp - |V(-2V)|*499/2K|.
        v = {tri_gain} * abs(ramp - {tri_zero});
    }}
    return vec4<f32>(v, v, v, 1.0);
}}

fn device_lzx_cadet_iv_process(uv: vec2<f32>, time: f32, h_adj: f32, v_adj: f32) -> vec4<f32> {{
    return device_lzx_cadet_iv_output(uv, time, 0u, h_adj, v_adj);
}}
"#,
        h_start = f(s.h_start),
        h_len = f(s.h_len),
        line = f(s.line),
        h_sync = f(s.h_sync),
        slope_h = f(I_CHARGE / C_H),
        tau_h = f(R_RESET_ON * C_H),
        v_start = f(s.v_start_lines),
        rows = f(s.rows),
        field = f(s.field),
        v_sync = f(s.v_sync),
        slope_v = f(I_CHARGE / C_V),
        tau_v = f(R_RESET_ON * C_V),
        inv = f(INV_CEILING),
        tri_gain = f(TRI_GAIN),
        tri_zero = f(TRI_ZERO),
    )
}

impl ShaderDevice for LzxCadetIv {
    fn device_id(&self) -> &'static str {
        "lzx_cadet_iv"
    }

    fn device_name(&self) -> &str {
        &self.name
    }

    fn shader_code(&self) -> &str {
        &self.shader
    }

    fn parameters(&self) -> Vec<DeviceParameter> {
        // R1 "H ADJ" and R3 "V ADJ" (stage 6, PCB silkscreen). The calibrated setting is not
        // stated (stage 10); 1.0 is the reference's top setting.
        vec![
            DeviceParameter::new("h_adj", "H ADJ (R1)", 0.0, 1.0, 1.0),
            DeviceParameter::new("v_adj", "V ADJ (R3)", 0.0, 1.0, 1.0),
        ]
    }

    fn inputs(&self) -> Vec<Port> {
        // Sync comes from the J1 header, not the panel (stage 2); the app's timing stands in.
        vec![]
    }

    fn outputs(&self) -> Vec<Port> {
        JACKS
            .iter()
            .map(|name| Port {
                name: name.to_string(),
                signal_type: SignalType::Mono,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One line of faithful/reference_check.txt: the simulated circuit, sampled at pixel centres.
    struct Ref {
        std: &'static str,
        trim: f64,
        ch: char,
        first: f64,
        last: f64,
        slope: f64,
        tri_min: f64,
        tri_min_ramp: f64,
        sum_lo: f64,
        sum_hi: f64,
        crosses_zero: bool,
    }

    const REFERENCE: &str =
        include_str!("../../../../../research/devices/lzx_cadet_iv/faithful/reference_check.txt");

    fn after<'a>(line: &'a str, key: &str) -> &'a str {
        let i = line.find(key).unwrap_or_else(|| panic!("{key:?} missing in {line:?}"));
        &line[i + key.len()..]
    }

    fn num(s: &str) -> f64 {
        let end = s
            .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+'))
            .unwrap_or(s.len());
        s[..end].parse().unwrap_or_else(|_| panic!("not a number: {s:?}"))
    }

    fn references() -> Vec<Ref> {
        let lines: Vec<&str> = REFERENCE.lines().collect();
        let mut out = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            if !(line.starts_with("ntsc ") || line.starts_with("pal ")) {
                continue;
            }
            let std = if line.starts_with("ntsc") { "ntsc" } else { "pal" };
            let ch = after(line, ") ").chars().next().unwrap();
            let sum = after(line, "ramp+inv ");
            let (lo, hi) = sum.split_once("..").unwrap();
            out.push(Ref {
                std,
                trim: num(after(line, "trim ")),
                ch,
                first: num(after(line, "ramp first ")),
                last: num(after(line, " last ")),
                slope: num(after(line, "slope ")),
                tri_min: num(after(line, "tri min ")),
                tri_min_ramp: num(after(after(line, "tri min "), "(ramp ")),
                sum_lo: num(lo),
                sum_hi: num(hi),
                crosses_zero: !lines[i + 1].contains("tri zero n/a"),
            });
        }
        assert_eq!(out.len(), 8, "reference_check.txt should hold 4 runs x 2 channels");
        out
    }

    /// The model's jack values across the picture for one channel, at pixel centres.
    fn sweep(r: &Ref) -> (Vec<[f64; 3]>, f64) {
        let std = if r.std == "ntsc" { NTSC } else { PAL };
        let m = CadetIvModel { std, h_adj: r.trim, v_adj: r.trim };
        let (n, span) = if r.ch == 'H' { (720, std.h_len) } else { (std.rows as usize, std.rows * std.line) };
        let vals = (0..n)
            .map(|i| {
                let x = (i as f64 + 0.5) / n as f64;
                let j = m.jacks(x, x);
                if r.ch == 'H' { [j[0], j[1], j[2]] } else { [j[3], j[4], j[5]] }
            })
            .collect();
        (vals, span)
    }

    fn fit_slope(y: &[f64], span: f64) -> f64 {
        let n = y.len() as f64;
        let xs: Vec<f64> = (0..y.len()).map(|i| (i as f64 + 0.5) / n).collect();
        let mx = xs.iter().sum::<f64>() / n;
        let my = y.iter().sum::<f64>() / n;
        let num: f64 = xs.iter().zip(y).map(|(x, v)| (x - mx) * (v - my)).sum();
        let den: f64 = xs.iter().map(|x| (x - mx).powi(2)).sum();
        num / den / span
    }

    // Margins: FAITHFUL.md §2 (rows 2, 3, 5, 6). Not changed here.
    const LEVEL: f64 = 0.010;
    const SLOPE: f64 = 0.01;

    #[test]
    fn start_level_is_grey_like_the_circuit() {
        for r in references() {
            let (v, _) = sweep(&r);
            let d = v[0][0] - r.first;
            assert!(d.abs() <= LEVEL, "{} {} {}: start {:.4} vs {:.4}", r.std, r.trim, r.ch, v[0][0], r.first);
            assert!(v[0][0] > 0.02, "start must be grey, not black");
        }
    }

    #[test]
    fn slope_matches_reference() {
        for r in references() {
            let (v, span) = sweep(&r);
            let ramp: Vec<f64> = v.iter().map(|j| j[0]).collect();
            let per_unit = if r.ch == 'H' { 1e-6 } else { 1e-3 };
            let model = fit_slope(&ramp, span) * per_unit * 1e3; // mV per µs (H) or ms (V)
            let err = model / r.slope - 1.0;
            assert!(err.abs() <= SLOPE, "{} {} {}: slope {:.3} vs {:.3}", r.std, r.trim, r.ch, model, r.slope);
        }
    }

    #[test]
    fn peak_matches_reference() {
        for r in references() {
            let (v, _) = sweep(&r);
            let d = v[v.len() - 1][0] - r.last;
            assert!(d.abs() <= LEVEL, "{} {} {}: peak {:.4} vs {:.4}", r.std, r.trim, r.ch, r.last + d, r.last);
        }
    }

    #[test]
    fn ramp_plus_inverted_matches_reference() {
        for r in references() {
            let (v, _) = sweep(&r);
            for j in &v {
                let s = j[0] + j[1];
                assert!(s >= r.sum_lo - LEVEL && s <= r.sum_hi + LEVEL, "{} {} {}: sum {:.4}", r.std, r.trim, r.ch, s);
            }
        }
    }

    #[test]
    fn triangle_zero_matches_reference() {
        let mut checked = 0;
        for r in references() {
            let (v, _) = sweep(&r);
            let m = (0..v.len()).min_by(|&a, &b| v[a][2].total_cmp(&v[b][2])).unwrap();
            assert!((v[m][2] - r.tri_min).abs() <= LEVEL, "{} {} {}: tri min {:.4} vs {:.4}", r.std, r.trim, r.ch, v[m][2], r.tri_min);
            if r.crosses_zero {
                assert!((v[m][0] - r.tri_min_ramp).abs() <= LEVEL, "{} {} {}: ramp at tri min {:.4} vs {:.4}", r.std, r.trim, r.ch, v[m][0], r.tri_min_ramp);
                checked += 1;
            }
        }
        assert_eq!(checked, 6, "six runs reach the triangle's zero (reference_check.txt)");
    }

    #[test]
    fn document_values() {
        // The hand-worked numbers in CIRCUIT.md: 1.00 mA, 17.86 mV/µs, 66.7 mV/ms, 1.006 V, 0.502 V, 2.004.
        assert!((I_CHARGE - 1.0e-3).abs() < 1e-6);
        assert!((I_CHARGE / C_H * 1e-6 - 17.86e-3).abs() < 1e-5);
        assert!((I_CHARGE / C_V * 1e-3 - 66.7e-3).abs() < 1e-4);
        assert!((INV_CEILING - 1.006).abs() < 5e-4);
        assert!((TRI_ZERO - 0.502).abs() < 5e-4);
        assert!((TRI_GAIN - 2.004).abs() < 5e-4);
        assert!((trim_fraction(0.0) - 0.333).abs() < 1e-3 && trim_fraction(1.0) == 1.0);
    }

    /// Test-only pass-through, so jacks other than port 0 reach the output through the composer.
    struct Thru;

    impl ShaderDevice for Thru {
        fn device_id(&self) -> &'static str {
            "test_thru"
        }
        fn device_name(&self) -> &str {
            "Thru"
        }
        fn shader_code(&self) -> &str {
            "fn device_test_thru_process(uv: vec2<f32>, time: f32, video_in: vec4<f32>) -> vec4<f32> { return video_in; }"
        }
        fn parameters(&self) -> Vec<DeviceParameter> {
            vec![]
        }
        fn inputs(&self) -> Vec<Port> {
            vec![Port { name: "video_in".to_string(), signal_type: SignalType::Rgb }]
        }
        fn outputs(&self) -> Vec<Port> {
            vec![Port { name: "video_out".to_string(), signal_type: SignalType::Rgb }]
        }
    }

    /// Composes the patch as the app does and renders it on the GPU off-screen, 720×240 (FAITHFUL.md
    /// NTSC window), for all six jacks at the default trimmers. Checks the GPU against the CPU model
    /// and writes a 3×2 contact sheet (panel order, sRGB-encoded as the app's sRGB surface shows it).
    /// Needs a GPU, so it is ignored by default:
    ///   CADET_IV_FRAME=<path.png> cargo test cadet_iv_gpu_frame -- --ignored
    #[test]
    #[ignore]
    fn cadet_iv_gpu_frame() {
        use crate::gpu::offscreen::{grey_png, render_patch_f32};
        use crate::gpu::DevicePatch;
        let (w, h) = (720u32, 240u32);
        let model = CadetIvModel { std: NTSC, h_adj: 1.0, v_adj: 1.0 };
        let mut sheet = vec![0u8; (w * 3 * h * 2) as usize];
        for (port, jack) in JACKS.iter().enumerate() {
            let mut patch = DevicePatch::new();
            let cadet = patch.add_device(Box::new(LzxCadetIv::new()), 0.0, 0.0);
            patch.output_device = cadet;
            if port > 0 {
                let thru = patch.add_device(Box::new(Thru), 0.0, 100.0);
                patch.connect(cadet, jack, thru, "video_in");
                patch.output_device = thru;
            }
            let px = render_patch_f32(&patch, w, h).expect("off-screen render");
            let mut worst = 0.0f64;
            for (i, p) in px.iter().enumerate() {
                let (x, y) = ((i as u32 % w) as f64, (i as u32 / w) as f64);
                let want = model.jacks((x + 0.5) / w as f64, (y + 0.5) / h as f64)[port];
                worst = worst.max((p[0] as f64 - want).abs());
                let srgb = |v: f64| {
                    let v = v.clamp(0.0, 1.0);
                    if v <= 0.0031308 { 12.92 * v } else { 1.055 * v.powf(1.0 / 2.4) - 0.055 }
                };
                let (cx, cy) = (port as u32 % 3, port as u32 / 3);
                let at = ((cy * h + y as u32) * w * 3 + cx * w + x as u32) as usize;
                sheet[at] = (srgb(p[0] as f64) * 255.0).round() as u8;
            }
            println!(
                "{jack}: first {:.4} last {:.4} (row 0) / first {:.4} last {:.4} (col 0); GPU vs CPU model max diff {:.6}",
                px[0][0], px[w as usize - 1][0], px[0][0], px[((h - 1) * w) as usize][0], worst
            );
            assert!(worst < 1e-4, "{jack}: GPU differs from CPU model by {worst}");
        }
        if let Ok(path) = std::env::var("CADET_IV_FRAME") {
            std::fs::write(&path, grey_png(w * 3, h * 2, &sheet)).expect("write frame");
            println!("wrote {path}");
        }
    }

    /// Cadet IV jack `jack` patched straight into the Wobulator (a scan modulator, so a second pass).
    fn into_wobulator(jack: &str) -> crate::gpu::DevicePatch {
        let mut patch = crate::gpu::DevicePatch::new();
        let cadet = patch.add_device(Box::new(LzxCadetIv::new()), 0.0, 0.0);
        let wob = patch.add_device(Box::new(crate::devices::PaikAbeWobulator::new()), 0.0, 100.0);
        patch.connect(cadet, jack, wob, "video_in");
        patch.output_device = wob;
        patch
    }

    /// Verifier finding on T-0374: across a pass boundary every jack arrived as the H ramp.
    /// The pass feeding the Wobulator must now hand on the jack that was patched.
    #[test]
    fn each_jack_reaches_a_scan_modulator() {
        let composer = crate::gpu::ShaderComposer::new();
        let mut first_passes = Vec::new();
        for (port, jack) in JACKS.iter().enumerate() {
            let plan = composer.compose_plan(&into_wobulator(jack)).expect("compose");
            assert_eq!(plan.segments.len(), 2, "{jack}: Cadet pass, then Wobulator pass");
            let wgsl = &plan.segments[0].wgsl_source;
            let returns = if port == 0 {
                "return signal_0;".to_string()
            } else {
                format!("return device_lzx_cadet_iv_output(uv, time, {port}u, ")
            };
            assert!(wgsl.contains(&returns), "{jack}: first pass should end with {returns:?}");
            first_passes.push(wgsl.clone());
        }
        for i in 0..6 {
            for j in i + 1..6 {
                assert_ne!(first_passes[i], first_passes[j], "{} and {} route the same", JACKS[i], JACKS[j]);
            }
        }
    }

    /// GPU readback of the pass the Wobulator samples, for each jack, against the CPU model.
    ///   cargo test cadet_iv_gpu_into_wobulator -- --ignored --nocapture
    #[test]
    #[ignore]
    fn cadet_iv_gpu_into_wobulator() {
        use crate::gpu::offscreen::render_segment_f32;
        let (w, h) = (720u32, 240u32);
        let model = CadetIvModel { std: NTSC, h_adj: 1.0, v_adj: 1.0 };
        for (port, jack) in JACKS.iter().enumerate() {
            let patch = into_wobulator(jack);
            let plan = crate::gpu::ShaderComposer::new().compose_plan(&patch).expect("compose");
            let px = render_segment_f32(&patch, &plan.segments[0], w, h).expect("render");
            let mut worst = 0.0f64;
            for (i, p) in px.iter().enumerate() {
                let (x, y) = ((i as u32 % w) as f64, (i as u32 / w) as f64);
                let want = model.jacks((x + 0.5) / w as f64, (y + 0.5) / h as f64)[port];
                worst = worst.max((p[0] as f64 - want).abs());
            }
            println!("{jack} -> Wobulator input texture: max diff from CPU model {worst:.6}");
            assert!(worst < 1e-4, "{jack}: texture into the Wobulator is not this jack ({worst})");
        }
    }

    #[test]
    fn device_has_six_jacks_and_two_trimmers() {
        let d = LzxCadetIv::new();
        assert_eq!(d.outputs().len(), 6);
        assert_eq!(d.parameters().len(), 2);
        assert!(d.inputs().is_empty());
        assert!(d.shader_code().contains("fn device_lzx_cadet_iv_output("));
    }
}
