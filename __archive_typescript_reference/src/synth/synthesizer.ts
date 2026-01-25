/**
 * Video Synthesizer
 *
 * Main synthesizer class that coordinates modules, routing, and frame processing.
 * Inspired by LZX Industries modular architecture with:
 * - Master sync generation
 * - Modular signal routing
 * - Frame-based and sample-based processing modes
 */

import {
  type Signal,
  type RGB,
  type FrameBuffer,
  type Module,
  type Patch,
  type Connection,
  createFrameBuffer,
  getPixel,
  setPixel,
  clamp,
} from '../core/types.js';

import { FrameSyncGenerator } from '../core/sync-generator.js';

import {
  VideoOscillator,
  RampGenerator,
  ShapeGenerator,
  NoiseGenerator,
  LFO,
} from '../modules/oscillators.js';

import {
  FoldingAmplifier,
  SoftClipper,
  Quantizer,
  Solarizer,
} from '../modules/waveshapers.js';

import {
  MatrixColorizer,
  ThresholdColorizer,
  LayeredColorizer,
  GradientMapper,
} from '../modules/colorizers.js';

import {
  CRTDisplay,
  PhosphorPersistence,
} from '../modules/crt-emulation.js';

import {
  VideoFeedback,
  FrameDelay,
} from '../modules/video-feedback.js';

import {
  RuttEtraDisplacement,
  Wobbulator,
  DeflectionProcessor,
} from '../modules/raster-manipulation.js';

import {
  VideoMixer,
  LumaKeyer,
} from '../modules/mixing.js';

/**
 * Module instance with ID and state
 */
interface ModuleInstance {
  id: string;
  module: Module;
  params: Record<string, number>;
}

/**
 * Signal bus for routing
 */
interface SignalBus {
  [key: string]: Signal;
}

/**
 * Synthesizer configuration
 */
export interface SynthesizerConfig {
  width: number;
  height: number;
  frameRate?: number;
  crtEmulation?: boolean;
}

/**
 * Main Video Synthesizer
 *
 * Provides a modular architecture for video synthesis with:
 * - Module instantiation and management
 * - Signal routing between modules
 * - Frame-based processing pipeline
 * - Patch save/load
 */
export class VideoSynthesizer {
  private config: SynthesizerConfig;
  private modules: Map<string, ModuleInstance> = new Map();
  private connections: Connection[] = [];

  // Frame buffers
  private outputBuffer: FrameBuffer;
  private workBuffer: FrameBuffer;

  // Built-in processors
  private syncGen: FrameSyncGenerator;
  private crtDisplay: CRTDisplay;
  private feedback: VideoFeedback;

  // Global signals
  private globalSignals: SignalBus = {};

  // Frame counter for animations
  private frameCount: number = 0;

  constructor(config: SynthesizerConfig) {
    this.config = {
      frameRate: 60,
      crtEmulation: true,
      ...config,
    };

    this.outputBuffer = createFrameBuffer(config.width, config.height);
    this.workBuffer = createFrameBuffer(config.width, config.height);

    this.syncGen = new FrameSyncGenerator(config.width, config.height);
    this.crtDisplay = new CRTDisplay(config.width, config.height);
    this.crtDisplay.setFrameRate(this.config.frameRate!);

    this.feedback = new VideoFeedback();

    // Initialize ramp generators for position signals
    this.initializeGlobalSignals();
  }

  private initializeGlobalSignals(): void {
    // Global time-based signals will be updated each frame
    this.globalSignals = {
      time: 0,
      frame: 0,
    };
  }

  /**
   * Add a module instance
   */
  addModule(id: string, moduleType: string, initialParams?: Record<string, number>): ModuleInstance {
    const module = this.createModule(moduleType);
    if (!module) {
      throw new Error(`Unknown module type: ${moduleType}`);
    }

    // Initialize params from defaults
    const params: Record<string, number> = {};
    for (const param of module.parameters) {
      params[param.name] = initialParams?.[param.name] ?? param.defaultValue;
    }

    const instance: ModuleInstance = { id, module, params };
    this.modules.set(id, instance);

    return instance;
  }

  /**
   * Create a module by type
   */
  private createModule(type: string): Module | null {
    switch (type) {
      // Oscillators
      case 'lfo': return new LFO();
      case 'video-osc': return new VideoOscillator();
      case 'ramp-gen': return new RampGenerator();
      case 'shape-gen': return new ShapeGenerator();
      case 'noise-gen': return new NoiseGenerator();

      // Waveshapers
      case 'folder': return new FoldingAmplifier();
      case 'soft-clip': return new SoftClipper();
      case 'quantizer': return new Quantizer();
      case 'solarize': return new Solarizer();

      // Colorizers
      case 'matrix-colorizer': return new MatrixColorizer();
      case 'threshold-colorizer': return new ThresholdColorizer();
      case 'layered-colorizer': return new LayeredColorizer();
      case 'gradient-mapper': return new GradientMapper();

      // CRT
      case 'phosphor': return new PhosphorPersistence();

      // Feedback
      case 'video-feedback': return new VideoFeedback();
      case 'frame-delay': return new FrameDelay();

      // Raster
      case 'rutt-etra': return new RuttEtraDisplacement();
      case 'wobbulator': return new Wobbulator();
      case 'deflection': return new DeflectionProcessor();

      // Mixing
      case 'video-mixer': return new VideoMixer();
      case 'luma-keyer': return new LumaKeyer();

      default:
        return null;
    }
  }

  /**
   * Remove a module
   */
  removeModule(id: string): void {
    this.modules.delete(id);
    // Remove connections involving this module
    this.connections = this.connections.filter(
      c => c.sourceModule !== id && c.destModule !== id
    );
  }

  /**
   * Connect two modules
   */
  connect(
    sourceModule: string,
    sourcePort: string,
    destModule: string,
    destPort: string,
    amount: number = 1
  ): void {
    this.connections.push({
      sourceModule,
      sourcePort,
      destModule,
      destPort,
      amount,
    });
  }

  /**
   * Disconnect modules
   */
  disconnect(sourceModule: string, sourcePort: string, destModule: string, destPort: string): void {
    this.connections = this.connections.filter(
      c => !(c.sourceModule === sourceModule && c.sourcePort === sourcePort &&
             c.destModule === destModule && c.destPort === destPort)
    );
  }

  /**
   * Set module parameter
   */
  setParam(moduleId: string, paramName: string, value: number): void {
    const instance = this.modules.get(moduleId);
    if (instance) {
      instance.params[paramName] = value;
    }
  }

  /**
   * Get module parameter
   */
  getParam(moduleId: string, paramName: string): number | undefined {
    return this.modules.get(moduleId)?.params[paramName];
  }

  /**
   * Process a single frame
   */
  processFrame(): FrameBuffer {
    // Update global time signals
    this.globalSignals.time = this.frameCount / (this.config.frameRate ?? 60);
    this.globalSignals.frame = this.frameCount;

    // Clear work buffer
    this.workBuffer.data.fill(0);

    // Process each pixel
    this.syncGen.reset();
    for (let y = 0; y < this.config.height; y++) {
      for (let x = 0; x < this.config.width; x++) {
        const pos = this.syncGen.getPosition();

        // Build signal bus with position signals
        const signals: SignalBus = {
          hRamp: pos.u,
          vRamp: pos.v,
          hCenter: pos.u * 2 - 1,
          vCenter: pos.v * 2 - 1,
          time: this.globalSignals.time,
          ...this.globalSignals,
        };

        // Process all modules in order and collect outputs
        for (const [id, instance] of this.modules) {
          // Update position for position-aware modules
          if ('updatePosition' in instance.module &&
              typeof (instance.module as VideoOscillator).updatePosition === 'function') {
            (instance.module as VideoOscillator).updatePosition(pos.u, pos.v);
          }

          // Gather inputs from connections
          const inputs: Record<string, Signal> = {};
          for (const conn of this.connections) {
            if (conn.destModule === id) {
              const sourceKey = `${conn.sourceModule}_${conn.sourcePort}`;
              if (sourceKey in signals) {
                inputs[conn.destPort] = signals[sourceKey] * (conn.amount ?? 1);
              }
            }
          }

          // Add position signals as default inputs
          inputs.hRamp = inputs.hRamp ?? pos.u;
          inputs.vRamp = inputs.vRamp ?? pos.v;

          // Process module
          const outputs = instance.module.process(inputs, instance.params);

          // Store outputs in signal bus
          for (const [port, value] of Object.entries(outputs)) {
            signals[`${id}_${port}`] = value;
          }
        }

        // Find final RGB output (from last colorizer or designated output module)
        let r = 0, g = 0, b = 0;

        // Look for RGB signals in priority order
        for (const [id] of [...this.modules].reverse()) {
          if (`${id}_r` in signals && `${id}_g` in signals && `${id}_b` in signals) {
            r = signals[`${id}_r`];
            g = signals[`${id}_g`];
            b = signals[`${id}_b`];
            break;
          }
        }

        // Write to work buffer
        setPixel(this.workBuffer, x, y, { r, g, b });

        this.syncGen.next();
      }
    }

    // Apply CRT emulation if enabled
    let outputFrame = this.workBuffer;
    if (this.config.crtEmulation) {
      outputFrame = this.crtDisplay.processFrame(this.workBuffer, {
        phosphorPersistence: 1,
        bloomAmount: 0.3,
        scanlineIntensity: 0.2,
        shadowMaskIntensity: 0.15,
        curvatureAmount: 0.05,
        vignetteAmount: 0.2,
      });
    }

    // Copy to output buffer
    this.outputBuffer.data.set(outputFrame.data);

    this.frameCount++;
    return this.outputBuffer;
  }

  /**
   * Get output buffer
   */
  getOutputBuffer(): FrameBuffer {
    return this.outputBuffer;
  }

  /**
   * Save current patch state
   */
  savePatch(name: string): Patch {
    const modules: Patch['modules'] = [];

    for (const [id, instance] of this.modules) {
      modules.push({
        id,
        type: instance.module.type,
        params: { ...instance.params },
      });
    }

    return {
      name,
      modules,
      connections: [...this.connections],
    };
  }

  /**
   * Load a patch
   */
  loadPatch(patch: Patch): void {
    // Clear existing state
    this.modules.clear();
    this.connections = [];

    // Add modules
    for (const mod of patch.modules) {
      this.addModule(mod.id, mod.type, mod.params);
    }

    // Add connections
    this.connections = [...patch.connections];
  }

  /**
   * Reset all modules
   */
  reset(): void {
    for (const instance of this.modules.values()) {
      instance.module.reset();
    }
    this.crtDisplay.reset();
    this.feedback.reset();
    this.frameCount = 0;
  }

  /**
   * Get frame count
   */
  getFrameCount(): number {
    return this.frameCount;
  }

  /**
   * Get configuration
   */
  getConfig(): SynthesizerConfig {
    return { ...this.config };
  }

  /**
   * List all modules
   */
  listModules(): Array<{ id: string; type: string; params: Record<string, number> }> {
    return [...this.modules.values()].map(m => ({
      id: m.id,
      type: m.module.type,
      params: { ...m.params },
    }));
  }

  /**
   * List all connections
   */
  listConnections(): Connection[] {
    return [...this.connections];
  }
}

/**
 * Create a synthesizer with a basic default patch
 */
export function createDefaultSynth(width: number = 720, height: number = 480): VideoSynthesizer {
  const synth = new VideoSynthesizer({ width, height });

  // Add a simple test pattern
  synth.addModule('ramp', 'ramp-gen');
  synth.addModule('colorizer', 'gradient-mapper', {
    preset: 2, // spectrum
  });

  synth.connect('ramp', 'radial', 'colorizer', 'in');

  return synth;
}

/**
 * Create a Rutt/Etra style patch
 */
export function createRuttEtraPatch(width: number = 720, height: number = 480): VideoSynthesizer {
  const synth = new VideoSynthesizer({ width, height, crtEmulation: true });

  // Create oscillating pattern
  synth.addModule('osc1', 'video-osc', {
    frequency: 4,
    waveform: 0, // sine
    orientation: 0, // horizontal
  });

  synth.addModule('osc2', 'video-osc', {
    frequency: 3,
    waveform: 0,
    orientation: 1, // vertical
  });

  // Mix oscillators
  synth.addModule('mixer', 'video-mixer', {
    mix: 0.5,
    mode: 2, // multiply
  });

  // Colorize
  synth.addModule('colorizer', 'threshold-colorizer', {
    levels: 8,
    hueOffset: 0.1,
    saturation: 0.9,
  });

  // Rutt/Etra displacement
  synth.addModule('rutt', 'rutt-etra', {
    zDepth: 0.15,
    rotationX: 0.2,
    scanlineGap: 0.3,
  });

  return synth;
}
