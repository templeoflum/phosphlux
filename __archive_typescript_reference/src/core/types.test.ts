/**
 * Tests for core types and utilities
 */

import { describe, it, expect } from 'vitest';
import {
  clamp,
  lerp,
  mapRange,
  rgbToYiq,
  yiqToRgb,
  createFrameBuffer,
  getPixel,
  setPixel,
  NTSC_TIMING,
  PAL_TIMING,
} from './types.js';

describe('clamp', () => {
  it('should return value when within range', () => {
    expect(clamp(0.5)).toBe(0.5);
    expect(clamp(0)).toBe(0);
    expect(clamp(1)).toBe(1);
  });

  it('should clamp values below minimum', () => {
    expect(clamp(-0.5)).toBe(0);
    expect(clamp(-100)).toBe(0);
  });

  it('should clamp values above maximum', () => {
    expect(clamp(1.5)).toBe(1);
    expect(clamp(100)).toBe(1);
  });

  it('should work with custom range', () => {
    expect(clamp(5, 0, 10)).toBe(5);
    expect(clamp(-5, 0, 10)).toBe(0);
    expect(clamp(15, 0, 10)).toBe(10);
    expect(clamp(0.5, -1, 1)).toBe(0.5);
  });
});

describe('lerp', () => {
  it('should interpolate between values', () => {
    expect(lerp(0, 10, 0)).toBe(0);
    expect(lerp(0, 10, 1)).toBe(10);
    expect(lerp(0, 10, 0.5)).toBe(5);
    expect(lerp(0, 10, 0.25)).toBe(2.5);
  });

  it('should handle negative values', () => {
    expect(lerp(-10, 10, 0.5)).toBe(0);
    expect(lerp(-5, -1, 0.5)).toBe(-3);
  });

  it('should extrapolate outside 0-1 range', () => {
    expect(lerp(0, 10, 2)).toBe(20);
    expect(lerp(0, 10, -1)).toBe(-10);
  });
});

describe('mapRange', () => {
  it('should map value from one range to another', () => {
    expect(mapRange(5, 0, 10, 0, 100)).toBe(50);
    expect(mapRange(0, 0, 10, 0, 100)).toBe(0);
    expect(mapRange(10, 0, 10, 0, 100)).toBe(100);
  });

  it('should handle inverted ranges', () => {
    expect(mapRange(0, 0, 10, 100, 0)).toBe(100);
    expect(mapRange(10, 0, 10, 100, 0)).toBe(0);
  });

  it('should handle negative ranges', () => {
    expect(mapRange(0, -1, 1, 0, 100)).toBe(50);
  });
});

describe('RGB/YIQ conversion', () => {
  it('should convert white correctly', () => {
    const white = { r: 1, g: 1, b: 1 };
    const yiq = rgbToYiq(white);

    expect(yiq.y).toBeCloseTo(1, 2);
    expect(yiq.i).toBeCloseTo(0, 2);
    expect(yiq.q).toBeCloseTo(0, 2);
  });

  it('should convert black correctly', () => {
    const black = { r: 0, g: 0, b: 0 };
    const yiq = rgbToYiq(black);

    expect(yiq.y).toBeCloseTo(0, 2);
    expect(yiq.i).toBeCloseTo(0, 2);
    expect(yiq.q).toBeCloseTo(0, 2);
  });

  it('should round-trip RGB through YIQ', () => {
    const colors = [
      { r: 1, g: 0, b: 0 },
      { r: 0, g: 1, b: 0 },
      { r: 0, g: 0, b: 1 },
      { r: 0.5, g: 0.5, b: 0.5 },
      { r: 0.3, g: 0.6, b: 0.9 },
    ];

    for (const original of colors) {
      const yiq = rgbToYiq(original);
      const restored = yiqToRgb(yiq);

      expect(restored.r).toBeCloseTo(original.r, 2);
      expect(restored.g).toBeCloseTo(original.g, 2);
      expect(restored.b).toBeCloseTo(original.b, 2);
    }
  });
});

describe('FrameBuffer', () => {
  it('should create buffer with correct dimensions', () => {
    const fb = createFrameBuffer(100, 50);

    expect(fb.width).toBe(100);
    expect(fb.height).toBe(50);
    expect(fb.data.length).toBe(100 * 50 * 3);
  });

  it('should initialize to zeros', () => {
    const fb = createFrameBuffer(10, 10);

    for (let i = 0; i < fb.data.length; i++) {
      expect(fb.data[i]).toBe(0);
    }
  });

  it('should set and get pixels correctly', () => {
    const fb = createFrameBuffer(100, 100);
    const color = { r: 0.5, g: 0.75, b: 0.25 };

    setPixel(fb, 50, 50, color);
    const retrieved = getPixel(fb, 50, 50);

    expect(retrieved.r).toBe(color.r);
    expect(retrieved.g).toBe(color.g);
    expect(retrieved.b).toBe(color.b);
  });

  it('should handle edge pixels', () => {
    const fb = createFrameBuffer(10, 10);

    // Top-left
    setPixel(fb, 0, 0, { r: 1, g: 0, b: 0 });
    expect(getPixel(fb, 0, 0).r).toBe(1);

    // Bottom-right
    setPixel(fb, 9, 9, { r: 0, g: 1, b: 0 });
    expect(getPixel(fb, 9, 9).g).toBe(1);
  });
});

describe('Timing Constants', () => {
  it('should have valid NTSC timing', () => {
    expect(NTSC_TIMING.hRate).toBeCloseTo(15734.264, 0);
    expect(NTSC_TIMING.vRate).toBeCloseTo(59.94, 1);
    expect(NTSC_TIMING.totalLines).toBe(525);
    expect(NTSC_TIMING.activeLines).toBe(480);
    expect(NTSC_TIMING.subcarrierFreq).toBe(3579545);
  });

  it('should have valid PAL timing', () => {
    expect(PAL_TIMING.hRate).toBe(15625);
    expect(PAL_TIMING.vRate).toBe(50);
    expect(PAL_TIMING.totalLines).toBe(625);
    expect(PAL_TIMING.activeLines).toBe(576);
  });
});
