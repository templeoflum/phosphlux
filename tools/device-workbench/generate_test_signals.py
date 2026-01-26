#!/usr/bin/env python3
"""
Video Synthesis Test Pattern Generator

Generates diagnostic test patterns specifically designed for testing
analog video synthesis circuit emulations. Unlike standard SMPTE/EBU
patterns designed for display calibration, these patterns reveal
TRANSFER FUNCTION behavior of circuits like:

- Folding amplifiers (where does the fold occur?)
- Comparators/keyers (what's the threshold voltage?)
- Soft keyers (what's the transition width?)
- Colorizers (what voltages map to what colors?)
- Waveshapers (what's the input->output curve?)

Resolution: 640x480 (NTSC square-pixel equivalent)
- 480 visible lines matches NTSC
- 4:3 aspect ratio (authentic to the era)
- Square pixels for clean digital work
- Integer scaling: 2x=1280x960, 3x=1920x1440

Each pattern is designed with intentional voltage/luminance values
that correspond to meaningful circuit behaviors.
"""

import numpy as np
from PIL import Image
from pathlib import Path

# Output resolution - NTSC square pixel equivalent
WIDTH = 640
HEIGHT = 480

# Output directory
OUTPUT_DIR = Path(__file__).parent / "test_signals"


def save_pattern(name: str, data: np.ndarray, description: str = ""):
    """Save a test pattern as PNG with metadata."""
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    # Clip to valid range and convert to 8-bit
    data_clipped = np.clip(data, 0, 1)
    data_8bit = (data_clipped * 255).astype(np.uint8)

    # Create RGB image (grayscale patterns replicated to RGB)
    if len(data_8bit.shape) == 2:
        rgb = np.stack([data_8bit, data_8bit, data_8bit], axis=-1)
    else:
        rgb = data_8bit

    img = Image.fromarray(rgb, mode='RGB')

    output_path = OUTPUT_DIR / f"{name}.png"
    img.save(output_path)
    print(f"  {name}.png - {description}")

    return output_path


# =============================================================================
# BASIC RAMPS - Reveal transfer function shape
# =============================================================================

def generate_ramp_h():
    """
    Horizontal ramp: 0.0 (left) -> 1.0 (right)

    REVEALS:
    - Full transfer function shape
    - Where folding amplifiers fold
    - Clipping behavior at extremes
    - Linearity of gain stages
    - Colorizer color mapping across voltage range

    Each column represents a specific input voltage:
    - Column 0 = 0.0V (black)
    - Column 320 = 0.5V (mid-gray)
    - Column 639 = 1.0V (white)
    """
    x = np.linspace(0, 1, WIDTH)
    data = np.tile(x, (HEIGHT, 1))
    return save_pattern("ramp_h", data,
        "Horizontal ramp 0->1, reveals transfer function")


def generate_ramp_v():
    """
    Vertical ramp: 0.0 (top) -> 1.0 (bottom)

    REVEALS:
    - Same as horizontal but orthogonal
    - Verifies no directional artifacts
    - Useful for testing vertical scan effects
    """
    y = np.linspace(0, 1, HEIGHT)
    data = np.tile(y.reshape(-1, 1), (1, WIDTH))
    return save_pattern("ramp_v", data,
        "Vertical ramp 0->1, orthogonal transfer test")


def generate_ramp_overscan():
    """
    Extended ramp: -0.2 -> 1.2 (clamped to display but conceptually extended)

    REVEALS:
    - How circuits handle out-of-range voltages
    - Super-black and super-white behavior
    - Folding amplifier extended range
    """
    x = np.linspace(-0.2, 1.2, WIDTH)
    data = np.tile(x, (HEIGHT, 1))
    return save_pattern("ramp_overscan", data,
        "Extended ramp -0.2->1.2, tests out-of-range")


# =============================================================================
# STEPPED PATTERNS - Reveal threshold behavior
# =============================================================================

def generate_gray10():
    """
    10-step grayscale: 0.0, 0.111, 0.222, ..., 1.0

    REVEALS:
    - Discrete threshold trigger points
    - Quantization in colorizers
    - Step response of keyers
    """
    steps = 10
    bar_width = WIDTH // steps
    data = np.zeros((HEIGHT, WIDTH))

    for i in range(steps):
        value = i / (steps - 1)
        data[:, i * bar_width:(i + 1) * bar_width] = value

    return save_pattern("gray10", data,
        "10-step grayscale, threshold triggers")


def generate_gray50():
    """50% gray field (0.5 everywhere) - tests midpoint behavior"""
    data = np.full((HEIGHT, WIDTH), 0.5)
    return save_pattern("gray50", data,
        "Uniform 50% gray, midpoint behavior")


def generate_threshold_finder():
    """
    5 precise voltage zones: 0.0, 0.25, 0.5, 0.75, 1.0

    REVEALS:
    - Exact threshold calibration points
    - Quarter-voltage response
    - Common comparator threshold values
    """
    data = np.zeros((HEIGHT, WIDTH))
    band_height = HEIGHT // 5
    thresholds = [0.0, 0.25, 0.5, 0.75, 1.0]

    for i, thresh in enumerate(thresholds):
        data[i * band_height:(i + 1) * band_height, :] = thresh

    return save_pattern("threshold_finder", data,
        "5 zones (0, 0.25, 0.5, 0.75, 1.0)")


def generate_threshold_sweep():
    """
    16-level threshold sweep (4-bit resolution)
    Matches common comparator/DAC resolution
    """
    steps = 16
    bar_width = WIDTH // steps
    data = np.zeros((HEIGHT, WIDTH))

    for i in range(steps):
        value = i / (steps - 1)
        data[:, i * bar_width:(i + 1) * bar_width] = value

    return save_pattern("threshold_sweep", data,
        "16-step sweep (4-bit resolution)")


# =============================================================================
# EDGE PATTERNS - Reveal transition behavior
# =============================================================================

def generate_edge_sharp():
    """
    Sharp 0->1 edge at center

    REVEALS:
    - Ringing and overshoot
    - Soft keyer transition width
    - Edge enhancement artifacts
    """
    data = np.zeros((HEIGHT, WIDTH))
    data[:, WIDTH // 2:] = 1.0
    return save_pattern("edge_sharp", data,
        "Sharp center edge, ringing/overshoot")


def generate_edges_multi():
    """
    Multiple sharp edges at 20%, 40%, 60%, 80%
    Tests edge response consistency
    """
    data = np.zeros((HEIGHT, WIDTH))
    edges = [0.2, 0.4, 0.6, 0.8]
    current_value = 0.0
    prev_x = 0

    for edge in edges:
        x = int(edge * WIDTH)
        data[:, prev_x:x] = current_value
        current_value = 1.0 - current_value
        prev_x = x
    data[:, prev_x:] = current_value

    return save_pattern("edges_multi", data,
        "Multiple edges, consistency test")


def generate_checker():
    """
    8x8 checkerboard - high frequency response test
    """
    square_size = 8
    data = np.zeros((HEIGHT, WIDTH))

    for y in range(HEIGHT):
        for x in range(WIDTH):
            if ((x // square_size) + (y // square_size)) % 2:
                data[y, x] = 1.0

    return save_pattern("checker", data,
        "8x8 checkerboard, high-frequency")


# =============================================================================
# FREQUENCY PATTERNS - Reveal bandwidth and aliasing
# =============================================================================

def generate_zone_plate():
    """
    Zone plate: circular frequency sweep

    REVEALS:
    - Frequency-dependent response
    - Aliasing at high frequencies
    - Moire patterns
    """
    y, x = np.ogrid[:HEIGHT, :WIDTH]
    cx, cy = WIDTH // 2, HEIGHT // 2
    r = np.sqrt((x - cx) ** 2 + (y - cy) ** 2)

    scale = 0.08
    data = (np.sin(r ** 2 * scale) + 1) / 2

    return save_pattern("zone_plate", data,
        "Zone plate, frequency/aliasing")


def generate_sweep():
    """
    Horizontal frequency sweep (low->high left to right)
    Continuous multiburst equivalent
    """
    x = np.linspace(0, 1, WIDTH)
    freq = 1 + x * 50
    phase = np.cumsum(freq) * 2 * np.pi / WIDTH

    y_pattern = (np.sin(phase) + 1) / 2
    data = np.tile(y_pattern, (HEIGHT, 1))

    return save_pattern("sweep", data,
        "Frequency sweep, bandwidth limits")


def generate_multiburst():
    """
    Multiburst: 6 discrete frequency bands
    Standard video bandwidth test
    """
    data = np.zeros((HEIGHT, WIDTH))
    band_width = WIDTH // 6
    frequencies = [2, 4, 8, 16, 32, 64]

    for i, freq in enumerate(frequencies):
        x_start = i * band_width
        x_end = (i + 1) * band_width
        x = np.linspace(0, freq * 2 * np.pi, band_width)
        pattern = (np.sin(x) + 1) / 2
        data[:, x_start:x_end] = pattern

    return save_pattern("multiburst", data,
        "6-band multiburst, discrete frequencies")


# =============================================================================
# SPECIALIZED VIDEO SYNTHESIS PATTERNS
# =============================================================================

def generate_colorizer_bars():
    """
    8 colorizer threshold bars (7 equal divisions = rainbow spectrum)
    Matches typical Paik-style colorizer threshold layout
    """
    steps = 8
    bar_width = WIDTH // steps
    data = np.zeros((HEIGHT, WIDTH))

    for i in range(steps):
        value = i / (steps - 1)
        data[:, i * bar_width:(i + 1) * bar_width] = value

    return save_pattern("colorizer_bars", data,
        "8 colorizer thresholds (rainbow)")


def generate_folding_test():
    """
    Folding amplifier test: ramp with markers at typical fold points
    Markers at 0.25, 0.5, 0.75 where folds typically occur
    """
    x = np.linspace(0, 1, WIDTH)
    data = np.tile(x, (HEIGHT, 1))

    # Add markers at common fold points
    fold_points = [0.25, 0.5, 0.75]
    marker_height = HEIGHT // 10

    for fp in fold_points:
        col = int(fp * (WIDTH - 1))
        # White marker line at top
        data[:marker_height, max(0, col-1):min(WIDTH, col+2)] = 1.0
        # Black marker line at bottom
        data[-marker_height:, max(0, col-1):min(WIDTH, col+2)] = 0.0

    return save_pattern("folding_test", data,
        "Ramp + fold markers (0.25/0.5/0.75)")


def generate_keyer_test():
    """
    Keyer test: vertical gradient with threshold markers
    Markers at 0.1, 0.3, 0.5, 0.7, 0.9 for key threshold testing
    """
    y = np.linspace(0, 1, HEIGHT)
    data = np.tile(y.reshape(-1, 1), (1, WIDTH))

    # Add marker lines at typical keyer thresholds
    key_levels = [0.1, 0.3, 0.5, 0.7, 0.9]
    marker_width = 30

    for kl in key_levels:
        row = int(kl * (HEIGHT - 1))
        # Alternating checkerboard markers on left edge
        for x in range(0, marker_width, 6):
            val = 1.0 if (x // 6) % 2 else 0.0
            data[max(0, row-2):min(HEIGHT, row+3), x:x+3] = val

    return save_pattern("keyer_test", data,
        "Gradient + keyer threshold markers")


def generate_feedback_test():
    """
    Feedback/recursion test: concentric rectangles
    Tests how feedback loops and scaling behave
    """
    data = np.zeros((HEIGHT, WIDTH))
    levels = [0.9, 0.7, 0.5, 0.3, 0.1]
    values = [0.2, 0.4, 0.6, 0.8, 1.0]

    for level, value in zip(levels, values):
        x_margin = int((1 - level) * WIDTH / 2)
        y_margin = int((1 - level) * HEIGHT / 2)
        data[y_margin:HEIGHT-y_margin, x_margin:WIDTH-x_margin] = value

    return save_pattern("feedback_test", data,
        "Concentric rectangles, feedback test")


def generate_comparator_test():
    """
    Comparator precision test: gradient with fine threshold markers
    Shows exact voltage levels for comparator calibration
    """
    x = np.linspace(0, 1, WIDTH)
    data = np.tile(x, (HEIGHT, 1))

    # Add fine grid lines every 10%
    for i in range(1, 10):
        col = int(i * 0.1 * (WIDTH - 1))
        # Thin alternating markers
        for y in range(0, HEIGHT, 20):
            val = 1.0 if (y // 20) % 2 else 0.0
            data[y:y+10, max(0, col-1):col+1] = val

    return save_pattern("comparator_test", data,
        "Gradient + 10% grid markers")


# =============================================================================
# REFERENCE FIELDS
# =============================================================================

def generate_black():
    """Pure black (0.0) - reference black level"""
    data = np.zeros((HEIGHT, WIDTH))
    return save_pattern("black", data, "Pure black (0.0)")


def generate_white():
    """Pure white (1.0) - reference white level"""
    data = np.ones((HEIGHT, WIDTH))
    return save_pattern("white", data, "Pure white (1.0)")


# =============================================================================
# COLOR PATTERNS
# =============================================================================

def generate_colorbars():
    """
    SMPTE-style color bars
    Bars: White, Yellow, Cyan, Green, Magenta, Red, Blue, Black
    Luminance: 1.0, 0.89, 0.70, 0.59, 0.41, 0.30, 0.11, 0.0
    """
    bar_width = WIDTH // 8

    colors = [
        (1.0, 1.0, 1.0),  # White
        (1.0, 1.0, 0.0),  # Yellow
        (0.0, 1.0, 1.0),  # Cyan
        (0.0, 1.0, 0.0),  # Green
        (1.0, 0.0, 1.0),  # Magenta
        (1.0, 0.0, 0.0),  # Red
        (0.0, 0.0, 1.0),  # Blue
        (0.0, 0.0, 0.0),  # Black
    ]

    data = np.zeros((HEIGHT, WIDTH, 3))
    for i, color in enumerate(colors):
        data[:, i * bar_width:(i + 1) * bar_width] = color

    return save_pattern("colorbars", data,
        "SMPTE color bars")


def generate_rgb_ramps():
    """Separate R, G, B channel ramps"""
    ramp = np.linspace(0, 1, WIDTH)
    base = np.tile(ramp, (HEIGHT, 1))

    # Red
    data = np.stack([base, np.zeros_like(base), np.zeros_like(base)], axis=-1)
    save_pattern("ramp_r", data, "Red channel ramp")

    # Green
    data = np.stack([np.zeros_like(base), base, np.zeros_like(base)], axis=-1)
    save_pattern("ramp_g", data, "Green channel ramp")

    # Blue
    data = np.stack([np.zeros_like(base), np.zeros_like(base), base], axis=-1)
    save_pattern("ramp_b", data, "Blue channel ramp")


# =============================================================================
# COMPOSITE DIAGNOSTIC PATTERN
# =============================================================================

def generate_synthesis_diagnostic():
    """
    Master diagnostic pattern combining multiple tests in one image

    Layout (4 quadrants):
    - Top-left: Horizontal ramp (transfer function)
    - Top-right: 8-step grayscale (thresholds)
    - Bottom-left: Zone plate (frequency/aliasing)
    - Bottom-right: Checkerboard (high-freq response)
    """
    data = np.zeros((HEIGHT, WIDTH))

    half_w = WIDTH // 2
    half_h = HEIGHT // 2

    # Top-left: Horizontal ramp
    x = np.linspace(0, 1, half_w)
    data[:half_h, :half_w] = np.tile(x, (half_h, 1))

    # Top-right: 8-step grayscale
    steps = 8
    bar_width = half_w // steps
    for i in range(steps):
        value = i / (steps - 1)
        data[:half_h, half_w + i * bar_width:half_w + (i + 1) * bar_width] = value

    # Bottom-left: Zone plate
    y, x = np.ogrid[:half_h, :half_w]
    cx, cy = half_w // 2, half_h // 2
    r = np.sqrt((x - cx) ** 2 + (y - cy) ** 2)
    data[half_h:, :half_w] = (np.sin(r ** 2 * 0.15) + 1) / 2

    # Bottom-right: Checkerboard
    square_size = 8
    for y in range(half_h):
        for x in range(half_w):
            if ((x // square_size) + (y // square_size)) % 2:
                data[half_h + y, half_w + x] = 1.0

    return save_pattern("synthesis_diagnostic", data,
        "4-quadrant composite diagnostic")


# =============================================================================
# MAIN
# =============================================================================

def generate_all():
    """Generate all test patterns."""
    print(f"\n{'='*60}")
    print(f"VIDEO SYNTHESIS TEST PATTERN GENERATOR")
    print(f"Resolution: {WIDTH}x{HEIGHT} (NTSC square-pixel)")
    print(f"{'='*60}\n")

    print("Basic Ramps (transfer function):")
    generate_ramp_h()
    generate_ramp_v()
    generate_ramp_overscan()

    print("\nStepped Patterns (thresholds):")
    generate_gray10()
    generate_gray50()
    generate_threshold_finder()
    generate_threshold_sweep()

    print("\nEdge Patterns (transitions):")
    generate_edge_sharp()
    generate_edges_multi()
    generate_checker()

    print("\nFrequency Patterns (bandwidth):")
    generate_zone_plate()
    generate_sweep()
    generate_multiburst()

    print("\nVideo Synthesis Specific:")
    generate_colorizer_bars()
    generate_folding_test()
    generate_keyer_test()
    generate_feedback_test()
    generate_comparator_test()

    print("\nReference Fields:")
    generate_black()
    generate_white()

    print("\nColor Patterns:")
    generate_colorbars()
    generate_rgb_ramps()

    print("\nComposite Diagnostics:")
    generate_synthesis_diagnostic()

    print(f"\n{'='*60}")
    print(f"Generated 25 test patterns in {OUTPUT_DIR}")
    print(f"{'='*60}\n")


if __name__ == "__main__":
    generate_all()
