/**
 * Core types for analog video synthesis emulation
 *
 * Signal standards based on LZX Industries conventions:
 * - Video signals: 0-1V range, DC coupled
 * - Control voltages: 0-1V (unipolar) or -1V to +1V (bipolar)
 * - All timing derived from master sync generator
 */

/** Sample rate for control-rate processing (audio rate) */
export const CONTROL_RATE = 48000;

/** Horizontal line rate for NTSC (15.734kHz) */
export const NTSC_H_RATE = 15734.264;

/** Vertical field rate for NTSC (59.94Hz) */
export const NTSC_V_RATE = 59.94;

/** NTSC color subcarrier frequency */
export const NTSC_SUBCARRIER = 3579545;

/** Horizontal line rate for PAL (15.625kHz) */
export const PAL_H_RATE = 15625;

/** Vertical field rate for PAL (50Hz) */
export const PAL_V_RATE = 50;

/** PAL color subcarrier frequency */
export const PAL_SUBCARRIER = 4433618.75;

/**
 * Video timing standard
 */
export type VideoStandard = 'NTSC' | 'PAL';

/**
 * Timing configuration for a video standard
 */
export interface TimingConfig {
  /** Horizontal line rate in Hz */
  hRate: number;
  /** Vertical field rate in Hz */
  vRate: number;
  /** Total lines per frame (including blanking) */
  totalLines: number;
  /** Active (visible) lines per frame */
  activeLines: number;
  /** Active samples per line (horizontal resolution) */
  activeSamplesPerLine: number;
  /** Color subcarrier frequency in Hz */
  subcarrierFreq: number;
  /** Horizontal sync pulse width in microseconds */
  hSyncWidth: number;
  /** Vertical sync pulse width in lines */
  vSyncLines: number;
  /** Front porch duration in microseconds */
  frontPorch: number;
  /** Back porch duration in microseconds */
  backPorch: number;
}

/**
 * Standard NTSC timing configuration
 */
export const NTSC_TIMING: TimingConfig = {
  hRate: NTSC_H_RATE,
  vRate: NTSC_V_RATE,
  totalLines: 525,
  activeLines: 480,
  activeSamplesPerLine: 720,
  subcarrierFreq: NTSC_SUBCARRIER,
  hSyncWidth: 4.7,      // microseconds
  vSyncLines: 3,
  frontPorch: 1.5,      // microseconds
  backPorch: 4.7,       // microseconds (includes color burst)
};

/**
 * Standard PAL timing configuration
 */
export const PAL_TIMING: TimingConfig = {
  hRate: PAL_H_RATE,
  vRate: PAL_V_RATE,
  totalLines: 625,
  activeLines: 576,
  activeSamplesPerLine: 720,
  subcarrierFreq: PAL_SUBCARRIER,
  hSyncWidth: 4.7,
  vSyncLines: 2.5,
  frontPorch: 1.65,
  backPorch: 5.7,
};

/**
 * A single video signal sample (0-1V range by convention)
 * Values outside 0-1 are allowed for overdrive/clipping effects
 */
export type Signal = number;

/**
 * RGB color triplet, each channel 0-1V
 */
export interface RGB {
  r: Signal;
  g: Signal;
  b: Signal;
}

/**
 * YIQ color space (NTSC native)
 * Y: luminance (0-1)
 * I: in-phase chrominance (-0.5957 to +0.5957)
 * Q: quadrature chrominance (-0.5226 to +0.5226)
 */
export interface YIQ {
  y: Signal;
  i: Signal;
  q: Signal;
}

/**
 * Frame buffer for video storage
 * Row-major order, RGB interleaved
 */
export interface FrameBuffer {
  width: number;
  height: number;
  data: Float32Array;
}

/**
 * Sync signals output from sync generator
 */
export interface SyncSignals {
  /** Composite sync (combined H and V) */
  composite: Signal;
  /** Horizontal sync pulse */
  hSync: Signal;
  /** Vertical sync pulse */
  vSync: Signal;
  /** Horizontal blanking interval */
  hBlank: Signal;
  /** Vertical blanking interval */
  vBlank: Signal;
  /** Combined blanking */
  blank: Signal;
  /** Horizontal ramp (0-1 over active line) */
  hRamp: Signal;
  /** Vertical ramp (0-1 over active frame) */
  vRamp: Signal;
  /** Current line number (0-based) */
  line: number;
  /** Current sample within line */
  sample: number;
  /** Current field (0 or 1 for interlaced) */
  field: number;
}

/**
 * Module input/output port types
 */
export type PortType = 'signal' | 'cv' | 'gate' | 'trigger';

/**
 * Module port descriptor
 */
export interface Port {
  name: string;
  type: PortType;
  /** Default value when unconnected */
  defaultValue: Signal;
  /** Minimum expected value */
  min?: number;
  /** Maximum expected value */
  max?: number;
  /** Human-readable description */
  description?: string;
}

/**
 * Parameter descriptor for module controls
 */
export interface Parameter {
  name: string;
  /** Default value */
  defaultValue: number;
  /** Minimum value */
  min: number;
  /** Maximum value */
  max: number;
  /** Step size for UI (0 for continuous) */
  step?: number;
  /** Unit label (e.g., "Hz", "V", "%") */
  unit?: string;
  /** Human-readable description */
  description?: string;
  /** Whether this parameter accepts CV modulation */
  cvModulatable?: boolean;
}

/**
 * Base interface for all synthesis modules
 */
export interface Module {
  /** Unique module type identifier */
  readonly type: string;
  /** Human-readable name */
  readonly name: string;
  /** Input port definitions */
  readonly inputs: ReadonlyArray<Port>;
  /** Output port definitions */
  readonly outputs: ReadonlyArray<Port>;
  /** Parameter definitions */
  readonly parameters: ReadonlyArray<Parameter>;

  /** Process one sample/pixel */
  process(inputs: Record<string, Signal>, params: Record<string, number>): Record<string, Signal>;

  /** Reset internal state */
  reset(): void;
}

/**
 * Connection between module ports
 */
export interface Connection {
  sourceModule: string;
  sourcePort: string;
  destModule: string;
  destPort: string;
  /** Optional attenuation/gain (-1 to +1 typical) */
  amount?: number;
}

/**
 * Complete patch state for save/recall
 */
export interface Patch {
  name: string;
  modules: Array<{
    id: string;
    type: string;
    params: Record<string, number>;
    position?: { x: number; y: number };
  }>;
  connections: Connection[];
}

/**
 * Clamp a signal to valid range
 */
export function clamp(value: Signal, min: number = 0, max: number = 1): Signal {
  return Math.max(min, Math.min(max, value));
}

/**
 * Linear interpolation
 */
export function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

/**
 * Map a value from one range to another
 */
export function mapRange(
  value: number,
  inMin: number,
  inMax: number,
  outMin: number,
  outMax: number
): number {
  return outMin + ((value - inMin) / (inMax - inMin)) * (outMax - outMin);
}

/**
 * Convert RGB to YIQ (NTSC color space)
 */
export function rgbToYiq(rgb: RGB): YIQ {
  return {
    y: 0.299 * rgb.r + 0.587 * rgb.g + 0.114 * rgb.b,
    i: 0.5959 * rgb.r - 0.2746 * rgb.g - 0.3213 * rgb.b,
    q: 0.2115 * rgb.r - 0.5227 * rgb.g + 0.3112 * rgb.b,
  };
}

/**
 * Convert YIQ to RGB
 */
export function yiqToRgb(yiq: YIQ): RGB {
  return {
    r: yiq.y + 0.956 * yiq.i + 0.619 * yiq.q,
    g: yiq.y - 0.272 * yiq.i - 0.647 * yiq.q,
    b: yiq.y - 1.106 * yiq.i + 1.703 * yiq.q,
  };
}

/**
 * Create an empty frame buffer
 */
export function createFrameBuffer(width: number, height: number): FrameBuffer {
  return {
    width,
    height,
    data: new Float32Array(width * height * 3),
  };
}

/**
 * Get pixel from frame buffer
 */
export function getPixel(fb: FrameBuffer, x: number, y: number): RGB {
  const idx = (y * fb.width + x) * 3;
  return {
    r: fb.data[idx],
    g: fb.data[idx + 1],
    b: fb.data[idx + 2],
  };
}

/**
 * Set pixel in frame buffer
 */
export function setPixel(fb: FrameBuffer, x: number, y: number, color: RGB): void {
  const idx = (y * fb.width + x) * 3;
  fb.data[idx] = color.r;
  fb.data[idx + 1] = color.g;
  fb.data[idx + 2] = color.b;
}
