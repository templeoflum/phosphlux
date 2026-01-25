/**
 * Analog Video Synthesizer
 *
 * A faithful digital emulation of analog video synthesis techniques
 * from the 1960s to present day.
 *
 * Based on the designs and principles of:
 * - Paik/Abe Video Synthesizer
 * - Sandin Image Processor
 * - Stephen Beck Direct Video Synthesizer
 * - Rutt/Etra Video Synthesizer
 * - Jones Colorizer
 * - LZX Industries Modular Video
 *
 * @module analog-video-synth
 */

// Core types and utilities
export {
  // Constants
  CONTROL_RATE,
  NTSC_H_RATE,
  NTSC_V_RATE,
  NTSC_SUBCARRIER,
  PAL_H_RATE,
  PAL_V_RATE,
  PAL_SUBCARRIER,
  NTSC_TIMING,
  PAL_TIMING,

  // Types
  type Signal,
  type RGB,
  type YIQ,
  type VideoStandard,
  type TimingConfig,
  type FrameBuffer,
  type SyncSignals,
  type Port,
  type PortType,
  type Parameter,
  type Module,
  type Connection,
  type Patch,

  // Utilities
  clamp,
  lerp,
  mapRange,
  rgbToYiq,
  yiqToRgb,
  createFrameBuffer,
  getPixel,
  setPixel,
} from './core/types.js';

// Sync Generation
export {
  SyncGenerator,
  FrameSyncGenerator,
  createNTSCSyncGenerator,
  createPALSyncGenerator,
  type SyncGeneratorConfig,
} from './core/sync-generator.js';

// All Modules
export * from './modules/index.js';

// Synthesizer
export {
  VideoSynthesizer,
  createDefaultSynth,
  createRuttEtraPatch,
  type SynthesizerConfig,
} from './synth/synthesizer.js';
