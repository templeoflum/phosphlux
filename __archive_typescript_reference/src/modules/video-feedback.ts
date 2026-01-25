/**
 * Video Feedback System
 *
 * The most powerful technique in analog video synthesis.
 *
 * Optical feedback: Camera pointed at its own monitor
 * Electronic feedback: Signal output routed back to input through processing
 *
 * Creates fractal, self-similar patterns through recursion.
 * Key characteristics:
 * - Scale factor (zoom) from camera-monitor distance
 * - Rotation from angular offset creates spirals
 * - Color drift accumulates through repeated processing
 * - Gain balance determines stability vs chaos
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
 * Video Feedback Processor
 *
 * Implements ping-pong buffer feedback with configurable:
 * - Decay/persistence
 * - Spatial transformation (zoom, rotate, translate)
 * - Color drift
 * - Soft clipping to prevent runaway values
 */
export class VideoFeedback implements Module {
  readonly type = 'video-feedback';
  readonly name = 'Video Feedback';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
    { name: 'zoomCv', type: 'cv', defaultValue: 0 },
    { name: 'rotateCv', type: 'cv', defaultValue: 0 },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'mix', defaultValue: 0.5, min: 0, max: 1, description: 'Feedback mix (0=input only, 1=feedback only)' },
    { name: 'decay', defaultValue: 0.95, min: 0, max: 1, description: 'Feedback decay per frame' },
    { name: 'zoom', defaultValue: 0.98, min: 0.5, max: 1.5, description: 'Zoom factor (<1 = zoom in)' },
    { name: 'rotation', defaultValue: 0, min: -0.1, max: 0.1, description: 'Rotation per frame (radians)' },
    { name: 'translateX', defaultValue: 0, min: -0.1, max: 0.1, description: 'Horizontal translation' },
    { name: 'translateY', defaultValue: 0, min: -0.1, max: 0.1, description: 'Vertical translation' },
    { name: 'hueShift', defaultValue: 0, min: -0.05, max: 0.05, description: 'Hue shift per iteration' },
    { name: 'saturationDecay', defaultValue: 1, min: 0.9, max: 1.1, description: 'Saturation change per iteration' },
    { name: 'softClip', defaultValue: 0.9, min: 0, max: 1, description: 'Soft clipping threshold' },
    { name: 'edgeMode', defaultValue: 0, min: 0, max: 2, step: 1, description: '0=black, 1=wrap, 2=mirror' },
  ];

  // Ping-pong buffers
  private bufferA: FrameBuffer | null = null;
  private bufferB: FrameBuffer | null = null;
  private currentBuffer: 'A' | 'B' = 'A';

  private width: number = 0;
  private height: number = 0;

  // Current position for per-pixel processing
  private currentX: number = 0;
  private currentY: number = 0;

  /**
   * Initialize or resize buffers
   */
  private ensureBuffers(width: number, height: number): void {
    if (this.width !== width || this.height !== height) {
      this.width = width;
      this.height = height;
      this.bufferA = createFrameBuffer(width, height);
      this.bufferB = createFrameBuffer(width, height);
    }
  }

  /**
   * Get the read buffer (previous frame)
   */
  private getReadBuffer(): FrameBuffer {
    return this.currentBuffer === 'A' ? this.bufferB! : this.bufferA!;
  }

  /**
   * Get the write buffer (current frame)
   */
  private getWriteBuffer(): FrameBuffer {
    return this.currentBuffer === 'A' ? this.bufferA! : this.bufferB!;
  }

  /**
   * Swap buffers for next frame
   */
  swapBuffers(): void {
    this.currentBuffer = this.currentBuffer === 'A' ? 'B' : 'A';
  }

  /**
   * Set current pixel position
   */
  setPosition(x: number, y: number): void {
    this.currentX = x;
    this.currentY = y;
  }

  process(
    inputs: Record<string, Signal>,
    params: Record<string, number>
  ): Record<string, Signal> {
    // Per-pixel processing requires frame context
    // Return input for now - use processFrame for full effect
    return {
      r: inputs.r ?? 0,
      g: inputs.g ?? 0,
      b: inputs.b ?? 0,
    };
  }

  /**
   * Process a complete frame with feedback
   */
  processFrame(
    input: FrameBuffer,
    params: Record<string, number>
  ): FrameBuffer {
    this.ensureBuffers(input.width, input.height);

    const mix = params.mix ?? 0.5;
    const decay = params.decay ?? 0.95;
    const zoom = params.zoom ?? 0.98;
    const rotation = params.rotation ?? 0;
    const translateX = params.translateX ?? 0;
    const translateY = params.translateY ?? 0;
    const hueShift = params.hueShift ?? 0;
    const saturationDecay = params.saturationDecay ?? 1;
    const softClip = params.softClip ?? 0.9;
    const edgeMode = Math.round(params.edgeMode ?? 0);

    const readBuffer = this.getReadBuffer();
    const writeBuffer = this.getWriteBuffer();

    // Precompute transformation matrix
    const cos = Math.cos(rotation);
    const sin = Math.sin(rotation);
    const centerX = input.width / 2;
    const centerY = input.height / 2;

    for (let y = 0; y < input.height; y++) {
      for (let x = 0; x < input.width; x++) {
        // Get input pixel
        const inputPixel = getPixel(input, x, y);

        // Calculate feedback sample position
        // Apply inverse transform to find where this pixel came from
        let fx = x - centerX;
        let fy = y - centerY;

        // Apply zoom (inverse = divide)
        fx /= zoom;
        fy /= zoom;

        // Apply rotation (inverse = negative angle)
        const rx = fx * cos + fy * sin;
        const ry = -fx * sin + fy * cos;

        // Apply translation (inverse = negative)
        const sx = rx - translateX * input.width + centerX;
        const sy = ry - translateY * input.height + centerY;

        // Sample from feedback buffer with edge handling
        let feedbackPixel: RGB;

        if (sx < 0 || sx >= input.width || sy < 0 || sy >= input.height) {
          switch (edgeMode) {
            case 0: // Black
              feedbackPixel = { r: 0, g: 0, b: 0 };
              break;
            case 1: // Wrap
              feedbackPixel = this.sampleBilinear(
                readBuffer,
                ((sx % input.width) + input.width) % input.width,
                ((sy % input.height) + input.height) % input.height
              );
              break;
            case 2: // Mirror
              feedbackPixel = this.sampleBilinear(
                readBuffer,
                this.mirror(sx, input.width),
                this.mirror(sy, input.height)
              );
              break;
            default:
              feedbackPixel = { r: 0, g: 0, b: 0 };
          }
        } else {
          feedbackPixel = this.sampleBilinear(readBuffer, sx, sy);
        }

        // Apply decay
        feedbackPixel.r *= decay;
        feedbackPixel.g *= decay;
        feedbackPixel.b *= decay;

        // Apply hue shift
        if (hueShift !== 0) {
          feedbackPixel = this.shiftHue(feedbackPixel, hueShift);
        }

        // Apply saturation change
        if (saturationDecay !== 1) {
          feedbackPixel = this.adjustSaturation(feedbackPixel, saturationDecay);
        }

        // Mix input with feedback
        let r = inputPixel.r * (1 - mix) + feedbackPixel.r * mix;
        let g = inputPixel.g * (1 - mix) + feedbackPixel.g * mix;
        let b = inputPixel.b * (1 - mix) + feedbackPixel.b * mix;

        // Apply soft clipping to prevent runaway
        if (softClip < 1) {
          r = this.softClipValue(r, softClip);
          g = this.softClipValue(g, softClip);
          b = this.softClipValue(b, softClip);
        }

        setPixel(writeBuffer, x, y, {
          r: clamp(r),
          g: clamp(g),
          b: clamp(b),
        });
      }
    }

    // Swap for next frame
    this.swapBuffers();

    return writeBuffer;
  }

  /**
   * Bilinear sampling from frame buffer
   */
  private sampleBilinear(fb: FrameBuffer, x: number, y: number): RGB {
    const x0 = Math.floor(x);
    const y0 = Math.floor(y);
    const x1 = Math.min(x0 + 1, fb.width - 1);
    const y1 = Math.min(y0 + 1, fb.height - 1);

    const fx = x - x0;
    const fy = y - y0;

    const p00 = getPixel(fb, Math.max(0, x0), Math.max(0, y0));
    const p10 = getPixel(fb, x1, Math.max(0, y0));
    const p01 = getPixel(fb, Math.max(0, x0), y1);
    const p11 = getPixel(fb, x1, y1);

    return {
      r: lerp(lerp(p00.r, p10.r, fx), lerp(p01.r, p11.r, fx), fy),
      g: lerp(lerp(p00.g, p10.g, fx), lerp(p01.g, p11.g, fx), fy),
      b: lerp(lerp(p00.b, p10.b, fx), lerp(p01.b, p11.b, fx), fy),
    };
  }

  /**
   * Mirror coordinate at boundaries
   */
  private mirror(coord: number, size: number): number {
    const period = size * 2;
    const normalized = ((coord % period) + period) % period;
    return normalized < size ? normalized : period - normalized - 1;
  }

  /**
   * Shift hue of RGB color
   */
  private shiftHue(color: RGB, shift: number): RGB {
    // Convert to HSV
    const max = Math.max(color.r, color.g, color.b);
    const min = Math.min(color.r, color.g, color.b);
    const delta = max - min;

    let h = 0;
    const s = max === 0 ? 0 : delta / max;
    const v = max;

    if (delta !== 0) {
      if (max === color.r) {
        h = ((color.g - color.b) / delta) % 6;
      } else if (max === color.g) {
        h = (color.b - color.r) / delta + 2;
      } else {
        h = (color.r - color.g) / delta + 4;
      }
      h /= 6;
      if (h < 0) h += 1;
    }

    // Shift hue
    h = (h + shift + 1) % 1;

    // Convert back to RGB
    const i = Math.floor(h * 6);
    const f = h * 6 - i;
    const p = v * (1 - s);
    const q = v * (1 - f * s);
    const t = v * (1 - (1 - f) * s);

    let r: number, g: number, b: number;
    switch (i % 6) {
      case 0: r = v; g = t; b = p; break;
      case 1: r = q; g = v; b = p; break;
      case 2: r = p; g = v; b = t; break;
      case 3: r = p; g = q; b = v; break;
      case 4: r = t; g = p; b = v; break;
      case 5: r = v; g = p; b = q; break;
      default: r = 0; g = 0; b = 0;
    }

    return { r, g, b };
  }

  /**
   * Adjust saturation of RGB color
   */
  private adjustSaturation(color: RGB, factor: number): RGB {
    const luma = 0.299 * color.r + 0.587 * color.g + 0.114 * color.b;
    return {
      r: lerp(luma, color.r, factor),
      g: lerp(luma, color.g, factor),
      b: lerp(luma, color.b, factor),
    };
  }

  /**
   * Soft clip value to prevent runaway
   */
  private softClipValue(x: number, threshold: number): number {
    if (x <= threshold) {
      return x;
    }
    // Soft saturation above threshold
    const excess = x - threshold;
    const range = 1 - threshold;
    return threshold + range * Math.tanh(excess / range);
  }

  /**
   * Get current feedback buffer for display
   */
  getBuffer(): FrameBuffer | null {
    return this.getReadBuffer();
  }

  /**
   * Clear feedback buffers
   */
  clear(): void {
    if (this.bufferA) {
      this.bufferA.data.fill(0);
    }
    if (this.bufferB) {
      this.bufferB.data.fill(0);
    }
  }

  reset(): void {
    this.clear();
  }
}

/**
 * Multi-Tap Feedback
 *
 * Advanced feedback with multiple delay taps at different times.
 * Creates more complex temporal effects than single-frame feedback.
 */
export class MultiTapFeedback implements Module {
  readonly type = 'multi-tap-feedback';
  readonly name = 'Multi-Tap Feedback';

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
    { name: 'tap1Delay', defaultValue: 1, min: 1, max: 30, step: 1, description: 'Tap 1 delay (frames)' },
    { name: 'tap1Level', defaultValue: 0.5, min: 0, max: 1 },
    { name: 'tap2Delay', defaultValue: 3, min: 1, max: 30, step: 1, description: 'Tap 2 delay (frames)' },
    { name: 'tap2Level', defaultValue: 0.3, min: 0, max: 1 },
    { name: 'tap3Delay', defaultValue: 7, min: 1, max: 30, step: 1, description: 'Tap 3 delay (frames)' },
    { name: 'tap3Level', defaultValue: 0.2, min: 0, max: 1 },
    { name: 'inputLevel', defaultValue: 0.7, min: 0, max: 1 },
  ];

  // Ring buffer of frames
  private frameBuffer: FrameBuffer[] = [];
  private writeIndex: number = 0;
  private maxDelay: number = 30;

  private width: number = 0;
  private height: number = 0;

  /**
   * Ensure buffer capacity
   */
  private ensureBuffers(width: number, height: number): void {
    if (this.width !== width || this.height !== height || this.frameBuffer.length === 0) {
      this.width = width;
      this.height = height;
      this.frameBuffer = [];
      for (let i = 0; i < this.maxDelay; i++) {
        this.frameBuffer.push(createFrameBuffer(width, height));
      }
      this.writeIndex = 0;
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
   * Process frame with multi-tap feedback
   */
  processFrame(
    input: FrameBuffer,
    params: Record<string, number>
  ): FrameBuffer {
    this.ensureBuffers(input.width, input.height);

    const tap1Delay = Math.round(params.tap1Delay ?? 1);
    const tap1Level = params.tap1Level ?? 0.5;
    const tap2Delay = Math.round(params.tap2Delay ?? 3);
    const tap2Level = params.tap2Level ?? 0.3;
    const tap3Delay = Math.round(params.tap3Delay ?? 7);
    const tap3Level = params.tap3Level ?? 0.2;
    const inputLevel = params.inputLevel ?? 0.7;

    // Get delayed frames
    const tap1Frame = this.getDelayedFrame(tap1Delay);
    const tap2Frame = this.getDelayedFrame(tap2Delay);
    const tap3Frame = this.getDelayedFrame(tap3Delay);

    // Create output by mixing input with taps
    const output = createFrameBuffer(input.width, input.height);

    for (let y = 0; y < input.height; y++) {
      for (let x = 0; x < input.width; x++) {
        const inp = getPixel(input, x, y);
        const t1 = tap1Frame ? getPixel(tap1Frame, x, y) : { r: 0, g: 0, b: 0 };
        const t2 = tap2Frame ? getPixel(tap2Frame, x, y) : { r: 0, g: 0, b: 0 };
        const t3 = tap3Frame ? getPixel(tap3Frame, x, y) : { r: 0, g: 0, b: 0 };

        setPixel(output, x, y, {
          r: clamp(inp.r * inputLevel + t1.r * tap1Level + t2.r * tap2Level + t3.r * tap3Level),
          g: clamp(inp.g * inputLevel + t1.g * tap1Level + t2.g * tap2Level + t3.g * tap3Level),
          b: clamp(inp.b * inputLevel + t1.b * tap1Level + t2.b * tap2Level + t3.b * tap3Level),
        });
      }
    }

    // Store current output in ring buffer
    const writeBuffer = this.frameBuffer[this.writeIndex];
    writeBuffer.data.set(output.data);
    this.writeIndex = (this.writeIndex + 1) % this.maxDelay;

    return output;
  }

  /**
   * Get frame at specified delay
   */
  private getDelayedFrame(delay: number): FrameBuffer | null {
    if (this.frameBuffer.length === 0) return null;
    delay = Math.min(delay, this.maxDelay - 1);
    const index = (this.writeIndex - delay + this.maxDelay) % this.maxDelay;
    return this.frameBuffer[index];
  }

  reset(): void {
    for (const fb of this.frameBuffer) {
      fb.data.fill(0);
    }
    this.writeIndex = 0;
  }
}

/**
 * Frame Delay / Frame Store
 *
 * Simple frame delay line for temporal effects.
 * Can be used to create echo, freeze frame, or time displacement.
 */
export class FrameDelay implements Module {
  readonly type = 'frame-delay';
  readonly name = 'Frame Delay';

  readonly inputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
    { name: 'freeze', type: 'gate', defaultValue: 0, description: 'Freeze frame when high' },
  ];

  readonly outputs: ReadonlyArray<Port> = [
    { name: 'r', type: 'signal', defaultValue: 0 },
    { name: 'g', type: 'signal', defaultValue: 0 },
    { name: 'b', type: 'signal', defaultValue: 0 },
  ];

  readonly parameters: ReadonlyArray<Parameter> = [
    { name: 'delay', defaultValue: 1, min: 0, max: 60, step: 1, description: 'Delay in frames' },
    { name: 'feedback', defaultValue: 0, min: 0, max: 0.99, description: 'Feedback amount' },
  ];

  private buffer: FrameBuffer[] = [];
  private writeIndex: number = 0;
  private maxDelay: number = 60;
  private frozen: boolean = false;
  private frozenFrame: FrameBuffer | null = null;

  private width: number = 0;
  private height: number = 0;

  private ensureBuffers(width: number, height: number): void {
    if (this.width !== width || this.height !== height) {
      this.width = width;
      this.height = height;
      this.buffer = [];
      for (let i = 0; i < this.maxDelay; i++) {
        this.buffer.push(createFrameBuffer(width, height));
      }
      this.frozenFrame = createFrameBuffer(width, height);
      this.writeIndex = 0;
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

  processFrame(
    input: FrameBuffer,
    params: Record<string, number>,
    freeze: boolean = false
  ): FrameBuffer {
    this.ensureBuffers(input.width, input.height);

    const delay = Math.round(params.delay ?? 1);
    const feedback = params.feedback ?? 0;

    // Handle freeze
    if (freeze && !this.frozen && this.frozenFrame) {
      this.frozenFrame.data.set(input.data);
      this.frozen = true;
    } else if (!freeze) {
      this.frozen = false;
    }

    if (this.frozen && this.frozenFrame) {
      return this.frozenFrame;
    }

    // Get delayed frame
    const delayedIndex = (this.writeIndex - delay + this.maxDelay) % this.maxDelay;
    const delayed = this.buffer[delayedIndex];

    // Create output (with optional feedback)
    const output = createFrameBuffer(input.width, input.height);

    if (feedback > 0) {
      for (let y = 0; y < input.height; y++) {
        for (let x = 0; x < input.width; x++) {
          const inp = getPixel(input, x, y);
          const del = getPixel(delayed, x, y);
          setPixel(output, x, y, {
            r: clamp(inp.r + del.r * feedback),
            g: clamp(inp.g + del.g * feedback),
            b: clamp(inp.b + del.b * feedback),
          });
        }
      }
    } else {
      output.data.set(delayed.data);
    }

    // Store input in buffer
    this.buffer[this.writeIndex].data.set(input.data);
    this.writeIndex = (this.writeIndex + 1) % this.maxDelay;

    return delay === 0 ? input : output;
  }

  reset(): void {
    for (const fb of this.buffer) {
      fb.data.fill(0);
    }
    this.writeIndex = 0;
    this.frozen = false;
  }
}
