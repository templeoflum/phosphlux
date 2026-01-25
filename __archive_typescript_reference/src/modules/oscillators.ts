/**
 * Oscillator Modules
 *
 * Core waveform generators for video synthesis. Based on classic analog
 * oscillator designs with both audio-rate and video-rate capabilities.
 *
 * LZX-style oscillators operate at video rate (MHz range) to create
 * stable geometric patterns locked to sync.
 */

import { type Signal, type Module, type Port, type Parameter, clamp } from '../core/types.js';

/**
 * Basic waveform types available in oscillators
 */
export type WaveformType = 'sine' | 'triangle' | 'sawtooth' | 'ramp' | 'square' | 'pulse';

/**
 * Low Frequency Oscillator (LFO)
 *
 * Audio/control rate oscillator for modulation purposes.
 * Frequency range: 0.01 Hz to 100 Hz typical
 */
export class LFO implements Module {
  readonly type = 'lfo';
  readonly name = 'LFO';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'fm', type: 'cv', defaultValue: 0, description: 'Frequency modulation input' },
    { name: 'reset', type: 'trigger', defaultValue: 0, description: 'Reset phase to 0' },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'sine', type: 'signal', defaultValue: 0 },
    { name: 'triangle', type: 'signal', defaultValue: 0 },
    { name: 'sawtooth', type: 'signal', defaultValue: 0 },
    { name: 'square', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'frequency', defaultValue: 1, min: 0.01, max: 100, unit: 'Hz', cvModulatable: true },
    { name: 'fmAmount', defaultValue: 0, min: -1, max: 1, description: 'FM depth' },
    { name: 'phase', defaultValue: 0, min: 0, max: 1, description: 'Initial phase offset' },
    { name: 'pulseWidth', defaultValue: 0.5, min: 0.01, max: 0.99, description: 'Pulse width for square wave' },
  ];

  private phase: number = 0;
  private sampleRate: number;
  private lastReset: Signal = 0;

  constructor(sampleRate: number = 48000) {
    this.sampleRate = sampleRate;
  }

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const fm = inputs.fm ?? 0;
    const reset = inputs.reset ?? 0;

    const frequency = params.frequency ?? 1;
    const fmAmount = params.fmAmount ?? 0;
    const phaseOffset = params.phase ?? 0;
    const pulseWidth = params.pulseWidth ?? 0.5;

    // Handle reset trigger (rising edge detection)
    if (reset > 0.5 && this.lastReset <= 0.5) {
      this.phase = phaseOffset;
    }
    this.lastReset = reset;

    // Calculate modulated frequency
    const modulatedFreq = frequency * Math.pow(2, fm * fmAmount * 4);

    // Advance phase
    this.phase += modulatedFreq / this.sampleRate;
    this.phase = this.phase % 1;

    // Calculate waveforms from current phase
    const p = (this.phase + phaseOffset) % 1;

    // Sine wave
    const sine = Math.sin(p * Math.PI * 2);

    // Triangle wave (0-1 range, then scaled to -1 to 1)
    const triangle = p < 0.5
      ? p * 4 - 1
      : 3 - p * 4;

    // Sawtooth (rising ramp, -1 to 1)
    const sawtooth = p * 2 - 1;

    // Square wave with pulse width
    const square = p < pulseWidth ? 1 : -1;

    // Convert from bipolar (-1 to 1) to unipolar (0 to 1) for video standard
    return {
      sine: (sine + 1) / 2,
      triangle: (triangle + 1) / 2,
      sawtooth: (sawtooth + 1) / 2,
      square: (square + 1) / 2,
    };
  }

  reset(): void {
    this.phase = 0;
    this.lastReset = 0;
  }

  /**
   * Get current phase (0-1)
   */
  getPhase(): number {
    return this.phase;
  }

  /**
   * Set sample rate
   */
  setSampleRate(rate: number): void {
    this.sampleRate = rate;
  }
}

/**
 * Video Rate Oscillator
 *
 * Generates patterns synchronized to video timing.
 * Creates horizontal/vertical bars, grids, and geometric patterns.
 *
 * Inspired by LZX Cadet and Expedition series modules.
 */
export class VideoOscillator implements Module {
  readonly type = 'video-osc';
  readonly name = 'Video Oscillator';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'hSync', type: 'signal', defaultValue: 0, description: 'Horizontal sync input' },
    { name: 'vSync', type: 'signal', defaultValue: 0, description: 'Vertical sync input' },
    { name: 'fm', type: 'cv', defaultValue: 0, description: 'Frequency modulation' },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'out', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'frequency', defaultValue: 1, min: 0.1, max: 100, description: 'Cycles per screen' },
    { name: 'fmAmount', defaultValue: 0, min: -1, max: 1 },
    { name: 'phase', defaultValue: 0, min: 0, max: 1 },
    { name: 'waveform', defaultValue: 0, min: 0, max: 3, step: 1, description: '0=sine, 1=tri, 2=saw, 3=square' },
    { name: 'orientation', defaultValue: 0, min: 0, max: 1, step: 1, description: '0=horizontal, 1=vertical' },
  ];

  private lastHSync: Signal = 0;
  private lastVSync: Signal = 0;
  private hPhase: number = 0;
  private vPhase: number = 0;

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const hSync = inputs.hSync ?? 0;
    const vSync = inputs.vSync ?? 0;
    const fm = inputs.fm ?? 0;

    const frequency = params.frequency ?? 1;
    const fmAmount = params.fmAmount ?? 0;
    const phaseOffset = params.phase ?? 0;
    const waveform = Math.round(params.waveform ?? 0);
    const orientation = Math.round(params.orientation ?? 0);

    // Reset phase on sync pulses
    if (hSync > 0.5 && this.lastHSync <= 0.5) {
      this.hPhase = 0;
    }
    if (vSync > 0.5 && this.lastVSync <= 0.5) {
      this.vPhase = 0;
    }
    this.lastHSync = hSync;
    this.lastVSync = vSync;

    // Calculate modulated frequency
    const modulatedFreq = frequency * Math.pow(2, fm * fmAmount);

    // Use horizontal or vertical phase based on orientation
    const phase = orientation === 0 ? this.hPhase : this.vPhase;
    const p = (phase * modulatedFreq + phaseOffset) % 1;

    // Generate waveform
    let value: Signal;
    switch (waveform) {
      case 0: // Sine
        value = (Math.sin(p * Math.PI * 2) + 1) / 2;
        break;
      case 1: // Triangle
        value = p < 0.5 ? p * 2 : 2 - p * 2;
        break;
      case 2: // Sawtooth
        value = p;
        break;
      case 3: // Square
        value = p < 0.5 ? 1 : 0;
        break;
      default:
        value = 0;
    }

    return { out: value };
  }

  /**
   * Update phase based on position (for frame-based processing)
   */
  updatePosition(u: number, v: number): void {
    this.hPhase = u;
    this.vPhase = v;
  }

  reset(): void {
    this.lastHSync = 0;
    this.lastVSync = 0;
    this.hPhase = 0;
    this.vPhase = 0;
  }
}

/**
 * Ramp Generator
 *
 * Generates synchronized ramp signals based on Beck's Direct Video Synthesizer
 * voltage-to-position converter concept. Essential for creating geometric shapes.
 *
 * Outputs both edge-referenced (ramp) and center-referenced (triangle) signals.
 */
export class RampGenerator implements Module {
  readonly type = 'ramp-gen';
  readonly name = 'Ramp Generator';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'hRamp', type: 'signal', defaultValue: 0, description: 'Horizontal position ramp from sync gen' },
    { name: 'vRamp', type: 'signal', defaultValue: 0, description: 'Vertical position ramp from sync gen' },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'hEdge', type: 'signal', defaultValue: 0, description: 'Horizontal edge ramp (0-1 left to right)' },
    { name: 'hCenter', type: 'signal', defaultValue: 0, description: 'Horizontal center triangle (0 at edges, 1 at center)' },
    { name: 'vEdge', type: 'signal', defaultValue: 0, description: 'Vertical edge ramp (0-1 top to bottom)' },
    { name: 'vCenter', type: 'signal', defaultValue: 0, description: 'Vertical center triangle (0 at edges, 1 at center)' },
    { name: 'radial', type: 'signal', defaultValue: 0, description: 'Distance from center (0 at center, 1 at corners)' },
    { name: 'angle', type: 'signal', defaultValue: 0, description: 'Angle from center (0-1 representing 0-360°)' },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'hOffset', defaultValue: 0.5, min: 0, max: 1, description: 'Horizontal center offset' },
    { name: 'vOffset', defaultValue: 0.5, min: 0, max: 1, description: 'Vertical center offset' },
    { name: 'hScale', defaultValue: 1, min: 0.1, max: 10, description: 'Horizontal scale' },
    { name: 'vScale', defaultValue: 1, min: 0.1, max: 10, description: 'Vertical scale' },
    { name: 'rotation', defaultValue: 0, min: 0, max: 1, description: 'Coordinate rotation (0-1 = 0-360°)' },
  ];

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const hRamp = inputs.hRamp ?? 0;
    const vRamp = inputs.vRamp ?? 0;

    const hOffset = params.hOffset ?? 0.5;
    const vOffset = params.vOffset ?? 0.5;
    const hScale = params.hScale ?? 1;
    const vScale = params.vScale ?? 1;
    const rotation = params.rotation ?? 0;

    // Calculate centered coordinates
    let x = (hRamp - hOffset) * hScale;
    let y = (vRamp - vOffset) * vScale;

    // Apply rotation if specified
    if (rotation !== 0) {
      const angle = rotation * Math.PI * 2;
      const cos = Math.cos(angle);
      const sin = Math.sin(angle);
      const xr = x * cos - y * sin;
      const yr = x * sin + y * cos;
      x = xr;
      y = yr;
    }

    // Edge ramps (0-1)
    const hEdge = clamp(hRamp);
    const vEdge = clamp(vRamp);

    // Center-referenced triangles (0 at edges, 1 at center)
    const hCenter = 1 - Math.abs(x) * 2;
    const vCenter = 1 - Math.abs(y) * 2;

    // Radial distance from center (0 at center, 1 at corners)
    const radial = Math.min(1, Math.sqrt(x * x + y * y) * Math.SQRT2);

    // Angle from center (0-1 representing 0-360°)
    const angleRad = Math.atan2(y, x);
    const angleSig = (angleRad / (Math.PI * 2) + 0.5) % 1;

    return {
      hEdge,
      hCenter: clamp(hCenter),
      vEdge,
      vCenter: clamp(vCenter),
      radial,
      angle: angleSig,
    };
  }

  reset(): void {
    // Stateless module
  }
}

/**
 * Shape Generator
 *
 * Creates basic geometric shapes using comparators on ramp signals.
 * Based on Beck's Geometric Region Processor concept.
 */
export class ShapeGenerator implements Module {
  readonly type = 'shape-gen';
  readonly name = 'Shape Generator';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'hRamp', type: 'signal', defaultValue: 0 },
    { name: 'vRamp', type: 'signal', defaultValue: 0 },
    { name: 'modX', type: 'cv', defaultValue: 0, description: 'X position modulation' },
    { name: 'modY', type: 'cv', defaultValue: 0, description: 'Y position modulation' },
    { name: 'modSize', type: 'cv', defaultValue: 0, description: 'Size modulation' },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'rect', type: 'signal', defaultValue: 0, description: 'Rectangle shape' },
    { name: 'circle', type: 'signal', defaultValue: 0, description: 'Circle/ellipse shape' },
    { name: 'diamond', type: 'signal', defaultValue: 0, description: 'Diamond shape' },
    { name: 'hBar', type: 'signal', defaultValue: 0, description: 'Horizontal bar' },
    { name: 'vBar', type: 'signal', defaultValue: 0, description: 'Vertical bar' },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'x', defaultValue: 0.5, min: 0, max: 1, description: 'X position' },
    { name: 'y', defaultValue: 0.5, min: 0, max: 1, description: 'Y position' },
    { name: 'width', defaultValue: 0.3, min: 0, max: 1, description: 'Width' },
    { name: 'height', defaultValue: 0.3, min: 0, max: 1, description: 'Height' },
    { name: 'softness', defaultValue: 0, min: 0, max: 0.2, description: 'Edge softness' },
  ];

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const hRamp = inputs.hRamp ?? 0;
    const vRamp = inputs.vRamp ?? 0;
    const modX = inputs.modX ?? 0;
    const modY = inputs.modY ?? 0;
    const modSize = inputs.modSize ?? 0;

    const x = (params.x ?? 0.5) + modX * 0.5;
    const y = (params.y ?? 0.5) + modY * 0.5;
    const width = Math.max(0.001, (params.width ?? 0.3) + modSize * 0.3);
    const height = Math.max(0.001, (params.height ?? 0.3) + modSize * 0.3);
    const softness = Math.max(0.001, params.softness ?? 0.001);

    // Calculate distances
    const dx = hRamp - x;
    const dy = vRamp - y;
    const absDx = Math.abs(dx);
    const absDy = Math.abs(dy);

    // Rectangle: smoothstep on both axes
    const rectX = 1 - smoothstep(width / 2 - softness, width / 2 + softness, absDx);
    const rectY = 1 - smoothstep(height / 2 - softness, height / 2 + softness, absDy);
    const rect = rectX * rectY;

    // Circle: distance from center
    const dist = Math.sqrt((dx * dx) / (width * width) + (dy * dy) / (height * height));
    const circle = 1 - smoothstep(0.5 - softness, 0.5 + softness, dist);

    // Diamond: Manhattan distance
    const manhattanDist = absDx / width + absDy / height;
    const diamond = 1 - smoothstep(0.5 - softness, 0.5 + softness, manhattanDist);

    // Horizontal bar
    const hBar = 1 - smoothstep(height / 2 - softness, height / 2 + softness, absDy);

    // Vertical bar
    const vBar = 1 - smoothstep(width / 2 - softness, width / 2 + softness, absDx);

    return {
      rect: clamp(rect),
      circle: clamp(circle),
      diamond: clamp(diamond),
      hBar: clamp(hBar),
      vBar: clamp(vBar),
    };
  }

  reset(): void {
    // Stateless module
  }
}

/**
 * Noise Generator
 *
 * Generates various types of noise for analog character.
 * Includes white noise, pink (1/f) noise, and video-coherent noise.
 */
export class NoiseGenerator implements Module {
  readonly type = 'noise-gen';
  readonly name = 'Noise Generator';

  readonly inputs: ReadonlyArray<Port> = [];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'white', type: 'signal', defaultValue: 0, description: 'White noise (uniform spectrum)' },
    { name: 'pink', type: 'signal', defaultValue: 0, description: 'Pink noise (1/f spectrum)' },
    { name: 'brown', type: 'signal', defaultValue: 0, description: 'Brown noise (1/f² spectrum)' },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'level', defaultValue: 0.1, min: 0, max: 1, description: 'Output level' },
    { name: 'seed', defaultValue: 0, min: 0, max: 1000000, step: 1, description: 'Random seed' },
  ];

  // Pink noise filter state (Voss-McCartney algorithm approximation)
  private pinkState: number[] = [0, 0, 0, 0, 0, 0, 0];
  private pinkKey: number = 0;

  // Brown noise state
  private brownState: number = 0;

  // Simple PRNG state
  private randomState: number = 1;

  constructor(seed: number = 1) {
    this.randomState = seed || 1;
  }

  private random(): number {
    // xorshift32 PRNG
    let x = this.randomState;
    x ^= x << 13;
    x ^= x >>> 17;
    x ^= x << 5;
    this.randomState = x >>> 0;
    return (x >>> 0) / 0xFFFFFFFF;
  }

  private gaussianRandom(): number {
    // Box-Muller transform for Gaussian distribution
    const u1 = this.random();
    const u2 = this.random();
    return Math.sqrt(-2 * Math.log(u1 || 0.0001)) * Math.cos(2 * Math.PI * u2);
  }

  process(
    _inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const level = params.level ?? 0.1;

    // White noise (Gaussian)
    const white = this.gaussianRandom() * level;

    // Pink noise using Paul Kellet's refined method
    const whiteForPink = this.random() * 2 - 1;
    this.pinkState[0] = 0.99886 * this.pinkState[0] + whiteForPink * 0.0555179;
    this.pinkState[1] = 0.99332 * this.pinkState[1] + whiteForPink * 0.0750759;
    this.pinkState[2] = 0.96900 * this.pinkState[2] + whiteForPink * 0.1538520;
    this.pinkState[3] = 0.86650 * this.pinkState[3] + whiteForPink * 0.3104856;
    this.pinkState[4] = 0.55000 * this.pinkState[4] + whiteForPink * 0.5329522;
    this.pinkState[5] = -0.7616 * this.pinkState[5] - whiteForPink * 0.0168980;
    const pink = (
      this.pinkState[0] + this.pinkState[1] + this.pinkState[2] +
      this.pinkState[3] + this.pinkState[4] + this.pinkState[5] +
      this.pinkState[6] + whiteForPink * 0.5362
    ) * 0.11 * level;
    this.pinkState[6] = whiteForPink * 0.115926;

    // Brown noise (integrated white noise)
    const whiteForBrown = this.random() * 2 - 1;
    this.brownState += whiteForBrown * 0.02;
    this.brownState *= 0.999; // Slight decay to prevent drift
    const brown = clamp(this.brownState * level + 0.5, 0, 1);

    return {
      white: clamp(white + 0.5, 0, 1),
      pink: clamp(pink + 0.5, 0, 1),
      brown,
    };
  }

  reset(): void {
    this.pinkState = [0, 0, 0, 0, 0, 0, 0];
    this.pinkKey = 0;
    this.brownState = 0;
  }

  /**
   * Set random seed
   */
  setSeed(seed: number): void {
    this.randomState = seed || 1;
  }
}

/**
 * Smoothstep function for soft edges
 */
function smoothstep(edge0: number, edge1: number, x: number): number {
  const t = clamp((x - edge0) / (edge1 - edge0), 0, 1);
  return t * t * (3 - 2 * t);
}

/**
 * Generate waveform sample at given phase
 */
export function generateWaveform(type: WaveformType, phase: number, pulseWidth: number = 0.5): Signal {
  const p = phase % 1;

  switch (type) {
    case 'sine':
      return (Math.sin(p * Math.PI * 2) + 1) / 2;

    case 'triangle':
      return p < 0.5 ? p * 2 : 2 - p * 2;

    case 'sawtooth':
      return p;

    case 'ramp':
      return 1 - p;

    case 'square':
      return p < pulseWidth ? 1 : 0;

    case 'pulse':
      return p < pulseWidth ? 1 : 0;

    default:
      return 0;
  }
}
