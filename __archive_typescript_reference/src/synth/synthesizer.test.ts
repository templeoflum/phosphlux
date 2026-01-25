/**
 * Tests for Video Synthesizer
 */

import { describe, it, expect, beforeEach } from 'vitest';
import {
  VideoSynthesizer,
  createDefaultSynth,
  createRuttEtraPatch,
} from './synthesizer.js';

describe('VideoSynthesizer', () => {
  let synth: VideoSynthesizer;

  beforeEach(() => {
    synth = new VideoSynthesizer({
      width: 320,
      height: 240,
      crtEmulation: false, // Disable for faster tests
    });
  });

  describe('initialization', () => {
    it('should initialize with correct config', () => {
      const config = synth.getConfig();

      expect(config.width).toBe(320);
      expect(config.height).toBe(240);
    });

    it('should start with no modules', () => {
      const modules = synth.listModules();
      expect(modules.length).toBe(0);
    });

    it('should start at frame 0', () => {
      expect(synth.getFrameCount()).toBe(0);
    });
  });

  describe('module management', () => {
    it('should add modules', () => {
      synth.addModule('osc1', 'video-osc');
      const modules = synth.listModules();

      expect(modules.length).toBe(1);
      expect(modules[0].id).toBe('osc1');
      expect(modules[0].type).toBe('video-osc');
    });

    it('should initialize module with default params', () => {
      synth.addModule('lfo', 'lfo');
      const modules = synth.listModules();
      const lfo = modules.find(m => m.id === 'lfo');

      expect(lfo?.params.frequency).toBeDefined();
    });

    it('should accept initial params', () => {
      synth.addModule('osc', 'video-osc', { frequency: 10 });
      const modules = synth.listModules();
      const osc = modules.find(m => m.id === 'osc');

      expect(osc?.params.frequency).toBe(10);
    });

    it('should remove modules', () => {
      synth.addModule('osc1', 'video-osc');
      synth.addModule('osc2', 'video-osc');

      synth.removeModule('osc1');

      const modules = synth.listModules();
      expect(modules.length).toBe(1);
      expect(modules[0].id).toBe('osc2');
    });

    it('should throw for unknown module types', () => {
      expect(() => synth.addModule('x', 'unknown-type')).toThrow();
    });
  });

  describe('parameter control', () => {
    beforeEach(() => {
      synth.addModule('osc', 'video-osc');
    });

    it('should set parameters', () => {
      synth.setParam('osc', 'frequency', 5);
      expect(synth.getParam('osc', 'frequency')).toBe(5);
    });

    it('should return undefined for non-existent param', () => {
      expect(synth.getParam('osc', 'nonexistent')).toBeUndefined();
    });

    it('should return undefined for non-existent module', () => {
      expect(synth.getParam('nonexistent', 'frequency')).toBeUndefined();
    });
  });

  describe('connections', () => {
    beforeEach(() => {
      synth.addModule('osc', 'video-osc');
      synth.addModule('colorizer', 'threshold-colorizer');
    });

    it('should add connections', () => {
      synth.connect('osc', 'out', 'colorizer', 'in');

      const connections = synth.listConnections();
      expect(connections.length).toBe(1);
      expect(connections[0].sourceModule).toBe('osc');
      expect(connections[0].destModule).toBe('colorizer');
    });

    it('should remove connections', () => {
      synth.connect('osc', 'out', 'colorizer', 'in');
      synth.disconnect('osc', 'out', 'colorizer', 'in');

      const connections = synth.listConnections();
      expect(connections.length).toBe(0);
    });

    it('should remove connections when module is removed', () => {
      synth.connect('osc', 'out', 'colorizer', 'in');
      synth.removeModule('osc');

      const connections = synth.listConnections();
      expect(connections.length).toBe(0);
    });
  });

  describe('frame processing', () => {
    it('should process frame', () => {
      synth.addModule('ramp', 'ramp-gen');
      synth.addModule('colorizer', 'gradient-mapper');

      const buffer = synth.processFrame();

      expect(buffer.width).toBe(320);
      expect(buffer.height).toBe(240);
      expect(buffer.data.length).toBe(320 * 240 * 3);
    });

    it('should increment frame counter', () => {
      synth.processFrame();
      expect(synth.getFrameCount()).toBe(1);

      synth.processFrame();
      expect(synth.getFrameCount()).toBe(2);
    });

    it('should reset frame counter on reset', () => {
      synth.processFrame();
      synth.processFrame();

      synth.reset();

      expect(synth.getFrameCount()).toBe(0);
    });
  });

  describe('patch save/load', () => {
    it('should save patch state', () => {
      synth.addModule('osc', 'video-osc', { frequency: 5 });
      synth.addModule('color', 'threshold-colorizer');
      synth.connect('osc', 'out', 'color', 'in');

      const patch = synth.savePatch('test-patch');

      expect(patch.name).toBe('test-patch');
      expect(patch.modules.length).toBe(2);
      expect(patch.connections.length).toBe(1);
    });

    it('should load patch state', () => {
      synth.addModule('osc', 'video-osc', { frequency: 5 });
      const patch = synth.savePatch('test');

      // Create new synth and load patch
      const newSynth = new VideoSynthesizer({ width: 320, height: 240 });
      newSynth.loadPatch(patch);

      const modules = newSynth.listModules();
      expect(modules.length).toBe(1);
      expect(modules[0].params.frequency).toBe(5);
    });
  });
});

describe('createDefaultSynth', () => {
  it('should create synth with default modules', () => {
    const synth = createDefaultSynth(320, 240);
    const modules = synth.listModules();

    expect(modules.length).toBeGreaterThan(0);
  });

  it('should be able to process frames', () => {
    const synth = createDefaultSynth(160, 120);
    const buffer = synth.processFrame();

    expect(buffer.data.length).toBe(160 * 120 * 3);
  });
});

describe('createRuttEtraPatch', () => {
  it('should create synth with Rutt/Etra modules', () => {
    const synth = createRuttEtraPatch(320, 240);
    const modules = synth.listModules();

    // Should have at least the rutt-etra module
    const hasRutt = modules.some(m => m.type === 'rutt-etra');
    expect(hasRutt).toBe(true);
  });
});
