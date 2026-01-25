/**
 * Tests for Sync Generator
 */

import { describe, it, expect, beforeEach } from 'vitest';
import {
  SyncGenerator,
  FrameSyncGenerator,
  createNTSCSyncGenerator,
  createPALSyncGenerator,
} from './sync-generator.js';

describe('SyncGenerator', () => {
  describe('NTSC', () => {
    let syncGen: SyncGenerator;

    beforeEach(() => {
      syncGen = createNTSCSyncGenerator(13500000); // 13.5 MHz sample rate
    });

    it('should initialize with correct timing', () => {
      const timing = syncGen.getTiming();

      expect(timing.hRate).toBeCloseTo(15734.264, 0);
      expect(timing.vRate).toBeCloseTo(59.94, 1);
      expect(timing.totalLines).toBe(525);
    });

    it('should generate sync pulses', () => {
      const signals = syncGen.tick();

      expect(signals.hSync).toBeDefined();
      expect(signals.vSync).toBeDefined();
      expect(signals.composite).toBeDefined();
    });

    it('should have hSync active at start of line', () => {
      syncGen.reset();
      const signals = syncGen.tick();

      expect(signals.hSync).toBe(1);
    });

    it('should generate ramps', () => {
      syncGen.reset();
      syncGen.setPosition(0, 100, 0); // Middle of active line

      const signals = syncGen.getCurrentSignals();

      expect(signals.hRamp).toBeGreaterThanOrEqual(0);
      expect(signals.hRamp).toBeLessThanOrEqual(1);
    });

    it('should detect active region correctly', () => {
      syncGen.reset();

      // In blanking at start
      syncGen.setPosition(0, 0, 0);
      expect(syncGen.isActive()).toBe(false);

      // In active region mid-line
      syncGen.setPosition(100, 500, 0);
      expect(syncGen.isActive()).toBe(true);
    });

    it('should wrap around at end of field', () => {
      syncGen.reset();

      // Advance through entire field
      const samplesPerField = syncGen.getSamplesPerField();
      for (let i = 0; i < samplesPerField; i++) {
        syncGen.tick();
      }

      // Should be back at start
      const signals = syncGen.getCurrentSignals();
      expect(signals.line).toBe(0);
      expect(signals.sample).toBe(0);
    });
  });

  describe('PAL', () => {
    let syncGen: SyncGenerator;

    beforeEach(() => {
      syncGen = createPALSyncGenerator(13500000);
    });

    it('should have correct PAL timing', () => {
      const timing = syncGen.getTiming();

      expect(timing.hRate).toBe(15625);
      expect(timing.vRate).toBe(50);
      expect(timing.totalLines).toBe(625);
    });
  });
});

describe('FrameSyncGenerator', () => {
  let syncGen: FrameSyncGenerator;

  beforeEach(() => {
    syncGen = new FrameSyncGenerator(720, 480);
  });

  it('should initialize with correct dimensions', () => {
    expect(syncGen.width).toBe(720);
    expect(syncGen.height).toBe(480);
  });

  it('should start at origin', () => {
    const pos = syncGen.getPosition();

    expect(pos.x).toBe(0);
    expect(pos.y).toBe(0);
    expect(pos.u).toBe(0);
    expect(pos.v).toBe(0);
  });

  it('should advance correctly', () => {
    syncGen.next();
    const pos = syncGen.getPosition();

    expect(pos.x).toBe(1);
    expect(pos.y).toBe(0);
  });

  it('should wrap at end of line', () => {
    for (let i = 0; i < 720; i++) {
      syncGen.next();
    }
    const pos = syncGen.getPosition();

    expect(pos.x).toBe(0);
    expect(pos.y).toBe(1);
  });

  it('should return false at end of frame', () => {
    // Go to last pixel
    syncGen.setPosition(719, 479);
    const result = syncGen.next();

    expect(result).toBe(false);
  });

  it('should generate correct ramps', () => {
    syncGen.setPosition(360, 240); // Center

    expect(syncGen.getHRamp()).toBeCloseTo(0.5, 2);
    expect(syncGen.getVRamp()).toBeCloseTo(0.5, 2);
  });

  it('should generate correct center signals', () => {
    syncGen.setPosition(0, 0);
    expect(syncGen.getHCenter()).toBe(-1);

    syncGen.setPosition(719, 479);
    expect(syncGen.getHCenter()).toBeCloseTo(1, 2);
  });

  it('should iterate over all pixels with forEach', () => {
    let count = 0;
    syncGen.forEach(() => {
      count++;
    });

    expect(count).toBe(720 * 480);
  });
});
