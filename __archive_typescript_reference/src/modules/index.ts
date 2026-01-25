/**
 * Module Exports
 *
 * All video synthesis modules organized by category.
 */

// Oscillators and Pattern Generators
export {
  LFO,
  VideoOscillator,
  RampGenerator,
  ShapeGenerator,
  NoiseGenerator,
  generateWaveform,
  type WaveformType,
} from './oscillators.js';

// Waveshapers and Nonlinear Processing
export {
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

// Colorizers
export {
  MatrixColorizer,
  ThresholdColorizer,
  LayeredColorizer,
  RGBEncoder,
  GradientMapper,
  ColorChord,
  rgbToHsv,
  hsvToRgb,
} from './colorizers.js';

// CRT Emulation
export {
  PhosphorPersistence,
  BeamBloom,
  ScanlineGenerator,
  ShadowMask,
  ScreenCurvature,
  CRTDisplay,
  P22_PHOSPHOR,
  type CRTConfig,
} from './crt-emulation.js';

// Video Feedback
export {
  VideoFeedback,
  MultiTapFeedback,
  FrameDelay,
} from './video-feedback.js';

// Raster Manipulation
export {
  RuttEtraDisplacement,
  Wobbulator,
  DeflectionProcessor,
  SlitScan,
} from './raster-manipulation.js';

// Mixing and Compositing
export {
  VideoMixer,
  LumaKeyer,
  ChromaKeyer,
  FourQuadrantMultiplier,
  Comparator,
  AdditiveRGBMixer,
} from './mixing.js';
