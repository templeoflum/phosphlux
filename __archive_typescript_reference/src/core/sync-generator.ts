/**
 * Sync Generator - The heartbeat of the video synthesizer
 *
 * Based on LZX Industries design philosophy where all timing derives from
 * a master sync generator. Generates horizontal and vertical sync pulses,
 * blanking intervals, and position ramps.
 *
 * This is the foundation - every other module locks to these signals.
 */

import {
  type Signal,
  type SyncSignals,
  type TimingConfig,
  type VideoStandard,
  NTSC_TIMING,
  PAL_TIMING,
} from './types.js';

/**
 * Configuration for the sync generator
 */
export interface SyncGeneratorConfig {
  /** Video standard (NTSC or PAL) */
  standard: VideoStandard;
  /** Sample rate in Hz */
  sampleRate: number;
  /** Enable interlaced output */
  interlaced?: boolean;
  /** Custom timing override */
  customTiming?: Partial<TimingConfig>;
}

/**
 * Master sync generator
 *
 * Generates all timing signals needed for video synthesis:
 * - Composite sync (H + V combined)
 * - Horizontal sync pulses
 * - Vertical sync pulses
 * - Blanking intervals
 * - Position ramps (horizontal and vertical)
 */
export class SyncGenerator {
  private config: SyncGeneratorConfig;
  private timing: TimingConfig;

  // Internal counters
  private sampleCounter: number = 0;
  private lineCounter: number = 0;
  private fieldCounter: number = 0;

  // Derived timing values (in samples)
  private samplesPerLine: number;
  private samplesPerField: number;
  private hSyncSamples: number;
  private hBlankStartSamples: number;
  private hBlankEndSamples: number;
  private activeStartSample: number;
  private activeEndSample: number;
  private vBlankStartLine: number;
  private vBlankEndLine: number;

  constructor(config: SyncGeneratorConfig) {
    this.config = config;
    this.timing = this.buildTiming();
    this.calculateDerivedValues();
  }

  private buildTiming(): TimingConfig {
    const baseTiming = this.config.standard === 'NTSC' ? NTSC_TIMING : PAL_TIMING;
    return { ...baseTiming, ...this.config.customTiming };
  }

  private calculateDerivedValues(): void {
    const { sampleRate } = this.config;
    const t = this.timing;

    // Calculate samples per line from horizontal rate
    this.samplesPerLine = Math.round(sampleRate / t.hRate);

    // Samples per field
    const linesPerField = this.config.interlaced
      ? t.totalLines / 2
      : t.totalLines;
    this.samplesPerField = this.samplesPerLine * linesPerField;

    // Horizontal timing in samples
    const usPerSample = 1_000_000 / sampleRate;
    this.hSyncSamples = Math.round(t.hSyncWidth / usPerSample);

    // Blanking intervals
    const lineTimeUs = 1_000_000 / t.hRate;
    const activeTimeUs = lineTimeUs - t.frontPorch - t.hSyncWidth - t.backPorch;
    const activeSamples = Math.round(activeTimeUs / usPerSample);

    this.hBlankStartSamples = Math.round((lineTimeUs - t.frontPorch) / usPerSample);
    this.hBlankEndSamples = Math.round((t.hSyncWidth + t.backPorch) / usPerSample);
    this.activeStartSample = this.hBlankEndSamples;
    this.activeEndSample = this.activeStartSample + activeSamples;

    // Vertical blanking (lines)
    this.vBlankStartLine = t.activeLines;
    this.vBlankEndLine = t.totalLines;
  }

  /**
   * Get current timing configuration
   */
  getTiming(): TimingConfig {
    return { ...this.timing };
  }

  /**
   * Get samples per line for current configuration
   */
  getSamplesPerLine(): number {
    return this.samplesPerLine;
  }

  /**
   * Get samples per field for current configuration
   */
  getSamplesPerField(): number {
    return this.samplesPerField;
  }

  /**
   * Get active video dimensions
   */
  getActiveDimensions(): { width: number; height: number } {
    return {
      width: this.activeEndSample - this.activeStartSample,
      height: this.timing.activeLines,
    };
  }

  /**
   * Reset all counters to start of frame
   */
  reset(): void {
    this.sampleCounter = 0;
    this.lineCounter = 0;
    this.fieldCounter = 0;
  }

  /**
   * Advance by one sample and return current sync signals
   */
  tick(): SyncSignals {
    const signals = this.getCurrentSignals();

    // Advance sample counter
    this.sampleCounter++;

    // Check for end of line
    if (this.sampleCounter >= this.samplesPerLine) {
      this.sampleCounter = 0;
      this.lineCounter++;

      // Check for end of field
      const linesPerField = this.config.interlaced
        ? Math.ceil(this.timing.totalLines / 2)
        : this.timing.totalLines;

      if (this.lineCounter >= linesPerField) {
        this.lineCounter = 0;
        this.fieldCounter = (this.fieldCounter + 1) % 2;
      }
    }

    return signals;
  }

  /**
   * Get sync signals for current position without advancing
   */
  getCurrentSignals(): SyncSignals {
    const sample = this.sampleCounter;
    const line = this.lineCounter;

    // Horizontal sync: active during sync pulse at start of line
    const hSync: Signal = sample < this.hSyncSamples ? 1 : 0;

    // Vertical sync: active during vertical sync lines
    const effectiveLine = this.config.interlaced && this.fieldCounter === 1
      ? line + 0.5
      : line;
    const vSync: Signal = effectiveLine >= this.timing.activeLines &&
      effectiveLine < this.timing.activeLines + this.timing.vSyncLines ? 1 : 0;

    // Composite sync (XOR of H and V during vertical, otherwise just H)
    const composite: Signal = vSync ? (hSync ? 0 : 1) : hSync;

    // Horizontal blanking
    const hBlank: Signal = sample < this.activeStartSample ||
      sample >= this.activeEndSample ? 1 : 0;

    // Vertical blanking
    const vBlank: Signal = line >= this.vBlankStartLine ? 1 : 0;

    // Combined blanking
    const blank: Signal = hBlank || vBlank ? 1 : 0;

    // Horizontal ramp (0-1 over active line)
    let hRamp: Signal = 0;
    if (sample >= this.activeStartSample && sample < this.activeEndSample) {
      hRamp = (sample - this.activeStartSample) /
        (this.activeEndSample - this.activeStartSample);
    }

    // Vertical ramp (0-1 over active frame)
    let vRamp: Signal = 0;
    if (line < this.timing.activeLines) {
      vRamp = line / this.timing.activeLines;

      // Add half-line offset for interlaced odd field
      if (this.config.interlaced && this.fieldCounter === 1) {
        vRamp += 0.5 / this.timing.activeLines;
      }
    }

    return {
      composite,
      hSync,
      vSync,
      hBlank,
      vBlank,
      blank,
      hRamp,
      vRamp,
      line,
      sample,
      field: this.fieldCounter,
    };
  }

  /**
   * Generate a complete frame of sync signals
   * Useful for offline processing
   */
  generateFrame(): SyncSignals[] {
    this.reset();
    const frame: SyncSignals[] = [];

    for (let i = 0; i < this.samplesPerField; i++) {
      frame.push(this.tick());
    }

    return frame;
  }

  /**
   * Set position to specific line and sample
   */
  setPosition(line: number, sample: number, field: number = 0): void {
    this.lineCounter = line % this.timing.totalLines;
    this.sampleCounter = sample % this.samplesPerLine;
    this.fieldCounter = field % 2;
  }

  /**
   * Check if current position is in active video region
   */
  isActive(): boolean {
    const signals = this.getCurrentSignals();
    return signals.blank === 0;
  }

  /**
   * Get normalized coordinates (0-1) for current position
   * Returns null if in blanking
   */
  getNormalizedPosition(): { x: number; y: number } | null {
    if (!this.isActive()) {
      return null;
    }

    const signals = this.getCurrentSignals();
    return {
      x: signals.hRamp,
      y: signals.vRamp,
    };
  }
}

/**
 * Create a sync generator with sensible defaults for NTSC
 */
export function createNTSCSyncGenerator(sampleRate: number = 13500000): SyncGenerator {
  return new SyncGenerator({
    standard: 'NTSC',
    sampleRate,
    interlaced: false,
  });
}

/**
 * Create a sync generator with sensible defaults for PAL
 */
export function createPALSyncGenerator(sampleRate: number = 13500000): SyncGenerator {
  return new SyncGenerator({
    standard: 'PAL',
    sampleRate,
    interlaced: false,
  });
}

/**
 * Simplified sync generator for frame-based processing
 * Instead of sample-accurate timing, generates per-pixel coordinates
 */
export class FrameSyncGenerator {
  readonly width: number;
  readonly height: number;

  private x: number = 0;
  private y: number = 0;

  constructor(width: number = 720, height: number = 480) {
    this.width = width;
    this.height = height;
  }

  /**
   * Reset to top-left corner
   */
  reset(): void {
    this.x = 0;
    this.y = 0;
  }

  /**
   * Get current position
   */
  getPosition(): { x: number; y: number; u: number; v: number } {
    return {
      x: this.x,
      y: this.y,
      u: this.x / this.width,
      v: this.y / this.height,
    };
  }

  /**
   * Advance to next pixel
   * Returns true if still in frame, false at end of frame
   */
  next(): boolean {
    this.x++;
    if (this.x >= this.width) {
      this.x = 0;
      this.y++;
      if (this.y >= this.height) {
        this.y = 0;
        return false;
      }
    }
    return true;
  }

  /**
   * Set position directly
   */
  setPosition(x: number, y: number): void {
    this.x = x % this.width;
    this.y = y % this.height;
  }

  /**
   * Generate horizontal ramp (0-1 across width)
   */
  getHRamp(): Signal {
    return this.x / this.width;
  }

  /**
   * Generate vertical ramp (0-1 down height)
   */
  getVRamp(): Signal {
    return this.y / this.height;
  }

  /**
   * Generate center-referenced horizontal signal (-1 to +1)
   */
  getHCenter(): Signal {
    return (this.x / this.width) * 2 - 1;
  }

  /**
   * Generate center-referenced vertical signal (-1 to +1)
   */
  getVCenter(): Signal {
    return (this.y / this.height) * 2 - 1;
  }

  /**
   * Iterate over all pixels with callback
   */
  forEach(callback: (x: number, y: number, u: number, v: number) => void): void {
    for (let y = 0; y < this.height; y++) {
      for (let x = 0; x < this.width; x++) {
        callback(x, y, x / this.width, y / this.height);
      }
    }
  }
}
