/**
 * CRT Emulation Module
 *
 * Faithful emulation of cathode ray tube display characteristics:
 *
 * - Phosphor persistence with per-channel decay (P22 standard)
 * - Beam bloom (bright areas spread due to space charge)
 * - Scanline structure
 * - Shadow mask / aperture grille patterns
 * - Curvature and geometry distortion
 * - NTSC encoding artifacts (chroma bandwidth, dot crawl)
 *
 * Based on real CRT physics and specifications from technical documentation.
 */

import {
  type Signal,
  type RGB,
  type FrameBuffer,
  type Module,
  type Port,
  type Parameter,
  clamp,
  lerp,
  createFrameBuffer,
  getPixel,
  setPixel,
} from '../core/types.js';

/**
 * P22 Phosphor decay time constants (in seconds)
 * Based on actual phosphor specifications
 */
export const P22_PHOSPHOR = {
  // Primary (fast) decay
  red: { tau1: 0.001, a1: 0.8 },      // ~1ms primary
  green: { tau1: 0.0005, a1: 0.85 },   // ~0.5ms primary
  blue: { tau1: 0.0003, a1: 0.9 },     // ~0.3ms primary
  // Secondary (slow) decay - the "afterglow"
  redSlow: { tau2: 0.030, a2: 0.2 },   // ~30ms secondary
  greenSlow: { tau2: 0.020, a2: 0.15 }, // ~20ms secondary
  blueSlow: { tau2: 0.010, a2: 0.1 },   // ~10ms secondary
};

/**
 * Configuration for CRT emulation
 */
export interface CRTConfig {
  width: number;
  height: number;
  phosphorType?: 'P22' | 'P31' | 'P4' | 'custom';
  scanlines?: boolean;
  curvature?: boolean;
  shadowMask?: boolean;
}

/**
 * Phosphor Persistence Processor
 *
 * Models the multi-stage exponential decay of CRT phosphors.
 * Each color channel (R, G, B) has different decay characteristics,
 * creating the characteristic temporal smearing of CRT displays.
 */
export class PhosphorPersistence implements Module {
  readonly type = 'phosphor';
  readonly name = 'Phosphor Persistence';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'persistence', defaultValue: 1, min: 0, max: 3, description: 'Persistence multiplier' },
    { name: 'phosphorType', defaultValue: 0, min: 0, max: 3, step: 1,
      description: '0=P22 (color TV), 1=P31 (green mono), 2=P4 (white), 3=custom' },
    { name: 'customTauR', defaultValue: 0.016, min: 0.001, max: 0.1, unit: 's' },
    { name: 'customTauG', defaultValue: 0.012, min: 0.001, max: 0.1, unit: 's' },
    { name: 'customTauB', defaultValue: 0.008, min: 0.001, max: 0.1, unit: 's' },
  ];

  // Previous frame buffer for persistence
  private prevFrame: FrameBuffer | null = null;
  private frameTime: number = 1 / 60; // Default 60fps

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const r = inputs.r ?? 0;
    const g = inputs.g ?? 0;
    const b = inputs.b ?? 0;

    // For per-pixel processing, we need previous values
    // This is a simplified version - full implementation needs frame buffer
    const persistence = params.persistence ?? 1;

    // Calculate decay factors based on phosphor type
    const { alphaR, alphaG, alphaB } = this.getDecayFactors(params);

    return {
      r: r,
      g: g,
      b: b,
    };
  }

  /**
   * Process a complete frame with persistence
   */
  processFrame(
    current: FrameBuffer,
    params: Record<string, number>
  ): FrameBuffer {
    const persistence = params.persistence ?? 1;

    // Initialize previous frame if needed
    if (!this.prevFrame ||
        this.prevFrame.width !== current.width ||
        this.prevFrame.height !== current.height) {
      this.prevFrame = createFrameBuffer(current.width, current.height);
    }

    const { alphaR, alphaG, alphaB } = this.getDecayFactors(params);

    // Create output buffer
    const output = createFrameBuffer(current.width, current.height);

    // Process each pixel
    for (let y = 0; y < current.height; y++) {
      for (let x = 0; x < current.width; x++) {
        const curr = getPixel(current, x, y);
        const prev = getPixel(this.prevFrame, x, y);

        // Multi-stage decay: L(t) = current * alpha + previous * (1 - alpha)
        // Higher alpha = faster decay = less persistence
        const outR = curr.r * alphaR + prev.r * (1 - alphaR);
        const outG = curr.g * alphaG + prev.g * (1 - alphaG);
        const outB = curr.b * alphaB + prev.b * (1 - alphaB);

        setPixel(output, x, y, {
          r: Math.max(curr.r, outR), // Phosphors can only add light
          g: Math.max(curr.g, outG),
          b: Math.max(curr.b, outB),
        });
      }
    }

    // Store for next frame
    this.prevFrame.data.set(output.data);

    return output;
  }

  private getDecayFactors(params: Record<string, number>): {
    alphaR: number;
    alphaG: number;
    alphaB: number;
  } {
    const persistence = params.persistence ?? 1;
    const phosphorType = Math.round(params.phosphorType ?? 0);

    let tauR: number, tauG: number, tauB: number;

    switch (phosphorType) {
      case 0: // P22 (standard color TV)
        tauR = 0.016;
        tauG = 0.012;
        tauB = 0.008;
        break;
      case 1: // P31 (green monochrome - oscilloscope)
        tauR = tauG = tauB = 0.032; // ~32ms, very persistent
        break;
      case 2: // P4 (white monochrome - fast)
        tauR = tauG = tauB = 0.0001; // ~0.1ms, very fast
        break;
      case 3: // Custom
        tauR = params.customTauR ?? 0.016;
        tauG = params.customTauG ?? 0.012;
        tauB = params.customTauB ?? 0.008;
        break;
      default:
        tauR = tauG = tauB = 0.016;
    }

    // Apply persistence multiplier
    tauR *= persistence;
    tauG *= persistence;
    tauB *= persistence;

    // Calculate alpha = 1 - exp(-frameTime / tau)
    // This is the portion of "new" value vs "old" value per frame
    const alphaR = 1 - Math.exp(-this.frameTime / tauR);
    const alphaG = 1 - Math.exp(-this.frameTime / tauG);
    const alphaB = 1 - Math.exp(-this.frameTime / tauB);

    return { alphaR, alphaG, alphaB };
  }

  /**
   * Set frame time for decay calculations
   */
  setFrameTime(seconds: number): void {
    this.frameTime = seconds;
  }

  reset(): void {
    this.prevFrame = null;
  }
}

/**
 * Beam Bloom Effect
 *
 * Models the electron beam spreading at high brightness
 * due to space charge effects (electron-electron repulsion).
 * Bright areas "glow" wider than dark areas.
 */
export class BeamBloom implements Module {
  readonly type = 'beam-bloom';
  readonly name = 'Beam Bloom';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'amount', defaultValue: 0.5, min: 0, max: 2, description: 'Bloom intensity' },
    { name: 'threshold', defaultValue: 0.7, min: 0, max: 1, description: 'Brightness threshold for bloom' },
    { name: 'radius', defaultValue: 3, min: 1, max: 10, description: 'Bloom radius in pixels' },
    { name: 'softness', defaultValue: 0.5, min: 0, max: 1, description: 'Bloom edge softness' },
  ];

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    // Per-pixel bloom requires spatial context
    // This is a placeholder - full implementation needs frame processing
    return {
      r: inputs.r ?? 0,
      g: inputs.g ?? 0,
      b: inputs.b ?? 0,
    };
  }

  /**
   * Process frame with bloom effect
   */
  processFrame(input: FrameBuffer, params: Record<string, number>): FrameBuffer {
    const amount = params.amount ?? 0.5;
    const threshold = params.threshold ?? 0.7;
    const radius = Math.round(params.radius ?? 3);
    const softness = params.softness ?? 0.5;

    const output = createFrameBuffer(input.width, input.height);

    // First pass: identify bright areas and create bloom mask
    const bloomMask = new Float32Array(input.width * input.height);

    for (let y = 0; y < input.height; y++) {
      for (let x = 0; x < input.width; x++) {
        const pixel = getPixel(input, x, y);
        const luminance = 0.299 * pixel.r + 0.587 * pixel.g + 0.114 * pixel.b;

        // Soft threshold
        const bloomAmount = smoothstep(threshold - softness * 0.2, threshold + softness * 0.2, luminance);
        bloomMask[y * input.width + x] = bloomAmount * luminance;
      }
    }

    // Second pass: blur the bloom mask and add to original
    for (let y = 0; y < input.height; y++) {
      for (let x = 0; x < input.width; x++) {
        const pixel = getPixel(input, x, y);

        // Simple box blur of bloom mask
        let bloomSum = 0;
        let samples = 0;

        for (let dy = -radius; dy <= radius; dy++) {
          for (let dx = -radius; dx <= radius; dx++) {
            const nx = x + dx;
            const ny = y + dy;

            if (nx >= 0 && nx < input.width && ny >= 0 && ny < input.height) {
              // Distance-based falloff
              const dist = Math.sqrt(dx * dx + dy * dy);
              if (dist <= radius) {
                const weight = 1 - (dist / radius);
                bloomSum += bloomMask[ny * input.width + nx] * weight;
                samples += weight;
              }
            }
          }
        }

        const bloom = samples > 0 ? bloomSum / samples : 0;

        // Add bloom to original (bloom is white/luminance)
        setPixel(output, x, y, {
          r: clamp(pixel.r + bloom * amount),
          g: clamp(pixel.g + bloom * amount),
          b: clamp(pixel.b + bloom * amount),
        });
      }
    }

    return output;
  }

  reset(): void {
    // Stateless
  }
}

/**
 * Scanline Generator
 *
 * Creates the characteristic horizontal line structure of CRT displays.
 * Models both the bright phosphor lines and dark gaps between them.
 */
export class ScanlineGenerator implements Module {
  readonly type = 'scanlines';
  readonly name = 'Scanlines';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'intensity', defaultValue: 0.3, min: 0, max: 1, description: 'Scanline darkness' },
    { name: 'thickness', defaultValue: 0.5, min: 0.1, max: 1, description: 'Scanline thickness' },
    { name: 'brightBoost', defaultValue: 1.2, min: 1, max: 2, description: 'Bright line boost' },
    { name: 'curvature', defaultValue: 0, min: 0, max: 0.5, description: 'Scanline curvature' },
  ];

  private currentY: number = 0;
  private height: number = 480;

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    const r = inputs.r ?? 0;
    const g = inputs.g ?? 0;
    const b = inputs.b ?? 0;

    const intensity = params.intensity ?? 0.3;
    const thickness = params.thickness ?? 0.5;
    const brightBoost = params.brightBoost ?? 1.2;

    // Calculate scanline position within a "pixel row"
    const scanlinePos = (this.currentY % 2) / 2;

    // Create smooth scanline profile
    let scanlineFactor: number;
    if (scanlinePos < thickness) {
      // On the bright part of the scanline
      scanlineFactor = brightBoost;
    } else {
      // In the dark gap - smooth falloff
      const gapPos = (scanlinePos - thickness) / (1 - thickness);
      scanlineFactor = lerp(brightBoost, 1 - intensity, smoothstep(0, 1, gapPos));
    }

    return {
      r: clamp(r * scanlineFactor),
      g: clamp(g * scanlineFactor),
      b: clamp(b * scanlineFactor),
    };
  }

  /**
   * Set current Y position for scanline calculation
   */
  setY(y: number, height: number): void {
    this.currentY = y;
    this.height = height;
  }

  /**
   * Process frame with scanlines
   */
  processFrame(input: FrameBuffer, params: Record<string, number>): FrameBuffer {
    const output = createFrameBuffer(input.width, input.height);
    const intensity = params.intensity ?? 0.3;
    const thickness = params.thickness ?? 0.5;
    const brightBoost = params.brightBoost ?? 1.2;

    for (let y = 0; y < input.height; y++) {
      // Scanline pattern repeats every 2 pixels (for 2x internal resolution effect)
      const scanlinePhase = (y % 2) / 2;

      let scanlineFactor: number;
      if (scanlinePhase < thickness) {
        scanlineFactor = brightBoost;
      } else {
        const gapPhase = (scanlinePhase - thickness) / (1 - thickness);
        scanlineFactor = lerp(brightBoost, 1 - intensity, gapPhase);
      }

      for (let x = 0; x < input.width; x++) {
        const pixel = getPixel(input, x, y);
        setPixel(output, x, y, {
          r: clamp(pixel.r * scanlineFactor),
          g: clamp(pixel.g * scanlineFactor),
          b: clamp(pixel.b * scanlineFactor),
        });
      }
    }

    return output;
  }

  reset(): void {
    this.currentY = 0;
  }
}

/**
 * Shadow Mask / Aperture Grille
 *
 * Models the physical structure that separates RGB phosphor stripes:
 * - Shadow mask: Dotted pattern (Trinitron-style has vertical stripes)
 * - Aperture grille: Vertical wire grid (Sony Trinitron)
 */
export class ShadowMask implements Module {
  readonly type = 'shadow-mask';
  readonly name = 'Shadow Mask';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'type', defaultValue: 0, min: 0, max: 2, step: 1,
      description: '0=shadow mask, 1=aperture grille, 2=slot mask' },
    { name: 'intensity', defaultValue: 0.3, min: 0, max: 1, description: 'Mask visibility' },
    { name: 'dotPitch', defaultValue: 3, min: 1, max: 6, step: 1, description: 'Pixel spacing' },
    { name: 'sharpness', defaultValue: 0.8, min: 0, max: 1, description: 'Mask edge sharpness' },
  ];

  private currentX: number = 0;
  private currentY: number = 0;

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    let r = inputs.r ?? 0;
    let g = inputs.g ?? 0;
    let b = inputs.b ?? 0;

    const type = Math.round(params.type ?? 0);
    const intensity = params.intensity ?? 0.3;
    const dotPitch = Math.round(params.dotPitch ?? 3);

    const x = this.currentX;
    const y = this.currentY;

    // Calculate mask pattern based on type
    let rMask = 1, gMask = 1, bMask = 1;

    switch (type) {
      case 0: // Shadow mask (triangular dot pattern)
        {
          const cellX = x % (dotPitch * 3);
          const cellY = y % (dotPitch * 2);
          const offset = (Math.floor(y / dotPitch) % 2) * dotPitch * 1.5;

          const subX = ((cellX + offset) % (dotPitch * 3)) / dotPitch;

          if (subX < 1) {
            rMask = 1; gMask = 1 - intensity; bMask = 1 - intensity;
          } else if (subX < 2) {
            rMask = 1 - intensity; gMask = 1; bMask = 1 - intensity;
          } else {
            rMask = 1 - intensity; gMask = 1 - intensity; bMask = 1;
          }
        }
        break;

      case 1: // Aperture grille (vertical stripes)
        {
          const stripeX = x % (dotPitch * 3);
          const stripe = Math.floor(stripeX / dotPitch);

          if (stripe === 0) {
            rMask = 1; gMask = 1 - intensity; bMask = 1 - intensity;
          } else if (stripe === 1) {
            rMask = 1 - intensity; gMask = 1; bMask = 1 - intensity;
          } else {
            rMask = 1 - intensity; gMask = 1 - intensity; bMask = 1;
          }
        }
        break;

      case 2: // Slot mask (rectangular arrangement)
        {
          const slotX = x % (dotPitch * 3);
          const slotY = y % (dotPitch * 2);
          const stripe = Math.floor(slotX / dotPitch);

          // Add vertical gaps
          const inGap = slotY >= dotPitch * 1.5;
          const gapFactor = inGap ? (1 - intensity * 0.5) : 1;

          if (stripe === 0) {
            rMask = gapFactor; gMask = (1 - intensity) * gapFactor; bMask = (1 - intensity) * gapFactor;
          } else if (stripe === 1) {
            rMask = (1 - intensity) * gapFactor; gMask = gapFactor; bMask = (1 - intensity) * gapFactor;
          } else {
            rMask = (1 - intensity) * gapFactor; gMask = (1 - intensity) * gapFactor; bMask = gapFactor;
          }
        }
        break;
    }

    return {
      r: r * rMask,
      g: g * gMask,
      b: b * bMask,
    };
  }

  /**
   * Set current position
   */
  setPosition(x: number, y: number): void {
    this.currentX = x;
    this.currentY = y;
  }

  reset(): void {
    this.currentX = 0;
    this.currentY = 0;
  }
}

/**
 * Screen Curvature
 *
 * Models the curved glass surface of CRT displays.
 * Creates barrel distortion and edge darkening (vignette).
 */
export class ScreenCurvature implements Module {
  readonly type = 'curvature';
  readonly name = 'Screen Curvature';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'u', type: 'signal', defaultValue: 0, description: 'Horizontal position (0-1)' },
    { name: 'v', type: 'signal', defaultValue: 0, description: 'Vertical position (0-1)' },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'u', type: 'signal', defaultValue: 0, description: 'Distorted horizontal' },
    { name: 'v', type: 'signal', defaultValue: 0, description: 'Distorted vertical' },
    { name: 'vignette', type: 'signal', defaultValue: 1, description: 'Edge darkening factor' },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'curvature', defaultValue: 0.1, min: 0, max: 0.5, description: 'Barrel distortion amount' },
    { name: 'cornerRadius', defaultValue: 0.03, min: 0, max: 0.1, description: 'Corner rounding' },
    { name: 'vignette', defaultValue: 0.3, min: 0, max: 1, description: 'Edge darkening' },
    { name: 'vignetteSmooth', defaultValue: 0.5, min: 0, max: 1, description: 'Vignette softness' },
  ];

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    let u = inputs.u ?? 0;
    let v = inputs.v ?? 0;

    const curvature = params.curvature ?? 0.1;
    const vignetteAmount = params.vignette ?? 0.3;
    const vignetteSmooth = params.vignetteSmooth ?? 0.5;

    // Convert to centered coordinates (-1 to 1)
    const cx = u * 2 - 1;
    const cy = v * 2 - 1;

    // Calculate distance from center
    const dist = Math.sqrt(cx * cx + cy * cy);

    // Apply barrel distortion
    const distortion = 1 + curvature * dist * dist;
    const dx = cx * distortion;
    const dy = cy * distortion;

    // Convert back to 0-1 range
    const uOut = (dx + 1) / 2;
    const vOut = (dy + 1) / 2;

    // Calculate vignette (corner darkening)
    const vignetteBase = 1 - dist * dist * vignetteAmount;
    const vignette = smoothstep(0, vignetteSmooth, vignetteBase);

    return {
      u: uOut,
      v: vOut,
      vignette: clamp(vignette),
    };
  }

  /**
   * Apply curvature to UV coordinates
   */
  distortUV(u: number, v: number, curvature: number): { u: number; v: number } {
    const cx = u * 2 - 1;
    const cy = v * 2 - 1;
    const dist = Math.sqrt(cx * cx + cy * cy);
    const distortion = 1 + curvature * dist * dist;

    return {
      u: (cx * distortion + 1) / 2,
      v: (cy * distortion + 1) / 2,
    };
  }

  reset(): void {
    // Stateless
  }
}

/**
 * Complete CRT Display Emulation
 *
 * Combines all CRT effects into a single processing pipeline.
 */
export class CRTDisplay {
  private phosphor: PhosphorPersistence;
  private bloom: BeamBloom;
  private scanlines: ScanlineGenerator;
  private shadowMask: ShadowMask;
  private curvature: ScreenCurvature;

  private width: number;
  private height: number;

  constructor(width: number = 720, height: number = 480) {
    this.width = width;
    this.height = height;
    this.phosphor = new PhosphorPersistence();
    this.bloom = new BeamBloom();
    this.scanlines = new ScanlineGenerator();
    this.shadowMask = new ShadowMask();
    this.curvature = new ScreenCurvature();
  }

  /**
   * Process a complete frame through all CRT effects
   */
  processFrame(
    input: FrameBuffer,
    params: {
      phosphorPersistence?: number;
      phosphorType?: number;
      bloomAmount?: number;
      bloomThreshold?: number;
      scanlineIntensity?: number;
      shadowMaskType?: number;
      shadowMaskIntensity?: number;
      curvatureAmount?: number;
      vignetteAmount?: number;
    } = {}
  ): FrameBuffer {
    let frame = input;

    // 1. Apply phosphor persistence
    frame = this.phosphor.processFrame(frame, {
      persistence: params.phosphorPersistence ?? 1,
      phosphorType: params.phosphorType ?? 0,
    });

    // 2. Apply beam bloom
    if ((params.bloomAmount ?? 0.5) > 0) {
      frame = this.bloom.processFrame(frame, {
        amount: params.bloomAmount ?? 0.5,
        threshold: params.bloomThreshold ?? 0.7,
        radius: 3,
      });
    }

    // 3. Apply scanlines
    if ((params.scanlineIntensity ?? 0.3) > 0) {
      frame = this.scanlines.processFrame(frame, {
        intensity: params.scanlineIntensity ?? 0.3,
        thickness: 0.5,
      });
    }

    // 4. Apply shadow mask and curvature (per-pixel)
    const output = createFrameBuffer(frame.width, frame.height);
    const curvatureAmount = params.curvatureAmount ?? 0.1;
    const vignetteAmount = params.vignetteAmount ?? 0.3;
    const maskType = params.shadowMaskType ?? 0;
    const maskIntensity = params.shadowMaskIntensity ?? 0.3;

    for (let y = 0; y < frame.height; y++) {
      for (let x = 0; x < frame.width; x++) {
        // Apply curvature distortion
        const u = x / frame.width;
        const v = y / frame.height;
        const distorted = this.curvature.distortUV(u, v, curvatureAmount);

        // Sample with distorted coordinates (nearest neighbor for now)
        const srcX = Math.floor(clamp(distorted.u) * (frame.width - 1));
        const srcY = Math.floor(clamp(distorted.v) * (frame.height - 1));

        // Check if outside screen bounds
        if (distorted.u < 0 || distorted.u > 1 || distorted.v < 0 || distorted.v > 1) {
          setPixel(output, x, y, { r: 0, g: 0, b: 0 });
          continue;
        }

        let pixel = getPixel(frame, srcX, srcY);

        // Apply shadow mask
        this.shadowMask.setPosition(x, y);
        const masked = this.shadowMask.process(
          { r: pixel.r, g: pixel.g, b: pixel.b },
          { type: maskType, intensity: maskIntensity, dotPitch: 3 }
        );

        // Apply vignette
        const cx = u * 2 - 1;
        const cy = v * 2 - 1;
        const dist = Math.sqrt(cx * cx + cy * cy);
        const vignette = clamp(1 - dist * dist * vignetteAmount);

        setPixel(output, x, y, {
          r: clamp(masked.r * vignette),
          g: clamp(masked.g * vignette),
          b: clamp(masked.b * vignette),
        });
      }
    }

    return output;
  }

  /**
   * Set frame time for phosphor calculations
   */
  setFrameRate(fps: number): void {
    this.phosphor.setFrameTime(1 / fps);
  }

  /**
   * Reset all state
   */
  reset(): void {
    this.phosphor.reset();
    this.bloom.reset();
    this.scanlines.reset();
    this.shadowMask.reset();
    this.curvature.reset();
  }
}

/**
 * Smoothstep function
 */
function smoothstep(edge0: number, edge1: number, x: number): number {
  const t = clamp((x - edge0) / (edge1 - edge0), 0, 1);
  return t * t * (3 - 2 * t);
}
