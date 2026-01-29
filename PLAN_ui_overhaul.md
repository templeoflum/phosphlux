# UI Overhaul Plan: Vintage Oscilloscope Aesthetic + Row-Based Node Flow

## Vision

Transform Phosphlux into a cohesive vintage test equipment aesthetic while restructuring the node editor for top-down signal flow with row-based organization. Think Tektronix oscilloscope meets Reason rack.

---

## Part 1: Color Palette & Theme System

### Primary Colors (from vintage_oscilloscope_palette.pdf)

```rust
// src/ui/theme.rs (NEW FILE)

pub struct PhosphluxTheme {
    // Backgrounds
    pub bg_dark: Color32,        // #0A0E0D - CRT Background (deepest)
    pub bg_panel: Color32,       // #1C1C1C - Bakelite Black (panels)
    pub bg_widget: Color32,      // #2A2A2A - Widget backgrounds

    // Hardware grays
    pub panel_front: Color32,    // #848482 - Battleship Gray
    pub case_body: Color32,      // #6B7B8C - Tektronix Gray
    pub aluminum: Color32,       // #A8A9AD - Borders, separators

    // Text
    pub text_primary: Color32,   // #F5F5DC - Cream Label
    pub text_secondary: Color32, // #A8A9AD - Aluminum (muted)
    pub text_disabled: Color32,  // #5A5A5A - Dimmed

    // Phosphor accents (active/signal states)
    pub phosphor_bright: Color32, // #39FF14 - P31 Phosphor Green (bright)
    pub phosphor_glow: Color32,   // #7FFF00 - Standard glow
    pub phosphor_fade: Color32,   // #4A7729 - Persistence/inactive

    // Semantic colors
    pub accent_selected: Color32, // #FFD700 - Dial Yellow
    pub accent_warning: Color32,  // #FFA500 - Warning Amber
    pub accent_error: Color32,    // #C41E3A - Warning Red
    pub accent_info: Color32,     // #4169E1 - P7 Blue

    // Node category colors
    pub cat_generators: Color32,  // #7FFF00 - Green phosphor
    pub cat_paik_abe: Color32,    // #FFD700 - Warm yellow
    pub cat_sandin: Color32,      // #4169E1 - Blue
    pub cat_jones: Color32,       // #FF6600 - Orange

    // Cables/connections
    pub cable_signal: Color32,    // #7FFF00 - Active signal
    pub cable_inactive: Color32,  // #4A7729 - No signal
}

impl Default for PhosphluxTheme {
    fn default() -> Self {
        Self {
            bg_dark: Color32::from_rgb(10, 14, 13),
            bg_panel: Color32::from_rgb(28, 28, 28),
            bg_widget: Color32::from_rgb(42, 42, 42),

            panel_front: Color32::from_rgb(132, 132, 130),
            case_body: Color32::from_rgb(107, 123, 140),
            aluminum: Color32::from_rgb(168, 169, 173),

            text_primary: Color32::from_rgb(245, 245, 220),
            text_secondary: Color32::from_rgb(168, 169, 173),
            text_disabled: Color32::from_rgb(90, 90, 90),

            phosphor_bright: Color32::from_rgb(57, 255, 20),
            phosphor_glow: Color32::from_rgb(127, 255, 0),
            phosphor_fade: Color32::from_rgb(74, 119, 41),

            accent_selected: Color32::from_rgb(255, 215, 0),
            accent_warning: Color32::from_rgb(255, 165, 0),
            accent_error: Color32::from_rgb(196, 30, 58),
            accent_info: Color32::from_rgb(65, 105, 225),

            cat_generators: Color32::from_rgb(127, 255, 0),
            cat_paik_abe: Color32::from_rgb(255, 215, 0),
            cat_sandin: Color32::from_rgb(65, 105, 225),
            cat_jones: Color32::from_rgb(255, 102, 0),

            cable_signal: Color32::from_rgb(127, 255, 0),
            cable_inactive: Color32::from_rgb(74, 119, 41),
        }
    }
}
```

### Apply Theme to egui

```rust
pub fn apply_theme(ctx: &egui::Context, theme: &PhosphluxTheme) {
    let mut style = (*ctx.style()).clone();

    // Visuals
    style.visuals.dark_mode = true;
    style.visuals.panel_fill = theme.bg_panel;
    style.visuals.window_fill = theme.bg_panel;
    style.visuals.extreme_bg_color = theme.bg_dark;
    style.visuals.faint_bg_color = theme.bg_widget;

    // Widget visuals
    style.visuals.widgets.noninteractive.bg_fill = theme.bg_widget;
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, theme.text_secondary);
    style.visuals.widgets.inactive.bg_fill = theme.bg_widget;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, theme.text_primary);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(50, 55, 50);
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, theme.phosphor_glow);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(40, 50, 40);
    style.visuals.widgets.active.fg_stroke = Stroke::new(2.0, theme.phosphor_bright);

    // Selection
    style.visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(255, 215, 0, 60);
    style.visuals.selection.stroke = Stroke::new(1.0, theme.accent_selected);

    // Separator
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, theme.aluminum);

    ctx.set_style(style);
}
```

---

## Part 2: Row-Based Node Layout

### Concept

Instead of free-form placement, nodes organize into **rows**. Each row can hold multiple nodes side-by-side. Signal flows **top-to-bottom** between rows.

```
┌─────────────────────────────────────────────┐
│  ROW 0: [Test Signal]  [Oscillator]         │  ← Sources
├─────────────────────────────────────────────┤
│         ↓                ↓                  │
├─────────────────────────────────────────────┤
│  ROW 1: [Mixer]                             │  ← Processing
├─────────────────────────────────────────────┤
│         ↓                                   │
├─────────────────────────────────────────────┤
│  ROW 2: [Colorizer]                         │  ← Output
└─────────────────────────────────────────────┘
```

### Node Geometry Changes

```rust
// Node dimensions for row-based layout
const NODE_WIDTH: f32 = 160.0;      // Fixed width
const NODE_HEIGHT: f32 = 100.0;     // Base height
const PORT_HEIGHT: f32 = 16.0;      // Height per port
const ROW_SPACING: f32 = 40.0;      // Vertical gap between rows
const NODE_SPACING: f32 = 20.0;     // Horizontal gap within row

// Ports on TOP (inputs) and BOTTOM (outputs)
fn socket_pos(device: &DeviceInstance, is_output: bool, index: usize, rect: Rect) -> Pos2 {
    let port_count = if is_output {
        device.device.outputs().len()
    } else {
        device.device.inputs().len()
    };

    // Distribute ports horizontally across node width
    let spacing = rect.width() / (port_count.max(1) as f32 + 1.0);
    let x = rect.min.x + spacing * (index as f32 + 1.0);

    if is_output {
        Pos2::new(x, rect.max.y)  // Bottom edge
    } else {
        Pos2::new(x, rect.min.y)  // Top edge
    }
}
```

### Row Management

```rust
// Add to NodeEditor struct
pub struct NodeEditor {
    // ... existing fields ...

    /// Row assignments: instance_id -> (row_index, position_in_row)
    pub node_rows: HashMap<u32, (usize, usize)>,

    /// Whether to use row-based layout vs free placement
    pub row_mode: bool,
}

impl NodeEditor {
    /// Place new node in appropriate row
    pub fn add_node_to_row(&mut self, instance_id: u32, patch: &DevicePatch) {
        // Determine row based on connections
        // - Nodes with no inputs go to row 0
        // - Nodes go one row below their highest input source
        let row = self.calculate_node_row(instance_id, patch);
        let pos_in_row = self.nodes_in_row(row);
        self.node_rows.insert(instance_id, (row, pos_in_row));
    }

    /// Calculate Y position based on row
    fn row_y_position(&self, row: usize) -> f32 {
        50.0 + row as f32 * (NODE_HEIGHT + ROW_SPACING)
    }

    /// Calculate X position based on position in row
    fn node_x_position(&self, pos_in_row: usize) -> f32 {
        50.0 + pos_in_row as f32 * (NODE_WIDTH + NODE_SPACING)
    }
}
```

### Vertical Bezier Curves

```rust
fn draw_bezier_connection(painter: &Painter, start: Pos2, end: Pos2, color: Color32, zoom: f32) {
    // Vertical control points for top-down flow
    let y_dist = (end.y - start.y).abs();
    let control_offset = (y_dist * 0.4).max(30.0 * zoom);

    let ctrl1 = Pos2::new(start.x, start.y + control_offset);  // Down from output
    let ctrl2 = Pos2::new(end.x, end.y - control_offset);      // Up to input

    // ... bezier drawing ...
}
```

---

## Part 3: UI Element Styling

### Styled Widgets

#### Sliders (Vintage Knob Feel)
```rust
fn draw_vintage_slider(ui: &mut Ui, value: &mut f32, range: RangeInclusive<f32>, label: &str) {
    // Dark track with phosphor fill
    // Cream label text
    // Yellow highlight on hover
    // Green glow when being adjusted
}
```

#### Buttons
```rust
// Bakelite-style buttons
// - Dark background (#1C1C1C)
// - Cream text (#F5F5DC)
// - Subtle aluminum border
// - Phosphor glow on hover
// - Pressed state: inset shadow
```

#### Panel Headers
```rust
// Battleship gray background
// Cream uppercase text (like panel labels)
// Subtle embossed look
```

#### Collapsing Headers
```rust
// Tektronix gray background
// Triangle indicator in phosphor green
// Cream text
```

### Node Visual Design

```rust
fn draw_node(painter: &Painter, device: &DeviceInstance, rect: Rect, theme: &PhosphluxTheme) {
    // Body: dark panel background with subtle bevel
    painter.rect_filled(rect, 4.0, theme.bg_panel);

    // Top edge: category color bar (3px)
    let category_color = match device.device.category() {
        "generators" => theme.cat_generators,
        "paik_abe" => theme.cat_paik_abe,
        // ...
    };
    let bar = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 3.0));
    painter.rect_filled(bar, 0.0, category_color);

    // Title: cream text, centered
    painter.text(
        rect.center_top() + Vec2::new(0.0, 12.0),
        Align2::CENTER_TOP,
        device.device.device_name(),
        FontId::proportional(11.0),
        theme.text_primary,
    );

    // Input ports (top edge): small circles with labels above
    for (i, input) in device.device.inputs().iter().enumerate() {
        let pos = socket_pos(device, false, i, rect);
        painter.circle_filled(pos, 5.0, theme.aluminum);
        painter.text(pos - Vec2::new(0.0, 10.0), Align2::CENTER_BOTTOM,
            &input.name, FontId::proportional(8.0), theme.text_secondary);
    }

    // Output ports (bottom edge): colored by category
    for (i, output) in device.device.outputs().iter().enumerate() {
        let pos = socket_pos(device, true, i, rect);
        painter.circle_filled(pos, 5.0, category_color);
        painter.text(pos + Vec2::new(0.0, 10.0), Align2::CENTER_TOP,
            &output.name, FontId::proportional(8.0), theme.text_secondary);
    }

    // Selection: yellow glow border
    if is_selected {
        painter.rect_stroke(rect.expand(2.0), 4.0, Stroke::new(2.0, theme.accent_selected));
    }

    // Output indicator: small LED
    if is_output_device {
        let led_pos = rect.right_top() + Vec2::new(-8.0, 8.0);
        painter.circle_filled(led_pos, 4.0, theme.accent_error); // Red LED
    }
}
```

---

## Part 4: Implementation Phases

### Phase 1: Theme System (src/ui/theme.rs)
1. Create `PhosphluxTheme` struct with all colors
2. Create `apply_theme()` function for egui styling
3. Apply theme on app startup
4. **Test**: All panels, buttons, text should use new colors

### Phase 2: Node Geometry
1. Change `socket_pos()` - inputs TOP, outputs BOTTOM
2. Change `node_rect()` - fixed width (160px), height based on ports
3. Update `point_near_bezier()` for vertical curves
4. Update `draw_bezier_connection()` for vertical flow
5. **Test**: Existing patches still work, connections look right

### Phase 3: Node Visuals
1. Update `draw_node()` with new visual design
2. Category color bars
3. Port styling (input vs output differentiation)
4. Selection/hover states with theme colors
5. **Test**: Nodes look cohesive with theme

### Phase 4: Row-Based Layout (Optional Enhancement)
1. Add `node_rows` tracking to `NodeEditor`
2. Auto-placement logic for new nodes
3. Row guides in canvas background
4. Snap-to-row behavior
5. **Test**: New nodes place in logical rows

### Phase 5: Widget Styling
1. Custom slider drawing (or egui style overrides)
2. Button styling
3. Panel header styling
4. Inspector panel polish
5. **Test**: All UI elements feel cohesive

---

## File Changes Summary

| File | Changes |
|------|---------|
| `src/ui/mod.rs` | Add `pub mod theme;` |
| `src/ui/theme.rs` | **NEW** - Theme colors and application |
| `src/ui/node_editor.rs` | socket_pos, node_rect, bezier curves, draw_node, row tracking |
| `src/main.rs` | Apply theme on startup, pass theme to render functions |

---

## Visual Reference

```
┌────────────────────────────────────────────────────────────────┐
│ ▸ File   Help                                    [●] OK        │ ← Menu: bg_panel, text_primary
├──────────────────┬─────────────────────────────────────────────┤
│ DEVICES          │                                             │
│ ─────────────────│    ┌──────────┐      ┌──────────┐          │
│ ▾ Generators     │    │▀▀▀▀▀▀▀▀▀▀│      │▀▀▀▀▀▀▀▀▀▀│          │ ← Green bar
│   [Test Signal]  │    │Test Sig  │      │Oscillator│          │
│   [Oscillator]   │    │          │      │          │          │
│ ▾ Paik/Abe      │    │    ●     │      │    ●     │          │ ← Output port
│   [Colorizer]    │    └────┬─────┘      └────┬─────┘          │
│                  │         │                  │                │
│ ─────────────────│         └────────┬─────────┘                │ ← Phosphor cables
│ INSPECTOR        │                  │                          │
│ ─────────────────│         ┌────────┴─────────┐                │
│ Test Signal      │         │▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀│                │ ← Yellow bar
│                  │         │    ●      ●      │                │ ← Input ports
│ Pattern ══════●  │         │    Colorizer     │                │
│ Speed   ══════●  │         │        ●    [●]  │                │ ← Output + LED
│                  │         └──────────────────┘                │
│                  │                                             │
├──────────────────┼─────────────────────────────────────────────┤
│                  │                  ┌─────────────────┐        │
│                  │                  │                 │        │
│                  │                  │    PREVIEW      │        │
│                  │                  │                 │        │
│                  │                  └─────────────────┘        │
└──────────────────┴─────────────────────────────────────────────┘
     ↑ Battleship gray panels              ↑ CRT dark background
```

---

## Success Criteria

1. **Aesthetic**: App feels like vintage test equipment, not generic dark mode
2. **Readability**: Cream text on dark backgrounds is easy to read
3. **Signal Flow**: Top-to-bottom flow is intuitive
4. **Consistency**: All UI elements use theme colors
5. **Functionality**: All existing features still work
