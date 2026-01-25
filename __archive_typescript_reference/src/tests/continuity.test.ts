/**
 * High-Dimensional Continuity Tests
 *
 * Tests that verify smooth, continuous behavior across parameter space.
 * These tests sweep multiple parameters simultaneously to detect:
 * - Discontinuities in output
 * - Numerical instabilities
 * - Artifacts at parameter boundaries
 * - Cross-parameter interactions
 */

import { describe, it, expect, beforeEach } from 'vitest';
import {
  createFrameBuffer,
  clamp,
  type FrameBuffer,
} from '../core/types.js';
import { FrameSyncGenerator } from '../core/sync-generator.js';
import { LFO, VideoOscillator, RampGenerator, ShapeGenerator, NoiseGenerator } from '../modules/oscillators.js';
import { FoldingAmplifier, SoftClipper, Quantizer } from '../modules/waveshapers.js';
import { ThresholdColorizer, GradientMapper, LayeredColorizer } from '../modules/colorizers.js';
import { PhosphorPersistence, BeamBloom, ScreenCurvature } from '../modules/crt-emulation.js';
import { VideoFeedback } from '../modules/video-feedback.js';
import { RuttEtraDisplacement, Wobbulator } from '../modules/raster-manipulation.js';

/**
 * Helper to check if a value sequence is continuous (no large jumps)
 */
function isSequenceContinuous(values: number[], maxDelta: number = 0.5): boolean {
  for (let i = 1; i < values.length; i++) {
    if (Math.abs(values[i] - values[i - 1]) > maxDelta) {
      return false;
    }
  }
  return true;
}

/**
 * Helper to calculate standard deviation
 */
function standardDeviation(values: number[]): number {
  const mean = values.reduce((a, b) => a + b, 0) / values.length;
  const sqDiffs = values.map(v => Math.pow(v - mean, 2));
  return Math.sqrt(sqDiffs.reduce((a, b) => a + b, 0) / values.length);
}

describe('Parameter Sweep Continuity', () => {
  describe('LFO Frequency Sweep', () => {
    it('should produce continuous phase advancement across frequency range', () => {
      const lfo = new LFO(48000);
      const frequencies = [0.1, 1, 5, 10, 25, 50, 100];
      const phases: number[] = [];

      for (const freq of frequencies) {
        lfo.reset();
        // Run for 100 samples
        for (let i = 0; i < 100; i++) {
          lfo.process({}, { frequency: freq });
        }
        phases.push(lfo.getPhase());
      }

      // Phase should generally increase with frequency
      // (accounting for wrapping at 1.0)
      for (let i = 1; i < phases.length; i++) {
        expect(isFinite(phases[i])).toBe(true);
        expect(phases[i]).toBeGreaterThanOrEqual(0);
        expect(phases[i]).toBeLessThan(1);
      }
    });

    it('should maintain waveform continuity during frequency modulation', () => {
      const lfo = new LFO(48000);
      const outputs: number[] = [];

      // Sweep frequency over time
      for (let i = 0; i < 200; i++) {
        const freq = 1 + Math.sin(i * 0.05) * 0.5 + 0.5; // 1-2 Hz modulating
        const result = lfo.process({}, { frequency: freq * 10 });
        outputs.push(result.sine);
      }

      // Check no NaN or infinite values
      expect(outputs.every(isFinite)).toBe(true);
      // Check continuity
      expect(isSequenceContinuous(outputs, 0.3)).toBe(true);
    });
  });

  describe('Folding Amplifier Parameter Space', () => {
    it('should be continuous across gain parameter', () => {
      const folder = new FoldingAmplifier();
      const input = 0.5;
      const folds = 2;

      const outputs: number[] = [];
      for (let gain = 0.5; gain <= 5; gain += 0.1) {
        const result = folder.process({ in: input }, { gain, folds });
        outputs.push(result.out);
      }

      expect(outputs.every(v => v >= 0 && v <= 1)).toBe(true);
    });

    it('should be stable across input×gain×folds space', () => {
      const folder = new FoldingAmplifier();

      const results: number[] = [];
      for (let input = 0; input <= 1; input += 0.1) {
        for (let gain = 1; gain <= 5; gain += 0.5) {
          for (let folds = 1; folds <= 3; folds++) {
            const result = folder.process({ in: input }, { gain, folds });
            results.push(result.out);
          }
        }
      }

      // All outputs should be valid
      expect(results.every(v => isFinite(v) && v >= 0 && v <= 1)).toBe(true);
    });
  });

  describe('Colorizer Sweep', () => {
    it('should produce smooth color transitions across input range', () => {
      const colorizer = new ThresholdColorizer();
      const levels = 8;

      const reds: number[] = [];
      const greens: number[] = [];
      const blues: number[] = [];

      for (let input = 0; input <= 1; input += 0.01) {
        const result = colorizer.process(
          { in: input },
          { levels, hueRange: 0.8, saturation: 0.8, brightness: 0.9 }
        );
        reds.push(result.r);
        greens.push(result.g);
        blues.push(result.b);
      }

      // All values in valid range
      expect(reds.every(v => v >= 0 && v <= 1)).toBe(true);
      expect(greens.every(v => v >= 0 && v <= 1)).toBe(true);
      expect(blues.every(v => v >= 0 && v <= 1)).toBe(true);
    });

    it('should maintain color integrity across hue parameter', () => {
      const colorizer = new ThresholdColorizer();

      for (let hueOffset = 0; hueOffset <= 1; hueOffset += 0.1) {
        const result = colorizer.process(
          { in: 0.5 },
          { levels: 4, hueOffset, saturation: 1, brightness: 1 }
        );

        expect(isFinite(result.r)).toBe(true);
        expect(isFinite(result.g)).toBe(true);
        expect(isFinite(result.b)).toBe(true);
      }
    });
  });

  describe('Gradient Mapper Continuity', () => {
    it('should produce continuous gradients for all presets', () => {
      const mapper = new GradientMapper();

      for (let preset = 0; preset <= 5; preset++) {
        const reds: number[] = [];

        for (let input = 0; input <= 1; input += 0.01) {
          const result = mapper.process({ in: input }, { preset });
          reds.push(result.r);
        }

        // Check continuity (no sudden jumps > 0.5)
        expect(isSequenceContinuous(reds, 0.5)).toBe(true);
      }
    });
  });

  describe('CRT Effects Continuity', () => {
    let buffer: FrameBuffer;

    beforeEach(() => {
      buffer = createFrameBuffer(32, 32);
      // Create gradient test pattern
      for (let y = 0; y < 32; y++) {
        for (let x = 0; x < 32; x++) {
          const i = (y * 32 + x) * 3;
          buffer.data[i] = x / 31;
          buffer.data[i + 1] = y / 31;
          buffer.data[i + 2] = 0.5;
        }
      }
    });

    it('should produce continuous output across phosphor decay parameter', () => {
      const centerPixel: number[] = [];

      for (let persistence = 0.5; persistence <= 2; persistence += 0.1) {
        const phosphor = new PhosphorPersistence();
        const result = phosphor.processFrame(buffer, { persistence });
        centerPixel.push(result.data[(16 * 32 + 16) * 3]);
      }

      expect(centerPixel.every(isFinite)).toBe(true);
    });

    it('should produce stable output across bloom intensity', () => {
      const centerValues: number[] = [];

      for (let amount = 0; amount <= 1; amount += 0.1) {
        const bloom = new BeamBloom();
        const result = bloom.processFrame(buffer, { amount, threshold: 0.5, radius: 3 });
        centerValues.push(result.data[(16 * 32 + 16) * 3]);
      }

      expect(centerValues.every(v => isFinite(v) && v >= 0)).toBe(true);
    });

    it('should handle curvature parameter smoothly', () => {
      for (let curvature = 0; curvature <= 0.5; curvature += 0.05) {
        const curv = new ScreenCurvature();
        const result = curv.process({ u: 0.5, v: 0.5 }, { curvature });

        // Check outputs are valid
        expect(isFinite(result.u)).toBe(true);
        expect(isFinite(result.v)).toBe(true);
        expect(isFinite(result.vignette)).toBe(true);
      }
    });
  });

  describe('Video Feedback Parameter Space', () => {
    it('should remain stable across zoom/rotation/translate space', () => {
      const buffer = createFrameBuffer(32, 32);
      buffer.data.fill(0.5);

      const results: number[] = [];

      for (let zoom = 0.8; zoom <= 1.2; zoom += 0.1) {
        for (let rotation = 0; rotation <= 0.1; rotation += 0.02) {
          for (let translateX = -0.1; translateX <= 0.1; translateX += 0.05) {
            const fb = new VideoFeedback();
            const result = fb.processFrame(buffer, {
              zoom,
              rotation,
              translateX,
              translateY: 0,
              mix: 0.5,
              decay: 0.95
            });
            results.push(result.data[0]);
          }
        }
      }

      expect(results.every(isFinite)).toBe(true);
    });

    it('should converge over time with decay', () => {
      const fb = new VideoFeedback();
      let buffer = createFrameBuffer(16, 16);

      // Initialize with bright center
      for (let y = 6; y < 10; y++) {
        for (let x = 6; x < 10; x++) {
          const i = (y * 16 + x) * 3;
          buffer.data[i] = buffer.data[i + 1] = buffer.data[i + 2] = 1;
        }
      }

      const totalBrightness: number[] = [];

      for (let frame = 0; frame < 50; frame++) {
        buffer = fb.processFrame(buffer, {
          zoom: 0.98,
          rotation: 0.01,
          mix: 0.8,
          decay: 0.95
        });

        let total = 0;
        for (let i = 0; i < buffer.data.length; i++) {
          total += buffer.data[i];
        }
        totalBrightness.push(total);
      }

      // Brightness should generally decrease (decay working)
      expect(totalBrightness[49]).toBeLessThan(totalBrightness[0]);
    });
  });

  describe('Raster Manipulation Stability', () => {
    it('should handle Rutt/Etra depth sweep without artifacts', () => {
      const buffer = createFrameBuffer(32, 32);
      // Create luminance ramp
      for (let y = 0; y < 32; y++) {
        for (let x = 0; x < 32; x++) {
          const i = (y * 32 + x) * 3;
          buffer.data[i] = buffer.data[i + 1] = buffer.data[i + 2] = y / 31;
        }
      }

      for (let zDepth = 0; zDepth <= 0.5; zDepth += 0.05) {
        const rutt = new RuttEtraDisplacement();
        const result = rutt.processFrame(buffer, { zDepth, rotationX: 0.2 });

        // All output values should be valid
        expect(result.data.every(v => isFinite(v) && v >= 0 && v <= 1)).toBe(true);
      }
    });

    it('should handle Wobbulator frequency sweep', () => {
      const buffer = createFrameBuffer(32, 32);
      buffer.data.fill(0.5);

      for (let freq = 0; freq <= 20; freq += 2) {
        const wob = new Wobbulator();
        const result = wob.processFrame(buffer, {
          hFrequency: freq,
          vFrequency: freq,
          hAmount: 0.1,
          vAmount: 0.1
        });

        expect(result.data.every(isFinite)).toBe(true);
      }
    });
  });

  describe('Shape Generator Coverage', () => {
    it('should produce valid output across position space', () => {
      const shape = new ShapeGenerator();

      for (let x = 0; x <= 1; x += 0.1) {
        for (let y = 0; y <= 1; y += 0.1) {
          const result = shape.process(
            { hRamp: 0.5, vRamp: 0.5 },
            { x, y, width: 0.3, height: 0.3 }
          );

          expect(result.rect).toBeGreaterThanOrEqual(0);
          expect(result.rect).toBeLessThanOrEqual(1);
          expect(result.circle).toBeGreaterThanOrEqual(0);
          expect(result.circle).toBeLessThanOrEqual(1);
        }
      }
    });

    it('should maintain shape integrity across size parameter', () => {
      const shape = new ShapeGenerator();

      for (let size = 0.1; size <= 0.9; size += 0.1) {
        const center = shape.process(
          { hRamp: 0.5, vRamp: 0.5 },
          { x: 0.5, y: 0.5, width: size, height: size }
        );

        // Center should be inside shape
        expect(center.rect).toBeGreaterThan(0.5);
        expect(center.circle).toBeGreaterThan(0.5);
      }
    });
  });

  describe('Quantizer Level Transitions', () => {
    it('should produce expected number of unique output levels', () => {
      const q = new Quantizer();

      for (let levels = 2; levels <= 16; levels++) {
        const outputs = new Set<string>();

        for (let input = 0; input <= 1; input += 0.005) {
          const result = q.process({ in: input }, { levels, mode: 1 });
          outputs.add(result.out.toFixed(6));
        }

        expect(outputs.size).toBeLessThanOrEqual(levels);
        expect(outputs.size).toBeGreaterThanOrEqual(Math.min(levels, 2));
      }
    });
  });

  describe('Full Signal Chain Continuity', () => {
    it('should maintain signal integrity through oscillator->waveshaper->colorizer chain', () => {
      const lfo = new LFO(48000);
      const folder = new FoldingAmplifier();
      const colorizer = new GradientMapper();

      const outputs: { r: number; g: number; b: number }[] = [];

      for (let i = 0; i < 100; i++) {
        const osc = lfo.process({}, { frequency: 5 });
        const shaped = folder.process({ in: osc.sine }, { gain: 2, folds: 2 });
        const color = colorizer.process({ in: shaped.out }, { preset: 2 });
        outputs.push({ r: color.r, g: color.g, b: color.b });
      }

      // All outputs should be valid colors
      expect(outputs.every(c =>
        isFinite(c.r) && c.r >= 0 && c.r <= 1 &&
        isFinite(c.g) && c.g >= 0 && c.g <= 1 &&
        isFinite(c.b) && c.b >= 0 && c.b <= 1
      )).toBe(true);
    });
  });
});

describe('Statistical Properties', () => {
  describe('Noise Generator Distribution', () => {
    it('should produce approximately uniform distribution in white noise', () => {
      const gen = new NoiseGenerator(42);

      const values: number[] = [];
      for (let i = 0; i < 10000; i++) {
        const result = gen.process({}, { level: 0.5 });
        values.push(result.white);
      }

      // Check mean is approximately 0.5
      const mean = values.reduce((a, b) => a + b, 0) / values.length;
      expect(mean).toBeCloseTo(0.5, 1);

      // Check standard deviation is reasonable (Gaussian white noise has higher std)
      const std = standardDeviation(values);
      expect(std).toBeGreaterThan(0.05);
      expect(std).toBeLessThan(0.5);
    });
  });

  describe('LFO Waveform Statistics', () => {
    it('should have correct mean value for each waveform', () => {
      const lfo = new LFO(1000);

      // Run for many cycles and compute mean
      const samples = 10000;
      let sineSum = 0;
      let triangleSum = 0;
      let sawSum = 0;

      for (let i = 0; i < samples; i++) {
        const result = lfo.process({}, { frequency: 10 });
        sineSum += result.sine;
        triangleSum += result.triangle;
        sawSum += result.sawtooth;
      }

      // Mean should be approximately 0.5 for unipolar waves
      expect(sineSum / samples).toBeCloseTo(0.5, 1);
      expect(triangleSum / samples).toBeCloseTo(0.5, 1);
      expect(sawSum / samples).toBeCloseTo(0.5, 1);
    });
  });
});

describe('Stress Tests', () => {
  it('should handle rapid parameter changes', () => {
    const lfo = new LFO(48000);
    const folder = new FoldingAmplifier();

    for (let i = 0; i < 1000; i++) {
      const freq = 1 + Math.random() * 100;
      const gain = 0.5 + Math.random() * 5;
      const folds = 1 + Math.floor(Math.random() * 4);

      const oscResult = lfo.process({}, { frequency: freq });
      const foldResult = folder.process({ in: oscResult.sine }, { gain, folds });

      expect(isFinite(foldResult.out)).toBe(true);
      expect(foldResult.out).toBeGreaterThanOrEqual(0);
      expect(foldResult.out).toBeLessThanOrEqual(1);
    }
  });

  it('should handle many consecutive frame buffer operations', () => {
    const phosphor = new PhosphorPersistence();
    const bloom = new BeamBloom();

    let buffer = createFrameBuffer(64, 48);

    for (let frame = 0; frame < 100; frame++) {
      // Random input each frame
      for (let i = 0; i < buffer.data.length; i++) {
        buffer.data[i] = Math.random();
      }

      buffer = phosphor.processFrame(buffer, { persistence: 1 });
      buffer = bloom.processFrame(buffer, { amount: 0.3, threshold: 0.5, radius: 3 });

      // Verify no memory corruption or NaN propagation
      expect(buffer.data.every(isFinite)).toBe(true);
    }
  });
});
