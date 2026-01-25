/**
 * Raster Manipulation Modules
 *
 * Based on the Rutt/Etra Video Synthesizer (1972-1974) and Wobbulator techniques.
 *
 * The Rutt/Etra was the first commercially marketed DC-coupled raster manipulation
 * system. Its signature effect is Z-axis displacement where luminance modulates
 * vertical deflection, creating topographic 3D appearances.
 *
 * Key concepts:
 * - DC coupling allows stable positioning (not just distortion)
 * - Deflection ramps passed through analog computer
 * - Control voltages modulate deflection amplifiers
 * - Luminance routed to Z-axis for displacement
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
 * Rutt/Etra Z-Axis Displacement
 *
 * The signature Rutt/Etra effect: luminance modulates vertical position.
 * Bright areas push scanlines up or down, creating a terrain map
 * of the image's own brightness.
 *
 * Implementation: UV coordinate distortion where y += luminance * depth
 */
export class RuttEtraDisplacement implements Module {
  readonly type = 'rutt-etra';
  readonly name = 'Rutt/Etra Displacement';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
    { name: 'depthCv', type: 'cv', defaultValue: 0, description: 'Depth modulation' },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'zDepth', defaultValue: 0.2, min: -1, max: 1, description: 'Z-axis displacement amount' },
    { name: 'perspective', defaultValue: 0, min: 0, max: 1, description: 'Perspective effect' },
    { name: 'scanlineGap', defaultValue: 0.5, min: 0, max: 1, description: 'Gap between scanlines' },
    { name: 'brightness', defaultValue: 1, min: 0, max: 2, description: 'Output brightness' },
    { name: 'lineWidth', defaultValue: 1, min: 0.5, max: 3, description: 'Scanline width' },
    { name: 'rotationX', defaultValue: 0.3, min: -0.5, max: 0.5, description: 'X-axis rotation (tilt)' },
    { name: 'rotationY', defaultValue: 0, min: -0.5, max: 0.5, description: 'Y-axis rotation' },
    { name: 'centerX', defaultValue: 0.5, min: 0, max: 1, description: 'Perspective center X' },
    { name: 'centerY', defaultValue: 0.5, min: 0, max: 1, description: 'Perspective center Y' },
  ];

  // Source frame for sampling
  private sourceFrame: FrameBuffer | null = null;

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    // Per-pixel processing without frame context
    return {
      r: inputs.r ?? 0,
      g: inputs.g ?? 0,
      b: inputs.b ?? 0,
    };
  }

  /**
   * Process frame with Rutt/Etra displacement effect
   */
  processFrame(
    input: FrameBuffer,
    params: Record<string, number>
  ): FrameBuffer {
    const zDepth = params.zDepth ?? 0.2;
    const perspective = params.perspective ?? 0;
    const scanlineGap = params.scanlineGap ?? 0.5;
    const brightness = params.brightness ?? 1;
    const lineWidth = params.lineWidth ?? 1;
    const rotationX = params.rotationX ?? 0.3;
    const rotationY = params.rotationY ?? 0;
    const centerX = params.centerX ?? 0.5;
    const centerY = params.centerY ?? 0.5;

    const output = createFrameBuffer(input.width, input.height);

    // Number of scanlines based on gap setting
    const numScanlines = Math.floor(input.height * (1 - scanlineGap * 0.9));
    const scanlineSpacing = input.height / numScanlines;

    // Clear output to black
    output.data.fill(0);

    // Process each scanline
    for (let scanline = 0; scanline < numScanlines; scanline++) {
      const srcY = Math.floor((scanline / numScanlines) * input.height);
      const normalizedY = scanline / numScanlines;

      // Apply perspective (lines further away are smaller/closer together)
      const perspectiveScale = 1 - perspective * (normalizedY - centerY) * 0.5;

      for (let x = 0; x < input.width; x++) {
        // Get source pixel
        const pixel = getPixel(input, x, srcY);

        // Calculate luminance
        const luminance = 0.299 * pixel.r + 0.587 * pixel.g + 0.114 * pixel.b;

        // Calculate displacement
        // Z-depth modulates the Y position based on luminance
        const displacement = luminance * zDepth * input.height;

        // Apply 3D rotation transformations
        const normalizedX = (x / input.width - centerX) * 2;
        const normalizedSrcY = (normalizedY - centerY) * 2;

        // Simple 3D projection
        let projX = normalizedX;
        let projY = normalizedSrcY;

        // Apply X rotation (tilt forward/backward)
        if (rotationX !== 0) {
          const cosX = Math.cos(rotationX * Math.PI);
          const sinX = Math.sin(rotationX * Math.PI);
          const z = displacement / input.height;
          projY = projY * cosX - z * sinX;
        }

        // Apply Y rotation
        if (rotationY !== 0) {
          const cosY = Math.cos(rotationY * Math.PI);
          const sinY = Math.sin(rotationY * Math.PI);
          projX = projX * cosY;
        }

        // Convert back to screen coordinates
        const destX = Math.floor((projX / 2 + centerX) * input.width);
        const baseY = (projY / 2 + centerY) * input.height;
        const destY = Math.floor(baseY - displacement * perspectiveScale);

        // Draw the scanline point with width
        const halfWidth = Math.floor(lineWidth / 2);
        for (let dy = -halfWidth; dy <= halfWidth; dy++) {
          const finalY = destY + dy;

          if (destX >= 0 && destX < output.width &&
              finalY >= 0 && finalY < output.height) {
            // Intensity falls off with distance from center of line
            const intensity = 1 - Math.abs(dy) / (lineWidth + 0.5);

            const existing = getPixel(output, destX, finalY);
            setPixel(output, destX, finalY, {
              r: Math.max(existing.r, pixel.r * brightness * intensity),
              g: Math.max(existing.g, pixel.g * brightness * intensity),
              b: Math.max(existing.b, pixel.b * brightness * intensity),
            });
          }
        }
      }
    }

    return output;
  }

  reset(): void {
    this.sourceFrame = null;
  }
}

/**
 * Wobbulator / Scan Modulation
 *
 * Based on the Paik/Abe Wobbulator that places extra deflection yokes
 * on top of the CRT yoke, driven by audio signals.
 *
 * Audio signals deform the magnetic scanning pattern, creating
 * fluid spatial distortions.
 */
export class Wobbulator implements Module {
  readonly type = 'wobbulator';
  readonly name = 'Wobbulator';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
    { name: 'modX', type: 'cv', defaultValue: 0, description: 'Horizontal deflection modulation' },
    { name: 'modY', type: 'cv', defaultValue: 0, description: 'Vertical deflection modulation' },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'amountX', defaultValue: 0.1, min: 0, max: 0.5, description: 'X modulation amount' },
    { name: 'amountY', defaultValue: 0.1, min: 0, max: 0.5, description: 'Y modulation amount' },
    { name: 'frequencyX', defaultValue: 2, min: 0.1, max: 20, description: 'X modulation frequency' },
    { name: 'frequencyY', defaultValue: 3, min: 0.1, max: 20, description: 'Y modulation frequency' },
    { name: 'phaseX', defaultValue: 0, min: 0, max: 1, description: 'X phase offset' },
    { name: 'phaseY', defaultValue: 0.25, min: 0, max: 1, description: 'Y phase offset' },
    { name: 'waveformX', defaultValue: 0, min: 0, max: 2, step: 1, description: '0=sine, 1=triangle, 2=square' },
    { name: 'waveformY', defaultValue: 0, min: 0, max: 2, step: 1 },
    { name: 'inductanceRolloff', defaultValue: 0.5, min: 0, max: 1,
      description: 'Simulates coil inductance high-frequency rolloff' },
  ];

  // For frame-based processing
  private currentX: number = 0;
  private currentY: number = 0;
  private width: number = 720;
  private height: number = 480;

  // Low-pass filter state for inductance simulation
  private lpfStateX: number = 0;
  private lpfStateY: number = 0;

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    return {
      r: inputs.r ?? 0,
      g: inputs.g ?? 0,
      b: inputs.b ?? 0,
    };
  }

  /**
   * Process frame with wobbulator effect
   */
  processFrame(
    input: FrameBuffer,
    params: Record<string, number>
  ): FrameBuffer {
    const amountX = params.amountX ?? 0.1;
    const amountY = params.amountY ?? 0.1;
    const frequencyX = params.frequencyX ?? 2;
    const frequencyY = params.frequencyY ?? 3;
    const phaseX = params.phaseX ?? 0;
    const phaseY = params.phaseY ?? 0.25;
    const waveformX = Math.round(params.waveformX ?? 0);
    const waveformY = Math.round(params.waveformY ?? 0);
    const inductanceRolloff = params.inductanceRolloff ?? 0.5;

    const output = createFrameBuffer(input.width, input.height);

    for (let y = 0; y < input.height; y++) {
      const normalizedY = y / input.height;

      for (let x = 0; x < input.width; x++) {
        const normalizedX = x / input.width;

        // Calculate modulation signals
        let modX = this.generateWaveform(waveformX,
          normalizedX * frequencyX + normalizedY * frequencyY * 0.1 + phaseX);
        let modY = this.generateWaveform(waveformY,
          normalizedY * frequencyY + normalizedX * frequencyX * 0.1 + phaseY);

        // Apply inductance rolloff (simple lowpass approximation)
        // Real deflection coils have ~6dB/octave rolloff above 1kHz
        if (inductanceRolloff > 0) {
          const alpha = inductanceRolloff * 0.3;
          modX = modX * (1 - alpha) + this.lpfStateX * alpha;
          modY = modY * (1 - alpha) + this.lpfStateY * alpha;
          this.lpfStateX = modX;
          this.lpfStateY = modY;
        }

        // Calculate displaced sample position
        const srcX = x + modX * amountX * input.width;
        const srcY = y + modY * amountY * input.height;

        // Sample with bilinear interpolation
        const pixel = this.sampleBilinear(input, srcX, srcY);
        setPixel(output, x, y, pixel);
      }
    }

    return output;
  }

  private generateWaveform(type: number, phase: number): number {
    const p = phase % 1;
    switch (type) {
      case 0: // Sine
        return Math.sin(p * Math.PI * 2);
      case 1: // Triangle
        return p < 0.5 ? p * 4 - 1 : 3 - p * 4;
      case 2: // Square
        return p < 0.5 ? 1 : -1;
      default:
        return Math.sin(p * Math.PI * 2);
    }
  }

  private sampleBilinear(fb: FrameBuffer, x: number, y: number): RGB {
    // Clamp to valid range
    x = Math.max(0, Math.min(fb.width - 1, x));
    y = Math.max(0, Math.min(fb.height - 1, y));

    const x0 = Math.floor(x);
    const y0 = Math.floor(y);
    const x1 = Math.min(x0 + 1, fb.width - 1);
    const y1 = Math.min(y0 + 1, fb.height - 1);

    const fx = x - x0;
    const fy = y - y0;

    const p00 = getPixel(fb, x0, y0);
    const p10 = getPixel(fb, x1, y0);
    const p01 = getPixel(fb, x0, y1);
    const p11 = getPixel(fb, x1, y1);

    return {
      r: lerp(lerp(p00.r, p10.r, fx), lerp(p01.r, p11.r, fx), fy),
      g: lerp(lerp(p00.g, p10.g, fx), lerp(p01.g, p11.g, fx), fy),
      b: lerp(lerp(p00.b, p10.b, fx), lerp(p01.b, p11.b, fx), fy),
    };
  }

  reset(): void {
    this.lpfStateX = 0;
    this.lpfStateY = 0;
  }
}

/**
 * Deflection Processor
 *
 * Generic UV coordinate manipulation for raster effects.
 * Can combine multiple modulation sources.
 */
export class DeflectionProcessor implements Module {
  readonly type = 'deflection';
  readonly name = 'Deflection Processor';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
    { name: 'displaceX', type: 'cv', defaultValue: 0 },
    { name: 'displaceY', type: 'cv', defaultValue: 0 },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'scaleX', defaultValue: 1, min: 0.1, max: 4, description: 'Horizontal scale' },
    { name: 'scaleY', defaultValue: 1, min: 0.1, max: 4, description: 'Vertical scale' },
    { name: 'offsetX', defaultValue: 0, min: -1, max: 1, description: 'Horizontal offset' },
    { name: 'offsetY', defaultValue: 0, min: -1, max: 1, description: 'Vertical offset' },
    { name: 'rotation', defaultValue: 0, min: -1, max: 1, description: 'Rotation (-1 to 1 = -180° to 180°)' },
    { name: 'displaceAmountX', defaultValue: 0.1, min: 0, max: 1 },
    { name: 'displaceAmountY', defaultValue: 0.1, min: 0, max: 1 },
    { name: 'mirrorX', defaultValue: 0, min: 0, max: 1, step: 1, description: 'Mirror horizontally' },
    { name: 'mirrorY', defaultValue: 0, min: 0, max: 1, step: 1, description: 'Mirror vertically' },
    { name: 'wrapMode', defaultValue: 0, min: 0, max: 2, step: 1, description: '0=clamp, 1=wrap, 2=mirror' },
  ];

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    return {
      r: inputs.r ?? 0,
      g: inputs.g ?? 0,
      b: inputs.b ?? 0,
    };
  }

  /**
   * Process frame with deflection effects
   */
  processFrame(
    input: FrameBuffer,
    params: Record<string, number>,
    displaceMapX?: FrameBuffer,
    displaceMapY?: FrameBuffer
  ): FrameBuffer {
    const scaleX = params.scaleX ?? 1;
    const scaleY = params.scaleY ?? 1;
    const offsetX = params.offsetX ?? 0;
    const offsetY = params.offsetY ?? 0;
    const rotation = (params.rotation ?? 0) * Math.PI;
    const displaceAmountX = params.displaceAmountX ?? 0.1;
    const displaceAmountY = params.displaceAmountY ?? 0.1;
    const mirrorX = (params.mirrorX ?? 0) > 0.5;
    const mirrorY = (params.mirrorY ?? 0) > 0.5;
    const wrapMode = Math.round(params.wrapMode ?? 0);

    const output = createFrameBuffer(input.width, input.height);
    const cos = Math.cos(rotation);
    const sin = Math.sin(rotation);
    const centerX = input.width / 2;
    const centerY = input.height / 2;

    for (let y = 0; y < input.height; y++) {
      for (let x = 0; x < input.width; x++) {
        // Start with normalized coordinates
        let u = x / input.width;
        let v = y / input.height;

        // Apply mirror
        if (mirrorX) u = u < 0.5 ? u * 2 : (1 - u) * 2;
        if (mirrorY) v = v < 0.5 ? v * 2 : (1 - v) * 2;

        // Get displacement from maps if provided
        let dispX = 0, dispY = 0;
        if (displaceMapX) {
          const dispPixel = getPixel(displaceMapX, x, y);
          dispX = (dispPixel.r - 0.5) * 2 * displaceAmountX;
        }
        if (displaceMapY) {
          const dispPixel = getPixel(displaceMapY, x, y);
          dispY = (dispPixel.r - 0.5) * 2 * displaceAmountY;
        }

        // Apply transformations
        // 1. Center
        u -= 0.5;
        v -= 0.5;

        // 2. Scale
        u /= scaleX;
        v /= scaleY;

        // 3. Rotate
        const ru = u * cos - v * sin;
        const rv = u * sin + v * cos;
        u = ru;
        v = rv;

        // 4. Translate
        u -= offsetX;
        v -= offsetY;

        // 5. Displacement
        u += dispX;
        v += dispY;

        // 6. Un-center
        u += 0.5;
        v += 0.5;

        // Apply wrap mode
        switch (wrapMode) {
          case 0: // Clamp
            u = clamp(u);
            v = clamp(v);
            break;
          case 1: // Wrap
            u = ((u % 1) + 1) % 1;
            v = ((v % 1) + 1) % 1;
            break;
          case 2: // Mirror
            u = this.mirrorCoord(u);
            v = this.mirrorCoord(v);
            break;
        }

        // Sample source
        const srcX = u * (input.width - 1);
        const srcY = v * (input.height - 1);
        const pixel = this.sampleBilinear(input, srcX, srcY);
        setPixel(output, x, y, pixel);
      }
    }

    return output;
  }

  private mirrorCoord(c: number): number {
    const period = 2;
    const normalized = ((c % period) + period) % period;
    return normalized < 1 ? normalized : period - normalized;
  }

  private sampleBilinear(fb: FrameBuffer, x: number, y: number): RGB {
    x = Math.max(0, Math.min(fb.width - 1, x));
    y = Math.max(0, Math.min(fb.height - 1, y));

    const x0 = Math.floor(x);
    const y0 = Math.floor(y);
    const x1 = Math.min(x0 + 1, fb.width - 1);
    const y1 = Math.min(y0 + 1, fb.height - 1);

    const fx = x - x0;
    const fy = y - y0;

    const p00 = getPixel(fb, x0, y0);
    const p10 = getPixel(fb, x1, y0);
    const p01 = getPixel(fb, x0, y1);
    const p11 = getPixel(fb, x1, y1);

    return {
      r: lerp(lerp(p00.r, p10.r, fx), lerp(p01.r, p11.r, fx), fy),
      g: lerp(lerp(p00.g, p10.g, fx), lerp(p01.g, p11.g, fx), fy),
      b: lerp(lerp(p00.b, p10.b, fx), lerp(p01.b, p11.b, fx), fy),
    };
  }

  reset(): void {
    // Stateless
  }
}

/**
 * Slit Scan Effect
 *
 * Based on John Whitney's slit-scan technique, later used in 2001: A Space Odyssey.
 * Creates streaking/smearing effects by sampling across time.
 */
export class SlitScan implements Module {
  readonly type = 'slit-scan';
  readonly name = 'Slit Scan';

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
    { name: 'slitPosition', defaultValue: 0.5, min: 0, max: 1, description: 'Position of slit' },
    { name: 'slitWidth', defaultValue: 0.1, min: 0.01, max: 0.5, description: 'Width of slit' },
    { name: 'direction', defaultValue: 0, min: 0, max: 1, step: 1, description: '0=horizontal, 1=vertical' },
    { name: 'speed', defaultValue: 1, min: 0.1, max: 5, description: 'Scroll speed' },
  ];

  // History buffer for temporal sampling
  private historyBuffer: Float32Array[] = [];
  private historyIndex: number = 0;
  private maxHistory: number = 256;
  private width: number = 0;
  private height: number = 0;

  private ensureHistory(width: number, height: number): void {
    if (this.width !== width || this.height !== height) {
      this.width = width;
      this.height = height;
      this.historyBuffer = [];
      for (let i = 0; i < this.maxHistory; i++) {
        this.historyBuffer.push(new Float32Array(Math.max(width, height) * 3));
      }
      this.historyIndex = 0;
    }
  }

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    return {
      r: inputs.r ?? 0,
      g: inputs.g ?? 0,
      b: inputs.b ?? 0,
    };
  }

  /**
   * Process frame with slit-scan effect
   */
  processFrame(
    input: FrameBuffer,
    params: Record<string, number>
  ): FrameBuffer {
    this.ensureHistory(input.width, input.height);

    const slitPosition = params.slitPosition ?? 0.5;
    const slitWidth = params.slitWidth ?? 0.1;
    const direction = Math.round(params.direction ?? 0);
    const speed = params.speed ?? 1;

    const output = createFrameBuffer(input.width, input.height);

    if (direction === 0) {
      // Horizontal slit (samples vertical strip, outputs horizontal history)
      const slitX = Math.floor(slitPosition * input.width);

      // Store current slit in history
      const currentHistory = this.historyBuffer[this.historyIndex];
      for (let y = 0; y < input.height; y++) {
        const pixel = getPixel(input, slitX, y);
        currentHistory[y * 3] = pixel.r;
        currentHistory[y * 3 + 1] = pixel.g;
        currentHistory[y * 3 + 2] = pixel.b;
      }

      // Build output from history
      for (let x = 0; x < input.width; x++) {
        const historyOffset = Math.floor((input.width - 1 - x) * speed);
        const histIdx = (this.historyIndex - historyOffset + this.maxHistory * 100) % this.maxHistory;
        const hist = this.historyBuffer[histIdx];

        for (let y = 0; y < input.height; y++) {
          setPixel(output, x, y, {
            r: hist[y * 3],
            g: hist[y * 3 + 1],
            b: hist[y * 3 + 2],
          });
        }
      }
    } else {
      // Vertical slit (samples horizontal strip, outputs vertical history)
      const slitY = Math.floor(slitPosition * input.height);

      // Store current slit in history
      const currentHistory = this.historyBuffer[this.historyIndex];
      for (let x = 0; x < input.width; x++) {
        const pixel = getPixel(input, x, slitY);
        currentHistory[x * 3] = pixel.r;
        currentHistory[x * 3 + 1] = pixel.g;
        currentHistory[x * 3 + 2] = pixel.b;
      }

      // Build output from history
      for (let y = 0; y < input.height; y++) {
        const historyOffset = Math.floor((input.height - 1 - y) * speed);
        const histIdx = (this.historyIndex - historyOffset + this.maxHistory * 100) % this.maxHistory;
        const hist = this.historyBuffer[histIdx];

        for (let x = 0; x < input.width; x++) {
          setPixel(output, x, y, {
            r: hist[x * 3],
            g: hist[x * 3 + 1],
            b: hist[x * 3 + 2],
          });
        }
      }
    }

    this.historyIndex = (this.historyIndex + 1) % this.maxHistory;
    return output;
  }

  reset(): void {
    for (const buf of this.historyBuffer) {
      buf.fill(0);
    }
    this.historyIndex = 0;
  }
}
