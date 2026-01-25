/**
 * Mixing and Compositing Modules
 *
 * Video signal mixing, keying, and compositing based on classic
 * analog techniques:
 *
 * - Additive mixing (signal summing)
 * - Multiplicative mixing (modulation)
 * - Luminance keying
 * - Chroma keying
 * - Fades and transitions
 */

import {
  type Signal,
  type RGB,
  type FrameBuffer,
  type Module,
  type Port,
  type Parameter,
  clamp,
  lerp,
  createFrameBuffer,
  getPixel,
  setPixel,
} from '../core/types.js';

/**
 * Two-Input Video Mixer
 *
 * Basic crossfader between two video signals with multiple blend modes.
 */
export class VideoMixer implements Module {
  readonly type = 'video-mixer';
  readonly name = 'Video Mixer';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'a_r', type: 'signal', defaultValue: 0, description: 'Input A Red' },
    { name: 'a_g', type: 'signal', defaultValue: 0, description: 'Input A Green' },
    { name: 'a_b', type: 'signal', defaultValue: 0, description: 'Input A Blue' },
    { name: 'b_r', type: 'signal', defaultValue: 0, description: 'Input B Red' },
    { name: 'b_g', type: 'signal', defaultValue: 0, description: 'Input B Green' },
    { name: 'b_b', type: 'signal', defaultValue: 0, description: 'Input B Blue' },
    { name: 'mixCv', type: 'cv', defaultValue: 0, description: 'Mix amount CV' },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'mix', defaultValue: 0.5, min: 0, max: 1, description: 'Crossfade (0=A, 1=B)' },
    { name: 'mode', defaultValue: 0, min: 0, max: 6, step: 1,
      description: '0=blend, 1=add, 2=multiply, 3=screen, 4=difference, 5=max, 6=min' },
    { name: 'aGain', defaultValue: 1, min: 0, max: 2, description: 'Input A gain' },
    { name: 'bGain', defaultValue: 1, min: 0, max: 2, description: 'Input B gain' },
  ];

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const aR = (inputs.a_r ?? 0) * (params.aGain ?? 1);
    const aG = (inputs.a_g ?? 0) * (params.aGain ?? 1);
    const aB = (inputs.a_b ?? 0) * (params.aGain ?? 1);

    const bR = (inputs.b_r ?? 0) * (params.bGain ?? 1);
    const bG = (inputs.b_g ?? 0) * (params.bGain ?? 1);
    const bB = (inputs.b_b ?? 0) * (params.bGain ?? 1);

    const mixCv = inputs.mixCv ?? 0;
    const mix = clamp((params.mix ?? 0.5) + mixCv * 0.5);
    const mode = Math.round(params.mode ?? 0);

    let r: number, g: number, b: number;

    switch (mode) {
      case 0: // Blend (linear crossfade)
        r = lerp(aR, bR, mix);
        g = lerp(aG, bG, mix);
        b = lerp(aB, bB, mix);
        break;

      case 1: // Add
        r = aR + bR * mix;
        g = aG + bG * mix;
        b = aB + bB * mix;
        break;

      case 2: // Multiply
        r = lerp(aR, aR * bR, mix);
        g = lerp(aG, aG * bG, mix);
        b = lerp(aB, aB * bB, mix);
        break;

      case 3: // Screen
        r = lerp(aR, 1 - (1 - aR) * (1 - bR), mix);
        g = lerp(aG, 1 - (1 - aG) * (1 - bG), mix);
        b = lerp(aB, 1 - (1 - aB) * (1 - bB), mix);
        break;

      case 4: // Difference
        r = lerp(aR, Math.abs(aR - bR), mix);
        g = lerp(aG, Math.abs(aG - bG), mix);
        b = lerp(aB, Math.abs(aB - bB), mix);
        break;

      case 5: // Max (lighten)
        r = lerp(aR, Math.max(aR, bR), mix);
        g = lerp(aG, Math.max(aG, bG), mix);
        b = lerp(aB, Math.max(aB, bB), mix);
        break;

      case 6: // Min (darken)
        r = lerp(aR, Math.min(aR, bR), mix);
        g = lerp(aG, Math.min(aG, bG), mix);
        b = lerp(aB, Math.min(aB, bB), mix);
        break;

      default:
        r = lerp(aR, bR, mix);
        g = lerp(aG, bG, mix);
        b = lerp(aB, bB, mix);
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
 * Luminance Keyer
 *
 * Creates a key (matte) signal from luminance for compositing.
 * Soft/hard key edges controllable via gain.
 */
export class LumaKeyer implements Module {
  readonly type = 'luma-keyer';
  readonly name = 'Luminance Keyer';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'fg_r', type: 'signal', defaultValue: 0, description: 'Foreground Red' },
    { name: 'fg_g', type: 'signal', defaultValue: 0, description: 'Foreground Green' },
    { name: 'fg_b', type: 'signal', defaultValue: 0, description: 'Foreground Blue' },
    { name: 'bg_r', type: 'signal', defaultValue: 0, description: 'Background Red' },
    { name: 'bg_g', type: 'signal', defaultValue: 0, description: 'Background Green' },
    { name: 'bg_b', type: 'signal', defaultValue: 0, description: 'Background Blue' },
    { name: 'key', type: 'signal', defaultValue: 0, description: 'External key input (optional)' },
    { name: 'thresholdCv', type: 'cv', defaultValue: 0 },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
    { name: 'key', type: 'signal', defaultValue: 0, description: 'Generated key signal' },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'threshold', defaultValue: 0.5, min: 0, max: 1, description: 'Key threshold' },
    { name: 'softness', defaultValue: 0.1, min: 0, max: 0.5, description: 'Edge softness' },
    { name: 'invert', defaultValue: 0, min: 0, max: 1, step: 1, description: 'Invert key' },
    { name: 'keySource', defaultValue: 0, min: 0, max: 1, step: 1,
      description: '0=foreground luma, 1=external key' },
    { name: 'gain', defaultValue: 1, min: 0, max: 2, description: 'Key gain' },
    { name: 'clip', defaultValue: 1, min: 0, max: 1, step: 1, description: 'Clip key to 0-1' },
  ];

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const fgR = inputs.fg_r ?? 0;
    const fgG = inputs.fg_g ?? 0;
    const fgB = inputs.fg_b ?? 0;

    const bgR = inputs.bg_r ?? 0;
    const bgG = inputs.bg_g ?? 0;
    const bgB = inputs.bg_b ?? 0;

    const externalKey = inputs.key ?? 0;
    const thresholdCv = inputs.thresholdCv ?? 0;

    const threshold = clamp((params.threshold ?? 0.5) + thresholdCv * 0.3);
    const softness = Math.max(0.001, params.softness ?? 0.1);
    const invert = (params.invert ?? 0) > 0.5;
    const keySource = Math.round(params.keySource ?? 0);
    const gain = params.gain ?? 1;
    const shouldClip = (params.clip ?? 1) > 0.5;

    // Determine key source
    let keyValue: number;
    if (keySource === 1) {
      keyValue = externalKey;
    } else {
      // Calculate luminance from foreground
      keyValue = 0.299 * fgR + 0.587 * fgG + 0.114 * fgB;
    }

    // Apply threshold with soft edge
    let key = smoothstep(threshold - softness, threshold + softness, keyValue);

    // Apply gain
    key *= gain;

    // Optionally clip
    if (shouldClip) {
      key = clamp(key);
    }

    // Invert if requested
    if (invert) {
      key = 1 - key;
    }

    // Composite: output = foreground * key + background * (1 - key)
    const r = fgR * key + bgR * (1 - key);
    const g = fgG * key + bgG * (1 - key);
    const b = fgB * key + bgB * (1 - key);

    return {
      r: clamp(r),
      g: clamp(g),
      b: clamp(b),
      key: clamp(key),
    };
  }

  reset(): void {
    // Stateless
  }
}

/**
 * Chroma Keyer
 *
 * Keys based on color (hue/saturation).
 * Classic "green screen" / "blue screen" effect.
 */
export class ChromaKeyer implements Module {
  readonly type = 'chroma-keyer';
  readonly name = 'Chroma Keyer';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'fg_r', type: 'signal', defaultValue: 0 },
    { name: 'fg_g', type: 'signal', defaultValue: 0 },
    { name: 'fg_b', type: 'signal', defaultValue: 0 },
    { name: 'bg_r', type: 'signal', defaultValue: 0 },
    { name: 'bg_g', type: 'signal', defaultValue: 0 },
    { name: 'bg_b', type: 'signal', defaultValue: 0 },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
    { name: 'key', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'keyHue', defaultValue: 0.33, min: 0, max: 1, description: 'Key hue (0.33 = green, 0.66 = blue)' },
    { name: 'hueRange', defaultValue: 0.1, min: 0, max: 0.5, description: 'Hue tolerance' },
    { name: 'satMin', defaultValue: 0.2, min: 0, max: 1, description: 'Minimum saturation for key' },
    { name: 'softness', defaultValue: 0.1, min: 0, max: 0.3, description: 'Edge softness' },
    { name: 'spillSuppress', defaultValue: 0.5, min: 0, max: 1, description: 'Spill suppression' },
    { name: 'invert', defaultValue: 0, min: 0, max: 1, step: 1 },
  ];

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    let fgR = inputs.fg_r ?? 0;
    let fgG = inputs.fg_g ?? 0;
    let fgB = inputs.fg_b ?? 0;

    const bgR = inputs.bg_r ?? 0;
    const bgG = inputs.bg_g ?? 0;
    const bgB = inputs.bg_b ?? 0;

    const keyHue = params.keyHue ?? 0.33;
    const hueRange = params.hueRange ?? 0.1;
    const satMin = params.satMin ?? 0.2;
    const softness = Math.max(0.001, params.softness ?? 0.1);
    const spillSuppress = params.spillSuppress ?? 0.5;
    const invert = (params.invert ?? 0) > 0.5;

    // Convert foreground to HSV
    const { h, s } = rgbToHsv({ r: fgR, g: fgG, b: fgB });

    // Calculate hue distance (circular)
    let hueDist = Math.abs(h - keyHue);
    if (hueDist > 0.5) hueDist = 1 - hueDist;

    // Key is strong when hue is close and saturation is high
    const hueKey = smoothstep(hueRange + softness, hueRange - softness, hueDist);
    const satKey = smoothstep(satMin - softness, satMin + softness, s);
    let key = hueKey * satKey;

    if (invert) {
      key = 1 - key;
    }

    // Spill suppression: reduce the key color from foreground
    if (spillSuppress > 0 && key < 1) {
      const spillAmount = (1 - key) * spillSuppress;
      if (keyHue < 0.17 || keyHue > 0.83) { // Red
        fgR = Math.max(0, fgR - spillAmount * 0.3);
      } else if (keyHue < 0.5) { // Green
        fgG = Math.max(0, fgG - spillAmount * 0.3);
      } else { // Blue
        fgB = Math.max(0, fgB - spillAmount * 0.3);
      }
    }

    // Composite
    const r = fgR * (1 - key) + bgR * key;
    const g = fgG * (1 - key) + bgG * key;
    const b = fgB * (1 - key) + bgB * key;

    return {
      r: clamp(r),
      g: clamp(g),
      b: clamp(b),
      key: clamp(key),
    };
  }

  reset(): void {
    // Stateless
  }
}

/**
 * Four-Quadrant Multiplier
 *
 * Based on the MC1445 IC used in the Sandin Image Processor.
 * Enables true multiplication of video signals (positive and negative).
 */
export class FourQuadrantMultiplier implements Module {
  readonly type = 'four-quadrant';
  readonly name = 'Four-Quadrant Multiplier';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'x', type: 'signal', defaultValue: 0, description: 'X input' },
    { name: 'y', type: 'signal', defaultValue: 0, description: 'Y input' },
    { name: 'carrier', type: 'signal', defaultValue: 0.5, description: 'Carrier/bias' },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'out', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'xGain', defaultValue: 1, min: 0, max: 2 },
    { name: 'yGain', defaultValue: 1, min: 0, max: 2 },
    { name: 'xOffset', defaultValue: 0, min: -1, max: 1, description: 'X DC offset (for bipolar operation)' },
    { name: 'yOffset', defaultValue: 0, min: -1, max: 1, description: 'Y DC offset' },
    { name: 'outputGain', defaultValue: 1, min: 0, max: 4 },
    { name: 'outputOffset', defaultValue: 0.5, min: 0, max: 1, description: 'Output DC offset' },
  ];

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const x = inputs.x ?? 0;
    const y = inputs.y ?? 0;

    const xGain = params.xGain ?? 1;
    const yGain = params.yGain ?? 1;
    const xOffset = params.xOffset ?? 0;
    const yOffset = params.yOffset ?? 0;
    const outputGain = params.outputGain ?? 1;
    const outputOffset = params.outputOffset ?? 0.5;

    // Convert to bipolar (-1 to +1)
    const xBipolar = (x * 2 - 1) * xGain + xOffset;
    const yBipolar = (y * 2 - 1) * yGain + yOffset;

    // Four-quadrant multiplication
    const product = xBipolar * yBipolar;

    // Convert back to unipolar with gain and offset
    const output = (product * outputGain + 1) / 2 + outputOffset - 0.5;

    return {
      out: clamp(output),
    };
  }

  reset(): void {
    // Stateless
  }
}

/**
 * Comparator / Hard Keyer
 *
 * Binary comparison for hard-edge keying effects.
 * Based on Sandin IP comparator module.
 */
export class Comparator implements Module {
  readonly type = 'comparator';
  readonly name = 'Comparator';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'a', type: 'signal', defaultValue: 0, description: 'Input A' },
    { name: 'b', type: 'signal', defaultValue: 0.5, description: 'Input B (threshold)' },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'out', type: 'signal', defaultValue: 0, description: '1 if A > B, else 0' },
    { name: 'inv', type: 'signal', defaultValue: 1, description: '1 if A <= B, else 0' },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'threshold', defaultValue: 0.5, min: 0, max: 1, description: 'Fixed threshold (used if B not connected)' },
    { name: 'hysteresis', defaultValue: 0, min: 0, max: 0.1, description: 'Hysteresis to prevent chattering' },
  ];

  private lastState: boolean = false;

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const a = inputs.a ?? 0;
    const b = inputs.b ?? (params.threshold ?? 0.5);
    const hysteresis = params.hysteresis ?? 0;

    let out: Signal;

    if (hysteresis > 0) {
      // Schmitt trigger behavior
      if (this.lastState) {
        out = a > (b - hysteresis) ? 1 : 0;
      } else {
        out = a > (b + hysteresis) ? 1 : 0;
      }
      this.lastState = out > 0.5;
    } else {
      out = a > b ? 1 : 0;
    }

    return {
      out,
      inv: 1 - out,
    };
  }

  reset(): void {
    this.lastState = false;
  }
}

/**
 * Additive RGB Mixer
 *
 * Simple additive mixing of multiple RGB sources.
 */
export class AdditiveRGBMixer implements Module {
  readonly type = 'additive-rgb';
  readonly name = 'Additive RGB Mixer';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'r1', type: 'signal', defaultValue: 0 },
    { name: 'g1', type: 'signal', defaultValue: 0 },
    { name: 'b1', type: 'signal', defaultValue: 0 },
    { name: 'r2', type: 'signal', defaultValue: 0 },
    { name: 'g2', type: 'signal', defaultValue: 0 },
    { name: 'b2', type: 'signal', defaultValue: 0 },
    { name: 'r3', type: 'signal', defaultValue: 0 },
    { name: 'g3', type: 'signal', defaultValue: 0 },
    { name: 'b3', type: 'signal', defaultValue: 0 },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'gain1', defaultValue: 1, min: 0, max: 2 },
    { name: 'gain2', defaultValue: 1, min: 0, max: 2 },
    { name: 'gain3', defaultValue: 1, min: 0, max: 2 },
    { name: 'masterGain', defaultValue: 1, min: 0, max: 2 },
    { name: 'clip', defaultValue: 1, min: 0, max: 1, step: 1, description: 'Clip output' },
  ];

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const gain1 = params.gain1 ?? 1;
    const gain2 = params.gain2 ?? 1;
    const gain3 = params.gain3 ?? 1;
    const masterGain = params.masterGain ?? 1;
    const shouldClip = (params.clip ?? 1) > 0.5;

    let r = (inputs.r1 ?? 0) * gain1 + (inputs.r2 ?? 0) * gain2 + (inputs.r3 ?? 0) * gain3;
    let g = (inputs.g1 ?? 0) * gain1 + (inputs.g2 ?? 0) * gain2 + (inputs.g3 ?? 0) * gain3;
    let b = (inputs.b1 ?? 0) * gain1 + (inputs.b2 ?? 0) * gain2 + (inputs.b3 ?? 0) * gain3;

    r *= masterGain;
    g *= masterGain;
    b *= masterGain;

    if (shouldClip) {
      r = clamp(r);
      g = clamp(g);
      b = clamp(b);
    }

    return { r, g, b };
  }

  reset(): void {
    // Stateless
  }
}

// ============================================
// Utility Functions
// ============================================

function smoothstep(edge0: number, edge1: number, x: number): number {
  const t = clamp((x - edge0) / (edge1 - edge0), 0, 1);
  return t * t * (3 - 2 * t);
}

function rgbToHsv(rgb: RGB): { h: number; s: number; v: number } {
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
