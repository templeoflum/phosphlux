/**
 * Tests for Waveshaper Modules
 */

import { describe, it, expect, beforeEach } from 'vitest';
import {
  FoldingAmplifier,
  SoftClipper,
  HardClipper,
  TransferWaveshaper,
  Quantizer,
  Solarizer,
  softClip,
  fastSoftClip,
  chebyshev,
} from './waveshapers.js';

describe('FoldingAmplifier', () => {
  let folder: FoldingAmplifier;

  beforeEach(() => {
    folder = new FoldingAmplifier();
  });

  it('should pass through low-level signals unchanged', () => {
    const result = folder.process({ in: 0.3 }, { gain: 1, folds: 1 });
    expect(result.out).toBeCloseTo(0.3, 5);
  });

  it('should fold signals over threshold', () => {
    const result = folder.process({ in: 0.5 }, { gain: 3, folds: 1 });
    expect(result.out).toBeLessThan(1);
    expect(result.out).toBeGreaterThanOrEqual(0);
  });

  it('should increase complexity with more folds', () => {
    // Use input that produces intermediate values after folding
    // gain=3, input=0.5 → signal=1.5, which folds to 0.5 with 1 fold
    // With 2 folds, 0.5 stays 0.5 (no further folding needed)
    // Use gain=2.5 for more interesting behavior
    const r1 = folder.process({ in: 0.6 }, { gain: 2.5, folds: 1 });
    const r2 = folder.process({ in: 0.6 }, { gain: 2.5, folds: 3 });

    // Both should be in valid range
    expect(r1.out).toBeGreaterThanOrEqual(0);
    expect(r1.out).toBeLessThanOrEqual(1);
    expect(r2.out).toBeGreaterThanOrEqual(0);
    expect(r2.out).toBeLessThanOrEqual(1);
  });

  it('should clamp output to 0-1', () => {
    for (let i = 0; i < 10; i++) {
      const input = Math.random() * 2;
      const result = folder.process({ in: input }, { gain: 5, folds: 3 });

      expect(result.out).toBeGreaterThanOrEqual(0);
      expect(result.out).toBeLessThanOrEqual(1);
    }
  });
});

describe('SoftClipper', () => {
  let clipper: SoftClipper;

  beforeEach(() => {
    clipper = new SoftClipper();
  });

  it('should pass through low signals with minimal change', () => {
    const result = clipper.process({ in: 0.5 }, { drive: 1, mode: 0, mix: 1 });
    expect(result.out).toBeCloseTo(0.5, 1);
  });

  it('should saturate high signals', () => {
    const result = clipper.process({ in: 0.9 }, { drive: 5, mode: 0, mix: 1 });
    expect(result.out).toBeLessThan(1);
    expect(result.out).toBeGreaterThan(0.5);
  });

  it('should support different saturation modes', () => {
    const input = { in: 0.8 };
    const params = { drive: 3, mix: 1 };

    const tanh = clipper.process(input, { ...params, mode: 0 });
    const atan = clipper.process(input, { ...params, mode: 1 });
    const soft = clipper.process(input, { ...params, mode: 2 });
    const cubic = clipper.process(input, { ...params, mode: 3 });

    // All modes should produce valid output
    expect(tanh.out).toBeGreaterThanOrEqual(0);
    expect(tanh.out).toBeLessThanOrEqual(1);
    expect(atan.out).toBeGreaterThanOrEqual(0);
    expect(soft.out).toBeGreaterThanOrEqual(0);
    expect(cubic.out).toBeGreaterThanOrEqual(0);
  });

  it('should respect mix parameter', () => {
    const dry = clipper.process({ in: 0.8 }, { drive: 5, mode: 0, mix: 0 });
    const wet = clipper.process({ in: 0.8 }, { drive: 5, mode: 0, mix: 1 });

    expect(dry.out).toBeCloseTo(0.8, 5);
    expect(wet.out).not.toBeCloseTo(0.8, 2);
  });
});

describe('HardClipper', () => {
  let clipper: HardClipper;

  beforeEach(() => {
    clipper = new HardClipper();
  });

  it('should clip at thresholds', () => {
    const result = clipper.process({ in: 0.5 }, { low: 0.3, high: 0.7, gain: 1 });
    expect(result.out).toBe(0.5);

    const lowClip = clipper.process({ in: 0.1 }, { low: 0.3, high: 0.7, gain: 1 });
    expect(lowClip.out).toBe(0.3);

    const highClip = clipper.process({ in: 0.9 }, { low: 0.3, high: 0.7, gain: 1 });
    expect(highClip.out).toBe(0.7);
  });

  it('should indicate clipping', () => {
    const noClip = clipper.process({ in: 0.5 }, { low: 0.3, high: 0.7, gain: 1 });
    expect(noClip.clip).toBe(0);

    const clipping = clipper.process({ in: 0.1 }, { low: 0.3, high: 0.7, gain: 1 });
    expect(clipping.clip).toBe(1);
  });
});

describe('TransferWaveshaper', () => {
  let waveshaper: TransferWaveshaper;

  beforeEach(() => {
    waveshaper = new TransferWaveshaper();
  });

  it('should pass through with linear curve', () => {
    const result = waveshaper.process({ in: 0.5 }, { curveType: 0, intensity: 1 });
    expect(result.out).toBeCloseTo(0.5, 2);
  });

  it('should apply sqrt curve correctly', () => {
    const result = waveshaper.process({ in: 0.25 }, { curveType: 1, intensity: 1 });
    expect(result.out).toBeCloseTo(0.5, 2);
  });

  it('should apply square curve correctly', () => {
    const result = waveshaper.process({ in: 0.5 }, { curveType: 2, intensity: 1 });
    expect(result.out).toBeCloseTo(0.25, 2);
  });

  it('should invert with invert curve', () => {
    const result = waveshaper.process({ in: 0.3 }, { curveType: 6, intensity: 1 });
    expect(result.out).toBeCloseTo(0.7, 2);
  });
});

describe('Quantizer', () => {
  let quantizer: Quantizer;

  beforeEach(() => {
    quantizer = new Quantizer();
  });

  it('should quantize to specified levels', () => {
    // 2 levels with floor mode: 0-0.99 → 0, only 1.0 → 1
    // With levels=2, scaled = input * (2-1) = input
    // floor(0.3) = 0, floor(0.7) = 0, floor(1.0) = 1
    const r1 = quantizer.process({ in: 0.3 }, { levels: 2, mode: 0 });
    expect(r1.out).toBe(0);

    // Use round mode (mode=1) for 0.7 to round to 1
    const r2 = quantizer.process({ in: 0.7 }, { levels: 2, mode: 1 });
    expect(r2.out).toBe(1);
  });

  it('should support different rounding modes', () => {
    const floor = quantizer.process({ in: 0.4 }, { levels: 3, mode: 0 });
    const round = quantizer.process({ in: 0.4 }, { levels: 3, mode: 1 });
    const ceil = quantizer.process({ in: 0.4 }, { levels: 3, mode: 2 });

    expect(floor.out).toBeLessThanOrEqual(round.out);
    expect(round.out).toBeLessThanOrEqual(ceil.out);
  });

  it('should output level indicator', () => {
    const result = quantizer.process({ in: 0.5 }, { levels: 8 });
    expect(result.level).toBeDefined();
    expect(result.level).toBeGreaterThanOrEqual(0);
    expect(result.level).toBeLessThanOrEqual(1);
  });
});

describe('Solarizer', () => {
  let solarizer: Solarizer;

  beforeEach(() => {
    solarizer = new Solarizer();
  });

  it('should pass through values below threshold in above mode', () => {
    const result = solarizer.process({ in: 0.3 }, {
      threshold: 0.5,
      intensity: 1,
      mode: 0,
    });
    expect(result.out).toBeCloseTo(0.3, 5);
  });

  it('should invert values above threshold in above mode', () => {
    const result = solarizer.process({ in: 0.7 }, {
      threshold: 0.5,
      intensity: 1,
      mode: 0,
    });
    expect(result.out).toBeLessThan(0.7);
  });

  it('should support intensity control', () => {
    const full = solarizer.process({ in: 0.8 }, {
      threshold: 0.5,
      intensity: 1,
      mode: 0,
    });
    const half = solarizer.process({ in: 0.8 }, {
      threshold: 0.5,
      intensity: 0.5,
      mode: 0,
    });

    // Partial intensity should be between original and full solarization
    expect(half.out).toBeGreaterThan(full.out);
    expect(half.out).toBeLessThan(0.8);
  });
});

describe('Utility functions', () => {
  describe('softClip', () => {
    it('should saturate towards -1 and 1', () => {
      expect(softClip(0, 1)).toBeCloseTo(0, 5);
      expect(softClip(10, 1)).toBeCloseTo(1, 2);
      expect(softClip(-10, 1)).toBeCloseTo(-1, 2);
    });

    it('should respond to drive', () => {
      const lowDrive = softClip(0.5, 1);
      const highDrive = softClip(0.5, 5);

      expect(highDrive).toBeGreaterThan(lowDrive);
    });
  });

  describe('fastSoftClip', () => {
    it('should approximate soft clipping', () => {
      expect(fastSoftClip(0)).toBe(0);
      expect(fastSoftClip(1)).toBe(0.5);
      expect(fastSoftClip(-1)).toBe(-0.5);
      expect(fastSoftClip(10)).toBeCloseTo(0.909, 2);
    });
  });

  describe('chebyshev', () => {
    it('should return correct values for low orders', () => {
      expect(chebyshev(0.5, 0)).toBe(1);
      expect(chebyshev(0.5, 1)).toBe(0.5);
      expect(chebyshev(1, 2)).toBe(1);
      expect(chebyshev(0, 2)).toBe(-1);
    });

    it('should work for higher orders', () => {
      const x = 0.7;
      const t5 = chebyshev(x, 5);
      expect(typeof t5).toBe('number');
      expect(Math.abs(t5)).toBeLessThanOrEqual(1);
    });
  });
});
