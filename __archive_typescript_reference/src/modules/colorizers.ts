/**
 * Colorizer Modules
 *
 * Various approaches to adding color to video signals, based on
 * classic synthesizer designs:
 *
 * - Paik/Abe: Resistive matrix colorization with nonlinear processing
 * - Jones: Diode/layered mixing with per-channel thresholds
 * - Sandin: Amplitude classifier based threshold colorization
 * - Beck: RGB synthesis from oscillators with color chords
 *
 * All colorizers work with monochrome input and produce RGB output.
 */

import {
  type Signal,
  type RGB,
  type Module,
  type Port,
  type Parameter,
  clamp,
  lerp,
} from '../core/types.js';

/**
 * Matrix Colorizer
 *
 * Based on Paik/Abe resistive summing matrix.
 * Multiple grayscale inputs are weighted and summed to RGB outputs.
 * Each input can contribute to any combination of R, G, B.
 */
export class MatrixColorizer implements Module {
  readonly type = 'matrix-colorizer';
  readonly name = 'Matrix Colorizer';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'in1', type: 'signal', defaultValue: 0, description: 'Input channel 1' },
    { name: 'in2', type: 'signal', defaultValue: 0, description: 'Input channel 2' },
    { name: 'in3', type: 'signal', defaultValue: 0, description: 'Input channel 3' },
    { name: 'in4', type: 'signal', defaultValue: 0, description: 'Input channel 4' },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    // Input 1 -> RGB weights
    { name: 'in1_r', defaultValue: 1, min: -1, max: 1, description: 'Input 1 to Red' },
    { name: 'in1_g', defaultValue: 0, min: -1, max: 1, description: 'Input 1 to Green' },
    { name: 'in1_b', defaultValue: 0, min: -1, max: 1, description: 'Input 1 to Blue' },
    // Input 2 -> RGB weights
    { name: 'in2_r', defaultValue: 0, min: -1, max: 1 },
    { name: 'in2_g', defaultValue: 1, min: -1, max: 1 },
    { name: 'in2_b', defaultValue: 0, min: -1, max: 1 },
    // Input 3 -> RGB weights
    { name: 'in3_r', defaultValue: 0, min: -1, max: 1 },
    { name: 'in3_g', defaultValue: 0, min: -1, max: 1 },
    { name: 'in3_b', defaultValue: 1, min: -1, max: 1 },
    // Input 4 -> RGB weights (yellow by default: R+G)
    { name: 'in4_r', defaultValue: 0.5, min: -1, max: 1 },
    { name: 'in4_g', defaultValue: 0.5, min: -1, max: 1 },
    { name: 'in4_b', defaultValue: 0, min: -1, max: 1 },
    // Master controls
    { name: 'gain', defaultValue: 1, min: 0, max: 2, description: 'Output gain' },
    { name: 'saturation', defaultValue: 1, min: 0, max: 2, description: 'Color saturation' },
  ];

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const in1 = inputs.in1 ?? 0;
    const in2 = inputs.in2 ?? 0;
    const in3 = inputs.in3 ?? 0;
    const in4 = inputs.in4 ?? 0;

    const gain = params.gain ?? 1;
    const saturation = params.saturation ?? 1;

    // Sum weighted inputs to each RGB channel
    let r = in1 * (params.in1_r ?? 1) +
            in2 * (params.in2_r ?? 0) +
            in3 * (params.in3_r ?? 0) +
            in4 * (params.in4_r ?? 0.5);

    let g = in1 * (params.in1_g ?? 0) +
            in2 * (params.in2_g ?? 1) +
            in3 * (params.in3_g ?? 0) +
            in4 * (params.in4_g ?? 0.5);

    let b = in1 * (params.in1_b ?? 0) +
            in2 * (params.in2_b ?? 0) +
            in3 * (params.in3_b ?? 1) +
            in4 * (params.in4_b ?? 0);

    // Apply gain
    r *= gain;
    g *= gain;
    b *= gain;

    // Apply saturation (desaturate toward luminance)
    if (saturation !== 1) {
      const luma = 0.299 * r + 0.587 * g + 0.114 * b;
      r = lerp(luma, r, saturation);
      g = lerp(luma, g, saturation);
      b = lerp(luma, b, saturation);
    }

    return {
      r: clamp(r),
      g: clamp(g),
      b: clamp(b),
    };
  }

  reset(): void {
    // Stateless
  }
}

/**
 * Threshold Colorizer (Amplitude Classifier)
 *
 * Based on Sandin Image Processor amplitude classifier.
 * Input is classified into discrete levels, each mapped to a color.
 * Creates posterized, banded color effects.
 */
export class ThresholdColorizer implements Module {
  readonly type = 'threshold-colorizer';
  readonly name = 'Threshold Colorizer';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'in', type: 'signal', defaultValue: 0, description: 'Luminance input' },
    { name: 'levelsCv', type: 'cv', defaultValue: 0, description: 'Levels modulation' },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'levels', defaultValue: 8, min: 2, max: 16, step: 1, description: 'Number of color levels' },
    { name: 'hueOffset', defaultValue: 0, min: 0, max: 1, description: 'Color palette rotation' },
    { name: 'hueRange', defaultValue: 1, min: 0, max: 2, description: 'Range of hues (1 = full spectrum)' },
    { name: 'saturation', defaultValue: 0.8, min: 0, max: 1 },
    { name: 'brightness', defaultValue: 0.9, min: 0, max: 1 },
    { name: 'softEdge', defaultValue: 0, min: 0, max: 0.2, description: 'Soften edges between levels' },
  ];

  // Predefined color palettes
  private static readonly PALETTES: RGB[][] = [
    // Rainbow
    [
      { r: 1, g: 0, b: 0 },
      { r: 1, g: 0.5, b: 0 },
      { r: 1, g: 1, b: 0 },
      { r: 0, g: 1, b: 0 },
      { r: 0, g: 1, b: 1 },
      { r: 0, g: 0, b: 1 },
      { r: 0.5, g: 0, b: 1 },
      { r: 1, g: 0, b: 1 },
    ],
  ];

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const input = inputs.in ?? 0;
    const levelsCv = inputs.levelsCv ?? 0;

    const levels = Math.max(2, Math.round((params.levels ?? 8) + levelsCv * 4));
    const hueOffset = params.hueOffset ?? 0;
    const hueRange = params.hueRange ?? 1;
    const saturation = params.saturation ?? 0.8;
    const brightness = params.brightness ?? 0.9;
    const softEdge = params.softEdge ?? 0;

    // Determine which level the input falls into
    const normalizedInput = clamp(input);
    const levelFloat = normalizedInput * (levels - 1);
    const levelIndex = Math.floor(levelFloat);
    const levelFrac = levelFloat - levelIndex;

    // Calculate hue for this level
    const hue = (hueOffset + (levelIndex / (levels - 1)) * hueRange) % 1;

    // Convert HSV to RGB
    let color = hsvToRgb(hue, saturation, brightness);

    // Optional soft edge interpolation to next level
    if (softEdge > 0 && levelIndex < levels - 1 && levelFrac > (1 - softEdge)) {
      const nextHue = (hueOffset + ((levelIndex + 1) / (levels - 1)) * hueRange) % 1;
      const nextColor = hsvToRgb(nextHue, saturation, brightness);
      const t = (levelFrac - (1 - softEdge)) / softEdge;

      color = {
        r: lerp(color.r, nextColor.r, t),
        g: lerp(color.g, nextColor.g, t),
        b: lerp(color.b, nextColor.b, t),
      };
    }

    return {
      r: color.r,
      g: color.g,
      b: color.b,
    };
  }

  reset(): void {
    // Stateless
  }
}

/**
 * Layered Colorizer (Jones Style)
 *
 * Based on Dave Jones colorizer with diode/layered mixing.
 * Multiple channels compete, with the brightest winning.
 * Creates hard boundaries between color zones.
 */
export class LayeredColorizer implements Module {
  readonly type = 'layered-colorizer';
  readonly name = 'Layered Colorizer';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'in', type: 'signal', defaultValue: 0, description: 'Main luminance input' },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    // Channel 1 (shadows)
    { name: 'ch1_clip', defaultValue: 0.25, min: 0, max: 1, description: 'Channel 1 threshold' },
    { name: 'ch1_r', defaultValue: 0.2, min: 0, max: 1 },
    { name: 'ch1_g', defaultValue: 0, min: 0, max: 1 },
    { name: 'ch1_b', defaultValue: 0.4, min: 0, max: 1 },
    { name: 'ch1_polarity', defaultValue: 0, min: 0, max: 1, step: 1, description: '0=normal, 1=inverted' },
    // Channel 2 (midtones)
    { name: 'ch2_clip', defaultValue: 0.5, min: 0, max: 1 },
    { name: 'ch2_r', defaultValue: 0, min: 0, max: 1 },
    { name: 'ch2_g', defaultValue: 0.6, min: 0, max: 1 },
    { name: 'ch2_b', defaultValue: 0.2, min: 0, max: 1 },
    { name: 'ch2_polarity', defaultValue: 0, min: 0, max: 1, step: 1 },
    // Channel 3 (highlights)
    { name: 'ch3_clip', defaultValue: 0.75, min: 0, max: 1 },
    { name: 'ch3_r', defaultValue: 1, min: 0, max: 1 },
    { name: 'ch3_g', defaultValue: 0.8, min: 0, max: 1 },
    { name: 'ch3_b', defaultValue: 0, min: 0, max: 1 },
    { name: 'ch3_polarity', defaultValue: 0, min: 0, max: 1, step: 1 },
    // Global
    { name: 'softness', defaultValue: 0.02, min: 0, max: 0.2, description: 'Edge softness' },
  ];

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    let input = inputs.in ?? 0;
    input = clamp(input);

    const softness = Math.max(0.001, params.softness ?? 0.02);

    // Process each channel
    const channels = [
      this.processChannel(input, params, 'ch1'),
      this.processChannel(input, params, 'ch2'),
      this.processChannel(input, params, 'ch3'),
    ];

    // Diode mixing: take maximum of each RGB channel
    // This creates the characteristic hard boundaries of Jones colorizer
    let r = 0, g = 0, b = 0;
    for (const ch of channels) {
      r = Math.max(r, ch.r * ch.key);
      g = Math.max(g, ch.g * ch.key);
      b = Math.max(b, ch.b * ch.key);
    }

    return {
      r: clamp(r),
      g: clamp(g),
      b: clamp(b),
    };
  }

  private processChannel(
    input: number,
    params: Record<string, number>,
    prefix: string
  ): { r: number; g: number; b: number; key: number } {
    const clip = params[`${prefix}_clip`] ?? 0.5;
    const r = params[`${prefix}_r`] ?? 0;
    const g = params[`${prefix}_g`] ?? 0;
    const b = params[`${prefix}_b`] ?? 0;
    const polarity = params[`${prefix}_polarity`] ?? 0;
    const softness = Math.max(0.001, params.softness ?? 0.02);

    // Apply polarity
    const processedInput = polarity > 0.5 ? 1 - input : input;

    // Create key based on threshold with soft edge
    const key = smoothstep(clip - softness, clip + softness, processedInput);

    return { r, g, b, key };
  }

  reset(): void {
    // Stateless
  }
}

/**
 * RGB Encoder
 *
 * Simple utility to combine separate R, G, B signals.
 * Includes hue rotation and saturation controls.
 */
export class RGBEncoder implements Module {
  readonly type = 'rgb-encoder';
  readonly name = 'RGB Encoder';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
    { name: 'hueCv', type: 'cv', defaultValue: 0, description: 'Hue rotation CV' },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
    { name: 'y', type: 'signal', defaultValue: 0, description: 'Luminance output' },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'hue', defaultValue: 0, min: 0, max: 1, description: 'Hue rotation (0-1 = 0-360°)' },
    { name: 'saturation', defaultValue: 1, min: 0, max: 2, description: 'Saturation' },
    { name: 'brightness', defaultValue: 1, min: 0, max: 2, description: 'Brightness' },
    { name: 'rGain', defaultValue: 1, min: 0, max: 2 },
    { name: 'gGain', defaultValue: 1, min: 0, max: 2 },
    { name: 'bGain', defaultValue: 1, min: 0, max: 2 },
  ];

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    let r = (inputs.r ?? 0) * (params.rGain ?? 1);
    let g = (inputs.g ?? 0) * (params.gGain ?? 1);
    let b = (inputs.b ?? 0) * (params.bGain ?? 1);

    const hueCv = inputs.hueCv ?? 0;
    const hueRotation = ((params.hue ?? 0) + hueCv) % 1;
    const saturation = params.saturation ?? 1;
    const brightness = params.brightness ?? 1;

    // Convert to HSV
    let { h, s, v } = rgbToHsv({ r, g, b });

    // Apply adjustments
    h = (h + hueRotation) % 1;
    s = clamp(s * saturation);
    v = clamp(v * brightness);

    // Convert back to RGB
    const result = hsvToRgb(h, s, v);

    // Calculate luminance
    const y = 0.299 * result.r + 0.587 * result.g + 0.114 * result.b;

    return {
      r: clamp(result.r),
      g: clamp(result.g),
      b: clamp(result.b),
      y: clamp(y),
    };
  }

  reset(): void {
    // Stateless
  }
}

/**
 * Gradient Mapper
 *
 * Maps luminance to a configurable color gradient.
 * Useful for heat maps, false color, and artistic effects.
 */
export class GradientMapper implements Module {
  readonly type = 'gradient-mapper';
  readonly name = 'Gradient Mapper';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'in', type: 'signal', defaultValue: 0 },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'preset', defaultValue: 0, min: 0, max: 5, step: 1,
      description: '0=heat, 1=cool, 2=spectrum, 3=fire, 4=electric, 5=custom' },
    // Custom gradient control points
    { name: 'color1_pos', defaultValue: 0, min: 0, max: 1 },
    { name: 'color1_r', defaultValue: 0, min: 0, max: 1 },
    { name: 'color1_g', defaultValue: 0, min: 0, max: 1 },
    { name: 'color1_b', defaultValue: 0, min: 0, max: 1 },
    { name: 'color2_pos', defaultValue: 0.5, min: 0, max: 1 },
    { name: 'color2_r', defaultValue: 0.5, min: 0, max: 1 },
    { name: 'color2_g', defaultValue: 0.5, min: 0, max: 1 },
    { name: 'color2_b', defaultValue: 0.5, min: 0, max: 1 },
    { name: 'color3_pos', defaultValue: 1, min: 0, max: 1 },
    { name: 'color3_r', defaultValue: 1, min: 0, max: 1 },
    { name: 'color3_g', defaultValue: 1, min: 0, max: 1 },
    { name: 'color3_b', defaultValue: 1, min: 0, max: 1 },
  ];

  // Preset gradients
  private static readonly PRESETS: Array<Array<{ pos: number; color: RGB }>> = [
    // Heat (black -> red -> yellow -> white)
    [
      { pos: 0, color: { r: 0, g: 0, b: 0 } },
      { pos: 0.33, color: { r: 0.8, g: 0, b: 0 } },
      { pos: 0.66, color: { r: 1, g: 0.8, b: 0 } },
      { pos: 1, color: { r: 1, g: 1, b: 1 } },
    ],
    // Cool (black -> blue -> cyan -> white)
    [
      { pos: 0, color: { r: 0, g: 0, b: 0 } },
      { pos: 0.33, color: { r: 0, g: 0, b: 0.8 } },
      { pos: 0.66, color: { r: 0, g: 0.8, b: 1 } },
      { pos: 1, color: { r: 1, g: 1, b: 1 } },
    ],
    // Spectrum (rainbow)
    [
      { pos: 0, color: { r: 1, g: 0, b: 0 } },
      { pos: 0.2, color: { r: 1, g: 1, b: 0 } },
      { pos: 0.4, color: { r: 0, g: 1, b: 0 } },
      { pos: 0.6, color: { r: 0, g: 1, b: 1 } },
      { pos: 0.8, color: { r: 0, g: 0, b: 1 } },
      { pos: 1, color: { r: 1, g: 0, b: 1 } },
    ],
    // Fire
    [
      { pos: 0, color: { r: 0, g: 0, b: 0 } },
      { pos: 0.25, color: { r: 0.5, g: 0, b: 0 } },
      { pos: 0.5, color: { r: 1, g: 0.3, b: 0 } },
      { pos: 0.75, color: { r: 1, g: 0.7, b: 0 } },
      { pos: 1, color: { r: 1, g: 1, b: 0.5 } },
    ],
    // Electric
    [
      { pos: 0, color: { r: 0, g: 0, b: 0.2 } },
      { pos: 0.3, color: { r: 0.2, g: 0, b: 0.8 } },
      { pos: 0.6, color: { r: 0.5, g: 0.2, b: 1 } },
      { pos: 0.8, color: { r: 0.8, g: 0.8, b: 1 } },
      { pos: 1, color: { r: 1, g: 1, b: 1 } },
    ],
  ];

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const input = clamp(inputs.in ?? 0);
    const presetIndex = Math.round(params.preset ?? 0);

    let gradient: Array<{ pos: number; color: RGB }>;

    if (presetIndex < GradientMapper.PRESETS.length) {
      gradient = GradientMapper.PRESETS[presetIndex];
    } else {
      // Custom gradient from parameters
      gradient = [
        {
          pos: params.color1_pos ?? 0,
          color: {
            r: params.color1_r ?? 0,
            g: params.color1_g ?? 0,
            b: params.color1_b ?? 0,
          },
        },
        {
          pos: params.color2_pos ?? 0.5,
          color: {
            r: params.color2_r ?? 0.5,
            g: params.color2_g ?? 0.5,
            b: params.color2_b ?? 0.5,
          },
        },
        {
          pos: params.color3_pos ?? 1,
          color: {
            r: params.color3_r ?? 1,
            g: params.color3_g ?? 1,
            b: params.color3_b ?? 1,
          },
        },
      ];
    }

    // Find the two gradient stops to interpolate between
    const color = this.sampleGradient(gradient, input);

    return {
      r: color.r,
      g: color.g,
      b: color.b,
    };
  }

  private sampleGradient(
    gradient: Array<{ pos: number; color: RGB }>,
    t: number
  ): RGB {
    // Sort by position
    const sorted = [...gradient].sort((a, b) => a.pos - b.pos);

    // Find bracketing stops
    let lower = sorted[0];
    let upper = sorted[sorted.length - 1];

    for (let i = 0; i < sorted.length - 1; i++) {
      if (sorted[i].pos <= t && sorted[i + 1].pos >= t) {
        lower = sorted[i];
        upper = sorted[i + 1];
        break;
      }
    }

    // Interpolate
    const range = upper.pos - lower.pos;
    const frac = range > 0 ? (t - lower.pos) / range : 0;

    return {
      r: lerp(lower.color.r, upper.color.r, frac),
      g: lerp(lower.color.g, upper.color.g, frac),
      b: lerp(lower.color.b, upper.color.b, frac),
    };
  }

  reset(): void {
    // Stateless
  }
}

/**
 * Color Chord Generator (Beck Style)
 *
 * Based on Stephen Beck's color chord system.
 * 6 knobs per channel (3 non-inverting + 3 inverting RGB amplifiers)
 * Creates complex color mixtures from simple inputs.
 */
export class ColorChord implements Module {
  readonly type = 'color-chord';
  readonly name = 'Color Chord';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'in1', type: 'signal', defaultValue: 0 },
    { name: 'in2', type: 'signal', defaultValue: 0 },
    { name: 'in3', type: 'signal', defaultValue: 0 },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    // Positive (non-inverting) gains
    { name: 'r_pos', defaultValue: 1, min: 0, max: 2, description: 'Red positive gain' },
    { name: 'g_pos', defaultValue: 1, min: 0, max: 2, description: 'Green positive gain' },
    { name: 'b_pos', defaultValue: 1, min: 0, max: 2, description: 'Blue positive gain' },
    // Negative (inverting) gains
    { name: 'r_neg', defaultValue: 0, min: 0, max: 2, description: 'Red negative gain' },
    { name: 'g_neg', defaultValue: 0, min: 0, max: 2, description: 'Green negative gain' },
    { name: 'b_neg', defaultValue: 0, min: 0, max: 2, description: 'Blue negative gain' },
    // Input routing
    { name: 'in1_to_r', defaultValue: 1, min: 0, max: 1 },
    { name: 'in2_to_g', defaultValue: 1, min: 0, max: 1 },
    { name: 'in3_to_b', defaultValue: 1, min: 0, max: 1 },
  ];

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const in1 = inputs.in1 ?? 0;
    const in2 = inputs.in2 ?? 0;
    const in3 = inputs.in3 ?? 0;

    // Route inputs to channels
    const rInput = in1 * (params.in1_to_r ?? 1);
    const gInput = in2 * (params.in2_to_g ?? 1);
    const bInput = in3 * (params.in3_to_b ?? 1);

    // Apply positive and negative gains
    // This allows "illegal" color values that create interesting effects
    const r = rInput * (params.r_pos ?? 1) - (1 - rInput) * (params.r_neg ?? 0);
    const g = gInput * (params.g_pos ?? 1) - (1 - gInput) * (params.g_neg ?? 0);
    const b = bInput * (params.b_pos ?? 1) - (1 - bInput) * (params.b_neg ?? 0);

    return {
      r: clamp(r),
      g: clamp(g),
      b: clamp(b),
    };
  }

  reset(): void {
    // Stateless
  }
}

// ============================================
// Color Space Utility Functions
// ============================================

/**
 * Convert RGB to HSV
 */
export function rgbToHsv(rgb: RGB): { h: number; s: number; v: number } {
  const r = rgb.r;
  const g = rgb.g;
  const b = rgb.b;

  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const delta = max - min;

  let h = 0;
  const s = max === 0 ? 0 : delta / max;
  const v = max;

  if (delta !== 0) {
    if (max === r) {
      h = ((g - b) / delta) % 6;
    } else if (max === g) {
      h = (b - r) / delta + 2;
    } else {
      h = (r - g) / delta + 4;
    }
    h /= 6;
    if (h < 0) h += 1;
  }

  return { h, s, v };
}

/**
 * Convert HSV to RGB
 */
export function hsvToRgb(h: number, s: number, v: number): RGB {
  const i = Math.floor(h * 6);
  const f = h * 6 - i;
  const p = v * (1 - s);
  const q = v * (1 - f * s);
  const t = v * (1 - (1 - f) * s);

  let r: number, g: number, b: number;

  switch (i % 6) {
    case 0: r = v; g = t; b = p; break;
    case 1: r = q; g = v; b = p; break;
    case 2: r = p; g = v; b = t; break;
    case 3: r = p; g = q; b = v; break;
    case 4: r = t; g = p; b = v; break;
    case 5: r = v; g = p; b = q; break;
    default: r = 0; g = 0; b = 0;
  }

  return { r, g, b };
}

/**
 * Smoothstep function
 */
function smoothstep(edge0: number, edge1: number, x: number): number {
  const t = clamp((x - edge0) / (edge1 - edge0), 0, 1);
  return t * t * (3 - 2 * t);
}
