/**
 * Tests for Oscillator Modules
 */

import { describe, it, expect, beforeEach } from 'vitest';
import {
  LFO,
  VideoOscillator,
  RampGenerator,
  ShapeGenerator,
  NoiseGenerator,
  generateWaveform,
} from './oscillators.js';

describe('LFO', () => {
  let lfo: LFO;

  beforeEach(() => {
    lfo = new LFO(48000);
  });

  it('should initialize with zero phase', () => {
    expect(lfo.getPhase()).toBe(0);
  });

  it('should generate all waveform outputs', () => {
    const result = lfo.process({}, { frequency: 1 });

    expect(result.sine).toBeDefined();
    expect(result.triangle).toBeDefined();
    expect(result.sawtooth).toBeDefined();
    expect(result.square).toBeDefined();
  });

  it('should output values in 0-1 range', () => {
    for (let i = 0; i < 1000; i++) {
      const result = lfo.process({}, { frequency: 100 });

      expect(result.sine).toBeGreaterThanOrEqual(0);
      expect(result.sine).toBeLessThanOrEqual(1);
      expect(result.triangle).toBeGreaterThanOrEqual(0);
      expect(result.triangle).toBeLessThanOrEqual(1);
    }
  });

  it('should reset phase on trigger', () => {
    // Advance phase
    for (let i = 0; i < 100; i++) {
      lfo.process({}, { frequency: 100 });
    }

    expect(lfo.getPhase()).toBeGreaterThan(0);

    // Reset trigger (rising edge)
    lfo.process({ reset: 0 }, { frequency: 100 });
    lfo.process({ reset: 1 }, { frequency: 100 });

    // Phase should be small (reset happens, then advances by 1 sample)
    // At 48kHz sample rate with 100Hz, each sample advances by 100/48000 = ~0.002
    expect(lfo.getPhase()).toBeLessThan(0.01);
  });

  it('should respond to frequency parameter', () => {
    // Higher frequency = faster phase advance
    lfo.reset();
    lfo.process({}, { frequency: 1000 });
    const phase1 = lfo.getPhase();

    lfo.reset();
    lfo.process({}, { frequency: 100 });
    const phase2 = lfo.getPhase();

    expect(phase1).toBeGreaterThan(phase2);
  });
});

describe('VideoOscillator', () => {
  let osc: VideoOscillator;

  beforeEach(() => {
    osc = new VideoOscillator();
  });

  it('should generate output', () => {
    osc.updatePosition(0.5, 0.5);
    const result = osc.process({}, { frequency: 1 });

    expect(result.out).toBeDefined();
    expect(result.out).toBeGreaterThanOrEqual(0);
    expect(result.out).toBeLessThanOrEqual(1);
  });

  it('should respond to position updates', () => {
    // Use positions that produce different phase values after frequency multiplication
    // With frequency=3, position 0.1 → phase (0.1*3)%1 = 0.3
    // With frequency=3, position 0.4 → phase (0.4*3)%1 = 0.2
    osc.updatePosition(0.1, 0);
    const result1 = osc.process({}, { frequency: 3, waveform: 2 }); // sawtooth

    osc.updatePosition(0.4, 0);
    const result2 = osc.process({}, { frequency: 3, waveform: 2 });

    expect(result1.out).not.toBe(result2.out);
  });
});

describe('RampGenerator', () => {
  let ramp: RampGenerator;

  beforeEach(() => {
    ramp = new RampGenerator();
  });

  it('should generate all outputs', () => {
    const result = ramp.process({ hRamp: 0.5, vRamp: 0.5 }, {});

    expect(result.hEdge).toBeDefined();
    expect(result.hCenter).toBeDefined();
    expect(result.vEdge).toBeDefined();
    expect(result.vCenter).toBeDefined();
    expect(result.radial).toBeDefined();
    expect(result.angle).toBeDefined();
  });

  it('should output edge ramps equal to input', () => {
    const result = ramp.process({ hRamp: 0.3, vRamp: 0.7 }, {});

    expect(result.hEdge).toBeCloseTo(0.3, 5);
    expect(result.vEdge).toBeCloseTo(0.7, 5);
  });

  it('should calculate center ramps correctly', () => {
    // At center, center ramps should be high
    const center = ramp.process({ hRamp: 0.5, vRamp: 0.5 }, {});
    expect(center.hCenter).toBeCloseTo(1, 5);
    expect(center.vCenter).toBeCloseTo(1, 5);

    // At edge, center ramps should be low
    const edge = ramp.process({ hRamp: 0, vRamp: 0 }, {});
    expect(edge.hCenter).toBeLessThan(0.1);
    expect(edge.vCenter).toBeLessThan(0.1);
  });

  it('should calculate radial distance from center', () => {
    const center = ramp.process({ hRamp: 0.5, vRamp: 0.5 }, {});
    expect(center.radial).toBeCloseTo(0, 5);

    const corner = ramp.process({ hRamp: 0, vRamp: 0 }, {});
    expect(corner.radial).toBeCloseTo(1, 5);
  });
});

describe('ShapeGenerator', () => {
  let shape: ShapeGenerator;

  beforeEach(() => {
    shape = new ShapeGenerator();
  });

  it('should generate all shape outputs', () => {
    const result = shape.process({ hRamp: 0.5, vRamp: 0.5 }, {
      x: 0.5,
      y: 0.5,
      width: 0.3,
      height: 0.3,
    });

    expect(result.rect).toBeDefined();
    expect(result.circle).toBeDefined();
    expect(result.diamond).toBeDefined();
    expect(result.hBar).toBeDefined();
    expect(result.vBar).toBeDefined();
  });

  it('should have high values at shape center', () => {
    const result = shape.process({ hRamp: 0.5, vRamp: 0.5 }, {
      x: 0.5,
      y: 0.5,
      width: 0.5,
      height: 0.5,
    });

    expect(result.rect).toBeGreaterThan(0.9);
    expect(result.circle).toBeGreaterThan(0.9);
    expect(result.diamond).toBeGreaterThan(0.9);
  });

  it('should have low values outside shape', () => {
    const result = shape.process({ hRamp: 0, vRamp: 0 }, {
      x: 0.5,
      y: 0.5,
      width: 0.2,
      height: 0.2,
    });

    expect(result.rect).toBeLessThan(0.1);
    expect(result.circle).toBeLessThan(0.1);
    expect(result.diamond).toBeLessThan(0.1);
  });
});

describe('NoiseGenerator', () => {
  let noise: NoiseGenerator;

  beforeEach(() => {
    noise = new NoiseGenerator(12345);
  });

  it('should generate all noise types', () => {
    const result = noise.process({}, { level: 0.5 });

    expect(result.white).toBeDefined();
    expect(result.pink).toBeDefined();
    expect(result.brown).toBeDefined();
  });

  it('should output values in 0-1 range', () => {
    for (let i = 0; i < 100; i++) {
      const result = noise.process({}, { level: 0.1 });

      expect(result.white).toBeGreaterThanOrEqual(0);
      expect(result.white).toBeLessThanOrEqual(1);
    }
  });

  it('should respond to level parameter', () => {
    let sumHigh = 0;
    let sumLow = 0;

    noise.setSeed(12345);
    for (let i = 0; i < 100; i++) {
      const r = noise.process({}, { level: 0.5 });
      sumHigh += Math.abs(r.white - 0.5);
    }

    noise.setSeed(12345);
    for (let i = 0; i < 100; i++) {
      const r = noise.process({}, { level: 0.1 });
      sumLow += Math.abs(r.white - 0.5);
    }

    expect(sumHigh).toBeGreaterThan(sumLow);
  });
});

describe('generateWaveform', () => {
  it('should generate sine wave', () => {
    expect(generateWaveform('sine', 0)).toBeCloseTo(0.5, 5);
    expect(generateWaveform('sine', 0.25)).toBeCloseTo(1, 5);
    expect(generateWaveform('sine', 0.5)).toBeCloseTo(0.5, 5);
    expect(generateWaveform('sine', 0.75)).toBeCloseTo(0, 5);
  });

  it('should generate triangle wave', () => {
    expect(generateWaveform('triangle', 0)).toBeCloseTo(0, 5);
    expect(generateWaveform('triangle', 0.25)).toBeCloseTo(0.5, 5);
    expect(generateWaveform('triangle', 0.5)).toBeCloseTo(1, 5);
    expect(generateWaveform('triangle', 0.75)).toBeCloseTo(0.5, 5);
  });

  it('should generate sawtooth wave', () => {
    expect(generateWaveform('sawtooth', 0)).toBeCloseTo(0, 5);
    expect(generateWaveform('sawtooth', 0.5)).toBeCloseTo(0.5, 5);
    expect(generateWaveform('sawtooth', 0.99)).toBeCloseTo(0.99, 5);
  });

  it('should generate square wave', () => {
    expect(generateWaveform('square', 0)).toBe(1);
    expect(generateWaveform('square', 0.4)).toBe(1);
    expect(generateWaveform('square', 0.5)).toBe(0);
    expect(generateWaveform('square', 0.9)).toBe(0);
  });

  it('should respect pulse width for square wave', () => {
    expect(generateWaveform('square', 0.2, 0.3)).toBe(1);
    expect(generateWaveform('square', 0.4, 0.3)).toBe(0);
  });
});
