#!/usr/bin/env python3
"""
Generate test signal images for the Device Workbench.

These are standard test patterns used to verify shader behavior:
- Ramps (horizontal/vertical gradients)
- Color bars
- Frequency sweeps
- Solid colors
- Checkerboard
"""

from pathlib import Path
import numpy as np
from PIL import Image

OUTPUT_DIR = Path(__file__).parent / "test_signals"
SIZE = (512, 512)  # Standard test size


def save_image(name: str, data: np.ndarray):
    """Save numpy array as PNG."""
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    # Ensure uint8
    if data.dtype != np.uint8:
        data = (np.clip(data, 0, 1) * 255).astype(np.uint8)

    img = Image.fromarray(data)
    img.save(OUTPUT_DIR / f"{name}.png")
    print(f"  Created: {name}.png")


def generate_ramp_h():
    """Horizontal gradient (black left, white right)."""
    w, h = SIZE
    ramp = np.linspace(0, 1, w)
    data = np.tile(ramp, (h, 1))
    data = np.stack([data, data, data], axis=-1)
    save_image("ramp_h", data)


def generate_ramp_v():
    """Vertical gradient (black top, white bottom)."""
    w, h = SIZE
    ramp = np.linspace(0, 1, h)
    data = np.tile(ramp.reshape(-1, 1), (1, w))
    data = np.stack([data, data, data], axis=-1)
    save_image("ramp_v", data)


def generate_colorbars():
    """SMPTE-style color bars."""
    w, h = SIZE
    colors = [
        [1.0, 1.0, 1.0],  # White
        [1.0, 1.0, 0.0],  # Yellow
        [0.0, 1.0, 1.0],  # Cyan
        [0.0, 1.0, 0.0],  # Green
        [1.0, 0.0, 1.0],  # Magenta
        [1.0, 0.0, 0.0],  # Red
        [0.0, 0.0, 1.0],  # Blue
        [0.0, 0.0, 0.0],  # Black
    ]

    bar_width = w // len(colors)
    data = np.zeros((h, w, 3))

    for i, color in enumerate(colors):
        x_start = i * bar_width
        x_end = (i + 1) * bar_width if i < len(colors) - 1 else w
        data[:, x_start:x_end] = color

    save_image("colorbars", data)


def generate_sweep():
    """Horizontal frequency sweep (sine waves increasing in frequency)."""
    w, h = SIZE
    x = np.linspace(0, 1, w)

    # Exponential frequency sweep from 1 to 64 cycles
    freq = np.exp(np.linspace(np.log(1), np.log(64), w))
    phase = np.cumsum(freq) / w * 2 * np.pi
    wave = (np.sin(phase) + 1) / 2

    data = np.tile(wave, (h, 1))
    data = np.stack([data, data, data], axis=-1)
    save_image("sweep", data)


def generate_gray50():
    """50% gray."""
    data = np.full((*SIZE, 3), 0.5)
    save_image("gray50", data)


def generate_white():
    """Pure white."""
    data = np.ones((*SIZE, 3))
    save_image("white", data)


def generate_black():
    """Pure black."""
    data = np.zeros((*SIZE, 3))
    save_image("black", data)


def generate_checker():
    """Checkerboard pattern."""
    w, h = SIZE
    check_size = 32

    x = np.arange(w) // check_size
    y = np.arange(h) // check_size
    xx, yy = np.meshgrid(x, y)

    data = ((xx + yy) % 2).astype(float)
    data = np.stack([data, data, data], axis=-1)
    save_image("checker", data)


def generate_rgb_ramps():
    """Separate R, G, B ramps (for channel testing)."""
    w, h = SIZE
    ramp = np.linspace(0, 1, w)
    base = np.tile(ramp, (h, 1))

    # Red ramp
    data = np.stack([base, np.zeros_like(base), np.zeros_like(base)], axis=-1)
    save_image("ramp_r", data)

    # Green ramp
    data = np.stack([np.zeros_like(base), base, np.zeros_like(base)], axis=-1)
    save_image("ramp_g", data)

    # Blue ramp
    data = np.stack([np.zeros_like(base), np.zeros_like(base), base], axis=-1)
    save_image("ramp_b", data)


def generate_circle():
    """Circular gradient (for radial effects testing)."""
    w, h = SIZE
    x = np.linspace(-1, 1, w)
    y = np.linspace(-1, 1, h)
    xx, yy = np.meshgrid(x, y)

    dist = np.sqrt(xx**2 + yy**2)
    data = np.clip(1 - dist, 0, 1)
    data = np.stack([data, data, data], axis=-1)
    save_image("circle", data)


def generate_zone_plate():
    """Zone plate (Fresnel pattern) - good for aliasing tests."""
    w, h = SIZE
    x = np.linspace(-1, 1, w)
    y = np.linspace(-1, 1, h)
    xx, yy = np.meshgrid(x, y)

    r2 = xx**2 + yy**2
    # Zone plate with increasing frequency toward edges
    data = (np.sin(r2 * 50) + 1) / 2
    data = np.stack([data, data, data], axis=-1)
    save_image("zone_plate", data)


def main():
    print("Generating test signals...")

    generate_ramp_h()
    generate_ramp_v()
    generate_colorbars()
    generate_sweep()
    generate_gray50()
    generate_white()
    generate_black()
    generate_checker()
    generate_rgb_ramps()
    generate_circle()
    generate_zone_plate()

    print(f"\nAll test signals saved to: {OUTPUT_DIR}")


if __name__ == "__main__":
    main()
