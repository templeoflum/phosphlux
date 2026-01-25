#!/usr/bin/env npx ts-node
/**
 * Visual Capture Tool
 *
 * Generates visual output from the video synthesis system and displays it.
 * Outputs can be viewed as ASCII art in the terminal or saved as PPM images.
 *
 * Usage:
 *   npx ts-node scripts/visual-capture.ts [preset] [--frames N] [--output file.ppm]
 */

import { VideoSynthesizer } from '../src/synth/synthesizer';
import { createFrameBuffer, getPixel, clamp, type FrameBuffer } from '../src/core/types';
import * as fs from 'fs';

// ASCII art characters from dark to bright
const ASCII_CHARS = ' .\'`^",:;Il!i><~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$';

/**
 * Convert a frame to ASCII art
 */
function frameToAscii(fb: FrameBuffer, targetWidth: number = 80): string {
  const aspectRatio = fb.width / fb.height;
  const scale = fb.width / targetWidth;
  const targetHeight = Math.floor(targetWidth / aspectRatio / 2.2); // Adjust for character aspect ratio

  let ascii = '';
  ascii += `┌${'─'.repeat(targetWidth)}┐\n`;

  for (let y = 0; y < targetHeight; y++) {
    ascii += '│';
    for (let x = 0; x < targetWidth; x++) {
      const srcX = Math.floor((x / targetWidth) * fb.width);
      const srcY = Math.floor((y / targetHeight) * fb.height);

      if (srcX < fb.width && srcY < fb.height) {
        const pixel = getPixel(fb, srcX, srcY);
        const luma = 0.299 * pixel.r + 0.587 * pixel.g + 0.114 * pixel.b;
        const charIdx = Math.floor(clamp(luma) * (ASCII_CHARS.length - 1));
        ascii += ASCII_CHARS[charIdx];
      } else {
        ascii += ' ';
      }
    }
    ascii += '│\n';
  }

  ascii += `└${'─'.repeat(targetWidth)}┘\n`;
  return ascii;
}

/**
 * Convert frame to PPM format
 */
function frameToPPM(fb: FrameBuffer): Buffer {
  const header = `P6\n${fb.width} ${fb.height}\n255\n`;
  const headerBuf = Buffer.from(header, 'ascii');
  const pixelBuf = Buffer.alloc(fb.width * fb.height * 3);

  for (let i = 0; i < fb.data.length; i += 3) {
    const idx = (i / 3) * 3;
    pixelBuf[idx] = Math.floor(clamp(fb.data[i]) * 255);
    pixelBuf[idx + 1] = Math.floor(clamp(fb.data[i + 1]) * 255);
    pixelBuf[idx + 2] = Math.floor(clamp(fb.data[i + 2]) * 255);
  }

  return Buffer.concat([headerBuf, pixelBuf]);
}

/**
 * Calculate frame statistics
 */
function analyzeFrame(fb: FrameBuffer): { minLuma: number; maxLuma: number; avgLuma: number } {
  let minLuma = 1, maxLuma = 0, sumLuma = 0;
  const pixels = fb.width * fb.height;

  for (let i = 0; i < fb.data.length; i += 3) {
    const luma = 0.299 * fb.data[i] + 0.587 * fb.data[i + 1] + 0.114 * fb.data[i + 2];
    minLuma = Math.min(minLuma, luma);
    maxLuma = Math.max(maxLuma, luma);
    sumLuma += luma;
  }

  return { minLuma, maxLuma, avgLuma: sumLuma / pixels };
}

/**
 * Available presets
 */
const PRESETS: Record<string, (synth: VideoSynthesizer) => void> = {
  'gradient': (synth) => {
    synth.addModule('ramp', 'ramp-gen');
    synth.addModule('colorizer', 'gradient-mapper', { preset: 2 });
    synth.connect('ramp', 'radial', 'colorizer', 'in');
  },

  'bars': (synth) => {
    synth.addModule('ramp', 'ramp-gen');
    synth.addModule('quantizer', 'quantizer', { levels: 8, mode: 1 });
    synth.addModule('colorizer', 'threshold-colorizer', { levels: 8, hueRange: 0.875 });
    synth.connect('ramp', 'hEdge', 'quantizer', 'in');
    synth.connect('quantizer', 'out', 'colorizer', 'in');
  },

  'oscillator': (synth) => {
    synth.addModule('ramp', 'ramp-gen');
    synth.addModule('osc', 'video-osc', { frequency: 8, waveform: 0 });
    synth.addModule('colorizer', 'gradient-mapper', { preset: 4 });
    // Oscillator uses position from ramp internally
    synth.connect('osc', 'out', 'colorizer', 'in');
  },

  'folded': (synth) => {
    synth.addModule('ramp', 'ramp-gen');
    synth.addModule('folder', 'folder', { gain: 4, folds: 3 });
    synth.addModule('colorizer', 'gradient-mapper', { preset: 1 });
    synth.connect('ramp', 'hEdge', 'folder', 'in');
    synth.connect('folder', 'out', 'colorizer', 'in');
  },

  'pattern': (synth) => {
    synth.addModule('osc1', 'video-osc', { frequency: 4, waveform: 0, orientation: 0 });
    synth.addModule('osc2', 'video-osc', { frequency: 3, waveform: 0, orientation: 1 });
    synth.addModule('mixer', 'video-mixer', { mix: 0.5, mode: 2 }); // multiply
    synth.addModule('colorizer', 'threshold-colorizer', { levels: 6, hueRange: 0.6 });
    // Note: mixer connection would need per-pixel processing
  },

  'shapes': (synth) => {
    synth.addModule('ramp', 'ramp-gen');
    synth.addModule('shape', 'shape-gen', { x: 0.5, y: 0.5, width: 0.4, height: 0.4 });
    synth.addModule('colorizer', 'gradient-mapper', { preset: 3 });
    // Would need proper connection
  },
};

/**
 * Main function
 */
async function main() {
  const args = process.argv.slice(2);

  // Parse arguments
  let preset = 'gradient';
  let frames = 1;
  let outputFile: string | null = null;
  let width = 64;
  let height = 48;
  let showAscii = true;

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--frames' && args[i + 1]) {
      frames = parseInt(args[i + 1], 10);
      i++;
    } else if (args[i] === '--output' && args[i + 1]) {
      outputFile = args[i + 1];
      i++;
    } else if (args[i] === '--width' && args[i + 1]) {
      width = parseInt(args[i + 1], 10);
      i++;
    } else if (args[i] === '--height' && args[i + 1]) {
      height = parseInt(args[i + 1], 10);
      i++;
    } else if (args[i] === '--no-ascii') {
      showAscii = false;
    } else if (!args[i].startsWith('-')) {
      preset = args[i];
    }
  }

  // Create synthesizer
  console.log(`\n🎬 Video Synthesizer Visual Capture\n`);
  console.log(`Resolution: ${width}x${height}`);
  console.log(`Preset: ${preset}`);
  console.log(`Frames: ${frames}\n`);

  const synth = new VideoSynthesizer({
    width,
    height,
    crtEmulation: false, // Disable for cleaner test patterns
  });

  // Apply preset
  const presetFn = PRESETS[preset];
  if (presetFn) {
    presetFn(synth);
  } else {
    console.log(`Available presets: ${Object.keys(PRESETS).join(', ')}`);
    process.exit(1);
  }

  // Generate frames
  for (let f = 0; f < frames; f++) {
    const frame = synth.processFrame();
    const stats = analyzeFrame(frame);

    console.log(`\n── Frame ${f + 1}/${frames} ──`);
    console.log(`Luminance: min=${stats.minLuma.toFixed(3)}, max=${stats.maxLuma.toFixed(3)}, avg=${stats.avgLuma.toFixed(3)}`);

    if (showAscii) {
      console.log(frameToAscii(frame, 70));
    }

    // Save last frame if output specified
    if (outputFile && f === frames - 1) {
      const ppmData = frameToPPM(frame);
      fs.writeFileSync(outputFile, ppmData);
      console.log(`Saved: ${outputFile}`);
    }
  }

  console.log('\n✓ Visual capture complete\n');
}

main().catch(console.error);
