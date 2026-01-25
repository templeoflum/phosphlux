/**
 * Analog Video Synthesizer Demo
 *
 * Interactive demonstration of the video synthesis engine.
 */

import {
  VideoSynthesizer,
  type FrameBuffer,
} from '../src/index.js';

// Detect mobile for lower resolution
const isMobile = /iPhone|iPad|iPod|Android/i.test(navigator.userAgent);
const WIDTH = isMobile ? 180 : 720;
const HEIGHT = isMobile ? 120 : 480;

// Canvas and context
const canvas = document.getElementById('display') as HTMLCanvasElement;
canvas.width = WIDTH;
canvas.height = HEIGHT;
const ctx = canvas.getContext('2d')!;

// UI elements
const presetSelect = document.getElementById('preset') as HTMLSelectElement;
const speedSlider = document.getElementById('speed') as HTMLInputElement;
const scanlinesSlider = document.getElementById('scanlines') as HTMLInputElement;
const bloomSlider = document.getElementById('bloom') as HTMLInputElement;
const curvatureSlider = document.getElementById('curvature') as HTMLInputElement;
const persistenceSlider = document.getElementById('persistence') as HTMLInputElement;
const hueSlider = document.getElementById('hue') as HTMLInputElement;
const saturationSlider = document.getElementById('saturation') as HTMLInputElement;

const resetBtn = document.getElementById('reset') as HTMLButtonElement;
const pauseBtn = document.getElementById('pause') as HTMLButtonElement;
const fullscreenBtn = document.getElementById('fullscreen') as HTMLButtonElement;

const fpsDisplay = document.getElementById('fps')!;
const frameDisplay = document.getElementById('frame')!;

// Value displays
const speedValue = document.getElementById('speedValue')!;
const scanlinesValue = document.getElementById('scanlinesValue')!;
const bloomValue = document.getElementById('bloomValue')!;
const curvatureValue = document.getElementById('curvatureValue')!;
const persistenceValue = document.getElementById('persistenceValue')!;
const hueValue = document.getElementById('hueValue')!;
const saturationValue = document.getElementById('saturationValue')!;

// Synthesizer
let synth: VideoSynthesizer;
let imageData: ImageData;
let paused = false;
let frameCount = 0;
let lastTime = performance.now();
let fps = 0;

// Initialize
function init() {
  try {
    synth = new VideoSynthesizer({
      width: WIDTH,
      height: HEIGHT,
      crtEmulation: !isMobile, // Disable heavy CRT effects on mobile
    });

    imageData = ctx.createImageData(WIDTH, HEIGHT);

    loadPreset('test');

    // Start render loop
    requestAnimationFrame(render);

    console.log(`Initialized: ${WIDTH}x${HEIGHT}, mobile: ${isMobile}`);
  } catch (e) {
    console.error('Init error:', e);
    ctx.fillStyle = 'red';
    ctx.font = '16px monospace';
    ctx.fillText('Error: ' + String(e), 10, 30);
  }
}

// Load a preset patch
function loadPreset(name: string) {
  try {
    synth.reset();

    // Clear existing modules
    const modules = synth.listModules();
    for (const m of modules) {
      synth.removeModule(m.id);
    }

    switch (name) {
      case 'test':
        // Simple test pattern with radial gradient
        synth.addModule('ramp', 'ramp-gen');
        synth.addModule('colorizer', 'gradient-mapper', {
          preset: 2, // spectrum
        });
        synth.connect('ramp', 'radial', 'colorizer', 'in');
        break;

      case 'ruttEtra':
        // Rutt/Etra style displacement
        synth.addModule('osc', 'video-osc', {
          frequency: 3,
          waveform: 0,
        });
        synth.addModule('colorizer', 'threshold-colorizer', {
          levels: 6,
          saturation: 0.8,
        });
        if (!isMobile) {
          synth.addModule('rutt', 'rutt-etra', {
            zDepth: 0.2,
            rotationX: 0.25,
            scanlineGap: 0.4,
          });
        }
        break;

      case 'feedback':
        // Video feedback pattern
        synth.addModule('ramp', 'ramp-gen');
        if (!isMobile) {
          synth.addModule('shape', 'shape-gen', {
            x: 0.5,
            y: 0.5,
            width: 0.3,
            height: 0.3,
          });
        }
        synth.addModule('colorizer', 'gradient-mapper', {
          preset: 4, // electric
        });
        if (!isMobile) {
          synth.addModule('feedback', 'video-feedback', {
            mix: 0.7,
            decay: 0.98,
            zoom: 0.97,
            rotation: 0.02,
            hueShift: 0.01,
          });
        }
        break;

      case 'colorBars':
        // Classic color bars
        synth.addModule('ramp', 'ramp-gen');
        synth.addModule('quantizer', 'quantizer', {
          levels: 8,
          mode: 0,
        });
        synth.addModule('colorizer', 'threshold-colorizer', {
          levels: 8,
          hueRange: 0.875,
          saturation: 0.9,
          brightness: 0.9,
        });
        synth.connect('ramp', 'hEdge', 'quantizer', 'in');
        synth.connect('quantizer', 'out', 'colorizer', 'in');
        break;

      case 'oscillator':
        // Multi-oscillator pattern
        synth.addModule('osc1', 'video-osc', {
          frequency: 5,
          waveform: 0,
          orientation: 0,
        });
        if (!isMobile) {
          synth.addModule('osc2', 'video-osc', {
            frequency: 4,
            waveform: 0,
            orientation: 1,
          });
          synth.addModule('mixer', 'video-mixer', {
            mix: 0.5,
            mode: 2, // multiply
          });
        }
        synth.addModule('colorizer', 'layered-colorizer', {
          ch1_clip: 0.3,
          ch1_r: 0.8, ch1_g: 0.2, ch1_b: 0.2,
          ch2_clip: 0.5,
          ch2_r: 0.2, ch2_g: 0.8, ch2_b: 0.2,
          ch3_clip: 0.7,
          ch3_r: 0.2, ch3_g: 0.2, ch3_b: 0.8,
        });
        break;
    }
  } catch (e) {
    console.error('Preset error:', e);
  }
}

// Convert frame buffer to ImageData
function frameBufferToImageData(fb: FrameBuffer, imgData: ImageData): void {
  const pixels = imgData.data;
  let j = 0;

  for (let i = 0; i < fb.data.length; i += 3) {
    pixels[j++] = Math.floor(fb.data[i] * 255);     // R
    pixels[j++] = Math.floor(fb.data[i + 1] * 255); // G
    pixels[j++] = Math.floor(fb.data[i + 2] * 255); // B
    pixels[j++] = 255;                               // A
  }
}

// Main render loop
function render(timestamp: number) {
  // Calculate FPS
  frameCount++;
  if (timestamp - lastTime >= 1000) {
    fps = frameCount;
    frameCount = 0;
    lastTime = timestamp;
    fpsDisplay.textContent = `FPS: ${fps}`;
  }

  if (!paused) {
    try {
      // Process frame
      const buffer = synth.processFrame();

      // Convert to canvas
      frameBufferToImageData(buffer, imageData);
      ctx.putImageData(imageData, 0, 0);

      frameDisplay.textContent = `Frame: ${synth.getFrameCount()}`;
    } catch (e) {
      console.error('Render error:', e);
    }
  }

  requestAnimationFrame(render);
}

// Event listeners
presetSelect?.addEventListener('change', () => {
  loadPreset(presetSelect.value);
});

// Slider event handlers
function setupSlider(
  slider: HTMLInputElement | null,
  valueDisplay: HTMLElement | null,
  callback?: (value: number) => void
) {
  if (!slider || !valueDisplay) return;

  const update = () => {
    const value = parseFloat(slider.value);
    valueDisplay.textContent = value.toFixed(1);
    callback?.(value);
  };

  slider.addEventListener('input', update);
  update();
}

setupSlider(speedSlider, speedValue);
setupSlider(scanlinesSlider, scanlinesValue);
setupSlider(bloomSlider, bloomValue);
setupSlider(curvatureSlider, curvatureValue);
setupSlider(persistenceSlider, persistenceValue);
setupSlider(hueSlider, hueValue);
setupSlider(saturationSlider, saturationValue);

resetBtn?.addEventListener('click', () => {
  synth.reset();
  loadPreset(presetSelect?.value || 'test');
});

pauseBtn?.addEventListener('click', () => {
  paused = !paused;
  pauseBtn.textContent = paused ? 'Play' : 'Pause';
});

fullscreenBtn?.addEventListener('click', () => {
  if (document.fullscreenElement) {
    document.exitFullscreen();
  } else {
    canvas.parentElement?.requestFullscreen();
  }
});

// Start
init();
