/**
 * Waveshaper Modules
 *
 * Nonlinear processing modules that create the distinctive character
 * of analog video synthesis. Includes:
 * - Paik/Abe style folding amplifier (solarization)
 * - Soft clipping / saturation
 * - Hard clipping
 * - Transfer function waveshaping
 *
 * These nonlinearities are what create the "analog warmth" and
 * distinctive visual effects of classic video synthesizers.
 */

import { type Signal, type Module, type Port, type Parameter, clamp } from '../core/types.js';

/**
 * Folding Amplifier
 *
 * Based on the Paik/Abe nonlinear processing amplifier.
 * Low-level signals pass through linearly, but signals above
 * a threshold "fold over" (invert polarity). This creates
 * the characteristic solarization effect.
 *
 * Multiple fold stages create complex waveshaping.
 */
export class FoldingAmplifier implements Module {
  readonly type = 'folder';
  readonly name = 'Folding Amplifier';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'in', type: 'signal', defaultValue: 0, description: 'Signal input' },
    { name: 'cv', type: 'cv', defaultValue: 0, description: 'Fold amount CV' },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'out', type: 'signal', defaultValue: 0, description: 'Folded signal output' },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'gain', defaultValue: 1, min: 0.1, max: 10, description: 'Input gain before folding' },
    { name: 'folds', defaultValue: 1, min: 0, max: 5, description: 'Number of fold stages' },
    { name: 'symmetry', defaultValue: 0.5, min: 0, max: 1, description: 'Fold symmetry (0.5 = centered)' },
    { name: 'cvAmount', defaultValue: 1, min: 0, max: 1, description: 'CV modulation depth' },
    { name: 'offset', defaultValue: 0, min: -1, max: 1, description: 'DC offset before folding' },
  ];

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const input = inputs.in ?? 0;
    const cv = inputs.cv ?? 0;

    const gain = params.gain ?? 1;
    const folds = params.folds ?? 1;
    const symmetry = params.symmetry ?? 0.5;
    const cvAmount = params.cvAmount ?? 1;
    const offset = params.offset ?? 0;

    // Apply gain and offset
    const modulatedGain = gain * (1 + cv * cvAmount);
    let signal = (input + offset) * modulatedGain;

    // Apply folding
    // The fold operation: when signal goes above 1 or below 0, it "bounces" back
    for (let i = 0; i < folds; i++) {
      signal = this.fold(signal, symmetry);
    }

    return { out: clamp(signal) };
  }

  /**
   * Single fold operation
   */
  private fold(value: number, symmetry: number): number {
    const threshold = symmetry;

    // Fold above upper threshold
    if (value > 1) {
      const excess = value - 1;
      value = 1 - excess;
    }

    // Fold below lower threshold
    if (value < 0) {
      const deficit = -value;
      value = deficit;
    }

    // Apply symmetry-based folding around center point
    if (symmetry !== 0.5) {
      const center = symmetry;
      if (value > center) {
        const dist = value - center;
        if (dist > (1 - center)) {
          value = center - (dist - (1 - center));
        }
      } else {
        const dist = center - value;
        if (dist > center) {
          value = center + (dist - center);
        }
      }
    }

    return value;
  }

  reset(): void {
    // Stateless
  }
}

/**
 * Soft Clipper / Saturator
 *
 * Provides gentle saturation using various transfer functions.
 * Models the soft clipping behavior of tubes and some solid-state circuits.
 */
export class SoftClipper implements Module {
  readonly type = 'soft-clip';
  readonly name = 'Soft Clipper';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'in', type: 'signal', defaultValue: 0 },
    { name: 'cv', type: 'cv', defaultValue: 0, description: 'Drive CV' },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'out', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'drive', defaultValue: 1, min: 0.1, max: 10, description: 'Saturation drive' },
    { name: 'mode', defaultValue: 0, min: 0, max: 3, step: 1, description: '0=tanh, 1=atan, 2=soft, 3=cubic' },
    { name: 'asymmetry', defaultValue: 0, min: -1, max: 1, description: 'Asymmetric clipping' },
    { name: 'mix', defaultValue: 1, min: 0, max: 1, description: 'Dry/wet mix' },
  ];

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const input = inputs.in ?? 0;
    const cv = inputs.cv ?? 0;

    const drive = (params.drive ?? 1) * (1 + cv * 2);
    const mode = Math.round(params.mode ?? 0);
    const asymmetry = params.asymmetry ?? 0;
    const mix = params.mix ?? 1;

    // Convert to bipolar for processing (-1 to 1)
    const bipolar = input * 2 - 1;

    // Apply drive
    let driven = bipolar * drive;

    // Apply asymmetry (different drive for positive vs negative)
    if (asymmetry !== 0) {
      if (driven > 0) {
        driven *= (1 + asymmetry);
      } else {
        driven *= (1 - asymmetry);
      }
    }

    // Apply saturation curve
    let saturated: number;
    switch (mode) {
      case 0: // tanh - smooth, classic tube-like
        saturated = Math.tanh(driven);
        break;

      case 1: // atan - sharper knee
        saturated = (2 / Math.PI) * Math.atan(driven * Math.PI / 2);
        break;

      case 2: // Soft algebraic: x / (1 + |x|)
        saturated = driven / (1 + Math.abs(driven));
        break;

      case 3: // Cubic soft clip
        if (driven > 1) {
          saturated = 2 / 3;
        } else if (driven < -1) {
          saturated = -2 / 3;
        } else {
          saturated = driven - (driven * driven * driven) / 3;
        }
        saturated *= 1.5; // Normalize
        break;

      default:
        saturated = Math.tanh(driven);
    }

    // Convert back to unipolar and apply mix
    const unipolar = (saturated + 1) / 2;
    const output = input * (1 - mix) + unipolar * mix;

    return { out: clamp(output) };
  }

  reset(): void {
    // Stateless
  }
}

/**
 * Hard Clipper
 *
 * Simple hard clipping with adjustable threshold.
 * Creates harsh digital-style clipping.
 */
export class HardClipper implements Module {
  readonly type = 'hard-clip';
  readonly name = 'Hard Clipper';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'in', type: 'signal', defaultValue: 0 },
    { name: 'thresholdCv', type: 'cv', defaultValue: 0 },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'out', type: 'signal', defaultValue: 0 },
    { name: 'clip', type: 'signal', defaultValue: 0, description: 'Clipping indicator' },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'low', defaultValue: 0, min: 0, max: 0.5, description: 'Low threshold' },
    { name: 'high', defaultValue: 1, min: 0.5, max: 1, description: 'High threshold' },
    { name: 'gain', defaultValue: 1, min: 0.1, max: 5, description: 'Input gain' },
  ];

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const input = inputs.in ?? 0;
    const thresholdCv = inputs.thresholdCv ?? 0;

    let low = params.low ?? 0;
    let high = params.high ?? 1;
    const gain = params.gain ?? 1;

    // Modulate thresholds with CV
    low = Math.max(0, low - thresholdCv * 0.2);
    high = Math.min(1, high + thresholdCv * 0.2);

    const amplified = input * gain;
    const clipped = clamp(amplified, low, high);

    // Clipping indicator
    const isClipping = amplified < low || amplified > high ? 1 : 0;

    return {
      out: clipped,
      clip: isClipping,
    };
  }

  reset(): void {
    // Stateless
  }
}

/**
 * Transfer Function Waveshaper
 *
 * Arbitrary waveshaping using a lookup table transfer function.
 * Can load custom curves for unique effects.
 */
export class TransferWaveshaper implements Module {
  readonly type = 'transfer';
  readonly name = 'Transfer Function';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'in', type: 'signal', defaultValue: 0 },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'out', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'curveType', defaultValue: 0, min: 0, max: 6, step: 1,
      description: '0=linear, 1=sqrt, 2=square, 3=scurve, 4=stairs, 5=sine, 6=invert' },
    { name: 'intensity', defaultValue: 1, min: 0, max: 2, description: 'Effect intensity' },
  ];

  // Lookup table for custom curves
  private lut: Float32Array = new Float32Array(256);
  private currentCurve: number = -1;

  constructor() {
    this.buildLUT(0);
  }

  private buildLUT(curveType: number): void {
    if (curveType === this.currentCurve) return;
    this.currentCurve = curveType;

    for (let i = 0; i < 256; i++) {
      const x = i / 255;
      let y: number;

      switch (curveType) {
        case 0: // Linear (pass-through)
          y = x;
          break;

        case 1: // Square root (expand darks)
          y = Math.sqrt(x);
          break;

        case 2: // Square (compress darks)
          y = x * x;
          break;

        case 3: // S-curve (increase contrast)
          y = x * x * (3 - 2 * x);
          break;

        case 4: // Stairs (posterize)
          y = Math.floor(x * 8) / 8;
          break;

        case 5: // Sine (wave distortion)
          y = (Math.sin((x - 0.5) * Math.PI) + 1) / 2;
          break;

        case 6: // Invert
          y = 1 - x;
          break;

        default:
          y = x;
      }

      this.lut[i] = y;
    }
  }

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const input = inputs.in ?? 0;
    const curveType = Math.round(params.curveType ?? 0);
    const intensity = params.intensity ?? 1;

    // Rebuild LUT if curve type changed
    this.buildLUT(curveType);

    // Look up value with linear interpolation
    const index = clamp(input) * 255;
    const indexLow = Math.floor(index);
    const indexHigh = Math.min(255, indexLow + 1);
    const frac = index - indexLow;

    const lutValue = this.lut[indexLow] * (1 - frac) + this.lut[indexHigh] * frac;

    // Mix with original based on intensity
    const output = input * (1 - intensity) + lutValue * intensity;

    return { out: clamp(output) };
  }

  reset(): void {
    this.currentCurve = -1;
  }

  /**
   * Set custom LUT values
   */
  setCustomLUT(values: number[]): void {
    const length = Math.min(values.length, 256);
    for (let i = 0; i < length; i++) {
      this.lut[i] = values[i];
    }
    // Fill remaining with last value
    for (let i = length; i < 256; i++) {
      this.lut[i] = this.lut[length - 1];
    }
    this.currentCurve = -2; // Custom
  }
}

/**
 * Quantizer
 *
 * Reduces signal to discrete levels, creating posterization effects.
 * Based on Sandin Amplitude Classifier concept.
 */
export class Quantizer implements Module {
  readonly type = 'quantizer';
  readonly name = 'Quantizer';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'in', type: 'signal', defaultValue: 0 },
    { name: 'levelsCv', type: 'cv', defaultValue: 0 },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'out', type: 'signal', defaultValue: 0 },
    { name: 'level', type: 'signal', defaultValue: 0, description: 'Current level (0-1 normalized)' },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'levels', defaultValue: 8, min: 2, max: 32, step: 1, description: 'Number of quantization levels' },
    { name: 'mode', defaultValue: 0, min: 0, max: 2, step: 1, description: '0=floor, 1=round, 2=ceil' },
    { name: 'smooth', defaultValue: 0, min: 0, max: 1, description: 'Smoothing between levels' },
  ];

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const input = inputs.in ?? 0;
    const levelsCv = inputs.levelsCv ?? 0;

    const levels = Math.max(2, Math.round((params.levels ?? 8) + levelsCv * 8));
    const mode = Math.round(params.mode ?? 0);
    const smooth = params.smooth ?? 0;

    // Quantize
    const scaled = clamp(input) * (levels - 1);
    let quantized: number;

    switch (mode) {
      case 0: // Floor
        quantized = Math.floor(scaled);
        break;
      case 1: // Round
        quantized = Math.round(scaled);
        break;
      case 2: // Ceil
        quantized = Math.ceil(scaled);
        break;
      default:
        quantized = Math.round(scaled);
    }

    // Optional smoothing (interpolate toward actual value)
    if (smooth > 0) {
      quantized = quantized * (1 - smooth) + scaled * smooth;
    }

    const output = quantized / (levels - 1);
    const levelOut = quantized / (levels - 1);

    return {
      out: clamp(output),
      level: levelOut,
    };
  }

  reset(): void {
    // Stateless
  }
}

/**
 * Solarizer
 *
 * Classic solarization effect combining threshold detection
 * with polarity inversion. Based on Paik/Abe and photographic techniques.
 */
export class Solarizer implements Module {
  readonly type = 'solarize';
  readonly name = 'Solarizer';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'in', type: 'signal', defaultValue: 0 },
    { name: 'thresholdCv', type: 'cv', defaultValue: 0 },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'out', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'threshold', defaultValue: 0.5, min: 0, max: 1, description: 'Solarization threshold' },
    { name: 'intensity', defaultValue: 1, min: 0, max: 1, description: 'Effect intensity' },
    { name: 'mode', defaultValue: 0, min: 0, max: 2, step: 1,
      description: '0=above, 1=below, 2=both' },
  ];

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const input = inputs.in ?? 0;
    const thresholdCv = inputs.thresholdCv ?? 0;

    const threshold = clamp((params.threshold ?? 0.5) + thresholdCv * 0.3);
    const intensity = params.intensity ?? 1;
    const mode = Math.round(params.mode ?? 0);

    let output = input;

    switch (mode) {
      case 0: // Invert above threshold
        if (input > threshold) {
          const inverted = threshold - (input - threshold);
          output = input * (1 - intensity) + inverted * intensity;
        }
        break;

      case 1: // Invert below threshold
        if (input < threshold) {
          const inverted = threshold + (threshold - input);
          output = input * (1 - intensity) + inverted * intensity;
        }
        break;

      case 2: // Full solarization (both directions)
        if (input > threshold) {
          const inverted = threshold - (input - threshold);
          output = input * (1 - intensity) + inverted * intensity;
        } else {
          const inverted = threshold + (threshold - input);
          output = input * (1 - intensity) + inverted * intensity;
        }
        break;
    }

    return { out: clamp(output) };
  }

  reset(): void {
    // Stateless
  }
}

/**
 * Utility functions for waveshaping
 */

/**
 * Soft clipping using tanh
 */
export function softClip(x: number, drive: number = 1): number {
  return Math.tanh(x * drive);
}

/**
 * SIMD-friendly soft clipping: x / (1 + |x|)
 */
export function fastSoftClip(x: number): number {
  return x / (1 + Math.abs(x));
}

/**
 * Chebyshev polynomial waveshaping for harmonic generation
 */
export function chebyshev(x: number, order: number): number {
  // Chebyshev polynomials of the first kind
  switch (order) {
    case 0: return 1;
    case 1: return x;
    case 2: return 2 * x * x - 1;
    case 3: return 4 * x * x * x - 3 * x;
    case 4: return 8 * x * x * x * x - 8 * x * x + 1;
    case 5: return 16 * x * x * x * x * x - 20 * x * x * x + 5 * x;
    default: {
      // Recursive definition for higher orders
      let t0 = 1;
      let t1 = x;
      for (let i = 2; i <= order; i++) {
        const t2 = 2 * x * t1 - t0;
        t0 = t1;
        t1 = t2;
      }
      return t1;
    }
  }
}
