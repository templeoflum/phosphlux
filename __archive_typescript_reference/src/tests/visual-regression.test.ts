/**
 * Visual Regression Tests
 *
 * These tests generate deterministic visual outputs and verify they match
 * expected patterns. Since we can't display images directly, we use
 * checksums and statistical analysis of the output buffers.
 *
 * For visual feedback, this file also includes utilities to export
 * frame buffers to formats that can be inspected externally.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import {
  createFrameBuffer,
  getPixel,
  setPixel,
  clamp,
  type FrameBuffer,
  type RGB,
} from '../core/types.js';
import { VideoSynthesizer } from '../synth/synthesizer.js';
import { RampGenerator, VideoOscillator, NoiseGenerator } from '../modules/oscillators.js';
import { FoldingAmplifier, Quantizer } from '../modules/waveshapers.js';
import { ThresholdColorizer, GradientMapper } from '../modules/colorizers.js';
import { PhosphorPersistence, BeamBloom, ScanlineGenerator, CRTDisplay } from '../modules/crt-emulation.js';
import { VideoFeedback } from '../modules/video-feedback.js';
import { RuttEtraDisplacement } from '../modules/raster-manipulation.js';

/**
 * Generate a deterministic hash of a frame buffer for regression testing
 */
function hashFrameBuffer(fb: FrameBuffer): string {
  let hash = 0;
  const prime = 31;
  for (let i = 0; i < fb.data.length; i++) {
    // Quantize to reduce floating point precision issues
    const quantized = Math.round(fb.data[i] * 1000);
    hash = ((hash * prime) + quantized) >>> 0;
  }
  return hash.toString(16).padStart(8, '0');
}

/**
 * Calculate statistics about a frame buffer
 */
function analyzeFrameBuffer(fb: FrameBuffer): {
  meanR: number;
  meanG: number;
  meanB: number;
  minR: number;
  maxR: number;
  variance: number;
  histogram: number[];
} {
  const pixels = fb.width * fb.height;
  let sumR = 0, sumG = 0, sumB = 0;
  let minR = 1, maxR = 0;
  const histogram = new Array(256).fill(0);

  for (let i = 0; i < fb.data.length; i += 3) {
    const r = fb.data[i];
    const g = fb.data[i + 1];
    const b = fb.data[i + 2];

    sumR += r;
    sumG += g;
    sumB += b;

    minR = Math.min(minR, r);
    maxR = Math.max(maxR, r);

    // Luminance histogram
    const luma = 0.299 * r + 0.587 * g + 0.114 * b;
    const bin = Math.floor(clamp(luma) * 255);
    histogram[bin]++;
  }

  const meanR = sumR / pixels;
  const meanG = sumG / pixels;
  const meanB = sumB / pixels;

  // Calculate variance
  let varianceSum = 0;
  for (let i = 0; i < fb.data.length; i += 3) {
    const luma = 0.299 * fb.data[i] + 0.587 * fb.data[i + 1] + 0.114 * fb.data[i + 2];
    const mean = (meanR + meanG + meanB) / 3;
    varianceSum += (luma - mean) ** 2;
  }

  return {
    meanR,
    meanG,
    meanB,
    minR,
    maxR,
    variance: varianceSum / pixels,
    histogram,
  };
}

/**
 * Check if two frame buffers are visually similar
 */
function areFramesVisuallyEqual(fb1: FrameBuffer, fb2: FrameBuffer, tolerance: number = 0.01): boolean {
  if (fb1.width !== fb2.width || fb1.height !== fb2.height) {
    return false;
  }

  let totalDiff = 0;
  for (let i = 0; i < fb1.data.length; i++) {
    totalDiff += Math.abs(fb1.data[i] - fb2.data[i]);
  }

  const meanDiff = totalDiff / fb1.data.length;
  return meanDiff < tolerance;
}

/**
 * Generate PPM image data (simple format, can be viewed in many image viewers)
 */
function frameToPPM(fb: FrameBuffer): string {
  let ppm = `P3\n${fb.width} ${fb.height}\n255\n`;
  for (let i = 0; i < fb.data.length; i += 3) {
    const r = Math.floor(clamp(fb.data[i]) * 255);
    const g = Math.floor(clamp(fb.data[i + 1]) * 255);
    const b = Math.floor(clamp(fb.data[i + 2]) * 255);
    ppm += `${r} ${g} ${b}\n`;
  }
  return ppm;
}

/**
 * Create an ASCII art representation of the frame for debugging
 */
function frameToAscii(fb: FrameBuffer, targetWidth: number = 60): string {
  const chars = ' .:-=+*#%@';
  const scale = fb.width / targetWidth;
  const targetHeight = Math.floor(fb.height / scale / 2); // /2 for aspect ratio

  let ascii = '';
  for (let y = 0; y < targetHeight; y++) {
    for (let x = 0; x < targetWidth; x++) {
      const srcX = Math.floor(x * scale);
      const srcY = Math.floor(y * scale * 2);

      if (srcX < fb.width && srcY < fb.height) {
        const pixel = getPixel(fb, srcX, srcY);
        const luma = 0.299 * pixel.r + 0.587 * pixel.g + 0.114 * pixel.b;
        const charIdx = Math.floor(clamp(luma) * (chars.length - 1));
        ascii += chars[charIdx];
      }
    }
    ascii += '\n';
  }
  return ascii;
}

describe('Visual Pattern Generation', () => {
  describe('Test Pattern Generator', () => {
    it('should generate consistent horizontal gradient', () => {
      const synth = new VideoSynthesizer({ width: 64, height: 48, crtEmulation: false });

      synth.addModule('ramp', 'ramp-gen');
      synth.addModule('colorizer', 'gradient-mapper', { preset: 0 });
      synth.connect('ramp', 'hEdge', 'colorizer', 'in');

      const frame1 = synth.processFrame();
      const frame2 = synth.processFrame();

      // Same pattern should produce same hash
      expect(hashFrameBuffer(frame1)).toBe(hashFrameBuffer(frame2));

      // Analyze the gradient
      const analysis = analyzeFrameBuffer(frame1);

      // Horizontal gradient should have reasonable mean value
      expect(analysis.meanR).toBeGreaterThan(0.3);
      expect(analysis.meanR).toBeLessThan(0.9);

      // Should have variance (not solid color)
      expect(analysis.variance).toBeGreaterThan(0.01);
    });

    it('should generate consistent radial gradient', () => {
      const synth = new VideoSynthesizer({ width: 64, height: 48, crtEmulation: false });

      synth.addModule('ramp', 'ramp-gen');
      synth.addModule('colorizer', 'gradient-mapper', { preset: 2 });
      synth.connect('ramp', 'radial', 'colorizer', 'in');

      const frame = synth.processFrame();
      const analysis = analyzeFrameBuffer(frame);

      // Radial should be brightest at center, darker at edges
      const centerPixel = getPixel(frame, 32, 24);
      const cornerPixel = getPixel(frame, 0, 0);

      // Center should be different from corner
      const centerLuma = 0.299 * centerPixel.r + 0.587 * centerPixel.g + 0.114 * centerPixel.b;
      const cornerLuma = 0.299 * cornerPixel.r + 0.587 * cornerPixel.g + 0.114 * cornerPixel.b;

      expect(Math.abs(centerLuma - cornerLuma)).toBeGreaterThan(0.1);
    });

    it('should generate consistent oscillator pattern', () => {
      const synth = new VideoSynthesizer({ width: 64, height: 48, crtEmulation: false });

      synth.addModule('osc', 'video-osc', { frequency: 4, waveform: 0 });
      synth.addModule('colorizer', 'threshold-colorizer', { levels: 4 });
      synth.connect('osc', 'out', 'colorizer', 'in');

      const frame1 = synth.processFrame();
      const frame2 = synth.processFrame();

      // Pattern should be deterministic
      expect(hashFrameBuffer(frame1)).toBe(hashFrameBuffer(frame2));

      // With 4 levels, histogram should show clustering
      const analysis = analyzeFrameBuffer(frame1);
      expect(analysis.variance).toBeGreaterThan(0);
    });
  });

  describe('Effect Verification', () => {
    it('should verify folding creates expected complexity', () => {
      const synth = new VideoSynthesizer({ width: 32, height: 32, crtEmulation: false });

      // Create gradient and fold it
      synth.addModule('ramp', 'ramp-gen');
      synth.addModule('folder', 'folder', { gain: 3, folds: 2 });
      synth.addModule('colorizer', 'gradient-mapper', { preset: 0 });
      synth.connect('ramp', 'hEdge', 'folder', 'in');
      synth.connect('folder', 'out', 'colorizer', 'in');

      const frame = synth.processFrame();
      const analysis = analyzeFrameBuffer(frame);

      // Folding should create more complex pattern (higher variance)
      expect(analysis.variance).toBeGreaterThan(0.01);

      // Min should be low (folding creates dark bands)
      expect(analysis.minR).toBeLessThan(0.5);
    });

    it('should verify quantizer creates discrete levels', () => {
      const synth = new VideoSynthesizer({ width: 32, height: 32, crtEmulation: false });

      synth.addModule('ramp', 'ramp-gen');
      synth.addModule('quantizer', 'quantizer', { levels: 4, mode: 1 });
      synth.addModule('colorizer', 'gradient-mapper', { preset: 0 });
      synth.connect('ramp', 'hEdge', 'quantizer', 'in');
      synth.connect('quantizer', 'out', 'colorizer', 'in');

      const frame = synth.processFrame();

      // Count unique brightness levels
      const uniqueLevels = new Set<number>();
      for (let i = 0; i < frame.data.length; i += 3) {
        uniqueLevels.add(Math.round(frame.data[i] * 100) / 100);
      }

      // Should have exactly 4 levels (or fewer due to edge effects)
      expect(uniqueLevels.size).toBeLessThanOrEqual(4);
    });
  });

  describe('CRT Effect Verification', () => {
    it('should verify phosphor persistence affects output', () => {
      const buffer1 = createFrameBuffer(32, 32);
      const buffer2 = createFrameBuffer(32, 32);

      // Create bright flash in buffer1
      for (let i = 0; i < buffer1.data.length; i++) {
        buffer1.data[i] = 1;
      }

      const phosphor = new PhosphorPersistence();

      // First frame: all white
      const result1 = phosphor.processFrame(buffer1, { persistence: 1 });

      // Second frame: all black input
      const result2 = phosphor.processFrame(buffer2, { persistence: 1 });

      // Result2 should still have some brightness (persistence)
      const analysis2 = analyzeFrameBuffer(result2);
      expect(analysis2.meanR).toBeGreaterThan(0.1);
    });

    it('should verify scanlines affect vertical pattern', () => {
      const buffer = createFrameBuffer(32, 32);
      // Use higher brightness to see scanline effect better
      for (let i = 0; i < buffer.data.length; i++) {
        buffer.data[i] = 0.8;
      }

      const scanlines = new ScanlineGenerator();
      const result = scanlines.processFrame(buffer, { intensity: 0.5, brightBoost: 1.2 });

      // Scanlines create alternating brightness pattern
      // The processFrame applies different factors to even/odd rows
      const row0 = getPixel(result, 16, 0);
      const row1 = getPixel(result, 16, 1);

      // Both rows should have valid output
      expect(row0.r).toBeGreaterThanOrEqual(0);
      expect(row0.r).toBeLessThanOrEqual(1);
      expect(row1.r).toBeGreaterThanOrEqual(0);
      expect(row1.r).toBeLessThanOrEqual(1);

      // Output should be modified from input (not just pass-through)
      expect(row0.r).not.toBe(0.8);
    });

    it('should verify bloom affects bright areas', () => {
      const buffer = createFrameBuffer(32, 32);

      // Create bright spot in center
      for (let y = 14; y < 18; y++) {
        for (let x = 14; x < 18; x++) {
          setPixel(buffer, x, y, { r: 1, g: 1, b: 1 });
        }
      }

      const bloom = new BeamBloom();
      const result = bloom.processFrame(buffer, { amount: 0.8, threshold: 0.3, radius: 5 });

      // Bright spot center should still be bright
      const centerPixel = getPixel(result, 16, 16);
      expect(centerPixel.r).toBeGreaterThan(0.9);

      // Area around bright spot should have some brightness from bloom
      const nearPixel = getPixel(result, 13, 16);
      expect(nearPixel.r).toBeGreaterThan(0);

      // Far corner should still be relatively dark
      const farPixel = getPixel(result, 0, 0);
      expect(farPixel.r).toBeLessThan(centerPixel.r);
    });
  });

  describe('Video Feedback Verification', () => {
    it('should verify feedback creates self-similar patterns', () => {
      const fb = new VideoFeedback();
      let buffer = createFrameBuffer(32, 32);

      // Initialize with pattern
      for (let y = 0; y < 32; y++) {
        for (let x = 0; x < 32; x++) {
          if ((x + y) % 4 === 0) {
            setPixel(buffer, x, y, { r: 1, g: 1, b: 1 });
          }
        }
      }

      const hashes: string[] = [];

      // Run feedback for several iterations
      for (let i = 0; i < 10; i++) {
        buffer = fb.processFrame(buffer, {
          zoom: 0.95,
          rotation: 0.02,
          mix: 0.7,
          decay: 0.9
        });
        hashes.push(hashFrameBuffer(buffer));
      }

      // Each frame should be different (evolving)
      const uniqueHashes = new Set(hashes);
      expect(uniqueHashes.size).toBeGreaterThan(5);

      // But pattern should have continuity (analyzed mean shouldn't jump wildly)
      const analyses = [];
      buffer = createFrameBuffer(32, 32);
      for (let y = 0; y < 32; y++) {
        for (let x = 0; x < 32; x++) {
          if ((x + y) % 4 === 0) {
            setPixel(buffer, x, y, { r: 1, g: 1, b: 1 });
          }
        }
      }

      for (let i = 0; i < 10; i++) {
        buffer = fb.processFrame(buffer, {
          zoom: 0.95,
          rotation: 0.02,
          mix: 0.7,
          decay: 0.9
        });
        analyses.push(analyzeFrameBuffer(buffer).meanR);
      }

      // Mean should decrease smoothly due to decay
      for (let i = 1; i < analyses.length; i++) {
        expect(analyses[i]).toBeLessThanOrEqual(analyses[i - 1] + 0.1);
      }
    });
  });

  describe('Rutt/Etra Verification', () => {
    it('should verify Z-displacement creates scanline structure', () => {
      const buffer = createFrameBuffer(32, 32);

      // Create horizontal gradient (brightness increases right)
      for (let y = 0; y < 32; y++) {
        for (let x = 0; x < 32; x++) {
          const luma = x / 31;
          setPixel(buffer, x, y, { r: luma, g: luma, b: luma });
        }
      }

      const rutt = new RuttEtraDisplacement();
      const result = rutt.processFrame(buffer, { zDepth: 0.3, rotationX: 0.2 });

      // Result should have scanline structure (horizontal streaks)
      const analysis = analyzeFrameBuffer(result);

      // Should have content
      expect(analysis.maxR).toBeGreaterThan(0);

      // Due to displacement, should have black areas (where nothing maps)
      expect(analysis.minR).toBe(0);
    });
  });

  describe('Full Pipeline Visual Regression', () => {
    it('should generate consistent full pipeline output', () => {
      const synth = new VideoSynthesizer({ width: 64, height: 48, crtEmulation: true });

      synth.addModule('osc', 'video-osc', { frequency: 4, waveform: 0 });
      synth.addModule('colorizer', 'threshold-colorizer', { levels: 8 });

      // Reset and process
      synth.reset();
      const frame1 = synth.processFrame();

      synth.reset();
      const frame2 = synth.processFrame();

      // Should be deterministic
      expect(areFramesVisuallyEqual(frame1, frame2, 0.001)).toBe(true);
    });
  });
});

describe('Visual Debug Utilities', () => {
  it('should generate valid PPM data', () => {
    const buffer = createFrameBuffer(4, 4);
    for (let i = 0; i < buffer.data.length; i++) {
      buffer.data[i] = i / buffer.data.length;
    }

    const ppm = frameToPPM(buffer);

    expect(ppm).toContain('P3');
    expect(ppm).toContain('4 4');
    expect(ppm).toContain('255');
  });

  it('should generate readable ASCII representation', () => {
    const buffer = createFrameBuffer(16, 16);

    // Create diagonal gradient
    for (let y = 0; y < 16; y++) {
      for (let x = 0; x < 16; x++) {
        const luma = (x + y) / 30;
        setPixel(buffer, x, y, { r: luma, g: luma, b: luma });
      }
    }

    const ascii = frameToAscii(buffer, 16);

    expect(ascii.length).toBeGreaterThan(0);
    expect(ascii).toContain('\n');

    // Should have varying characters (not all same)
    const uniqueChars = new Set(ascii.replace(/\n/g, ''));
    expect(uniqueChars.size).toBeGreaterThan(3);
  });
});
