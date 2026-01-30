//! Phosphlux Theme System
//!
//! Vintage oscilloscope aesthetic inspired by Tektronix test equipment.
//! Based on the vintage_oscilloscope_palette color scheme.

use egui::{Color32, Visuals, Style, Stroke, Rounding};

/// Phosphlux color palette - vintage oscilloscope aesthetic
#[derive(Clone)]
pub struct PhosphluxTheme {
    // Backgrounds
    pub bg_dark: Color32,      // CRT Background (canvas, deepest)
    pub bg_panel: Color32,     // Bakelite Black (panel backgrounds)
    pub bg_widget: Color32,    // Widget backgrounds (buttons, sliders)

    // Hardware Grays
    pub panel_front: Color32,  // Battleship Gray
    pub case_body: Color32,    // Tektronix Gray
    pub aluminum: Color32,     // Borders, separators

    // Text
    pub text_primary: Color32,   // Cream Label (main text)
    pub text_secondary: Color32, // Muted text
    pub text_disabled: Color32,  // Dimmed

    // Phosphor Accents
    pub phosphor_bright: Color32, // P31 Green (bright active states)
    pub phosphor_glow: Color32,   // Standard glow (hover, active)
    pub phosphor_fade: Color32,   // Persistence (inactive, muted)

    // Semantic Colors
    pub accent_selected: Color32, // Dial Yellow (selection)
    pub accent_warning: Color32,  // Warning Amber
    pub accent_error: Color32,    // Warning Red
    pub accent_info: Color32,     // P7 Blue

    // Node Categories
    pub cat_generators: Color32,
    pub cat_paik_abe: Color32,
    pub cat_sandin: Color32,
    pub cat_jones: Color32,
}

impl Default for PhosphluxTheme {
    fn default() -> Self {
        Self {
            // Backgrounds
            bg_dark: Color32::from_rgb(0x0A, 0x0E, 0x0D),      // #0A0E0D
            bg_panel: Color32::from_rgb(0x1C, 0x1C, 0x1C),     // #1C1C1C
            bg_widget: Color32::from_rgb(0x2A, 0x2A, 0x2A),    // #2A2A2A

            // Hardware Grays
            panel_front: Color32::from_rgb(0x84, 0x84, 0x82),  // #848482
            case_body: Color32::from_rgb(0x6B, 0x7B, 0x8C),    // #6B7B8C
            aluminum: Color32::from_rgb(0xA8, 0xA9, 0xAD),     // #A8A9AD

            // Text
            text_primary: Color32::from_rgb(0xF5, 0xF5, 0xDC),   // #F5F5DC (Cream)
            text_secondary: Color32::from_rgb(0xA8, 0xA9, 0xAD), // #A8A9AD
            text_disabled: Color32::from_rgb(0x5A, 0x5A, 0x5A),  // #5A5A5A

            // Phosphor Accents
            phosphor_bright: Color32::from_rgb(0x39, 0xFF, 0x14), // #39FF14
            phosphor_glow: Color32::from_rgb(0x7F, 0xFF, 0x00),   // #7FFF00
            phosphor_fade: Color32::from_rgb(0x4A, 0x77, 0x29),   // #4A7729

            // Semantic Colors
            accent_selected: Color32::from_rgb(0xFF, 0xD7, 0x00), // #FFD700
            accent_warning: Color32::from_rgb(0xFF, 0xA5, 0x00),  // #FFA500
            accent_error: Color32::from_rgb(0xC4, 0x1E, 0x3A),    // #C41E3A
            accent_info: Color32::from_rgb(0x41, 0x69, 0xE1),     // #4169E1

            // Node Categories
            cat_generators: Color32::from_rgb(0x7F, 0xFF, 0x00), // Phosphor green
            cat_paik_abe: Color32::from_rgb(0xFF, 0xD7, 0x00),   // Warm yellow
            cat_sandin: Color32::from_rgb(0x41, 0x69, 0xE1),     // Blue
            cat_jones: Color32::from_rgb(0xFF, 0x66, 0x00),      // Orange
        }
    }
}

impl PhosphluxTheme {
    /// Get color for a device category based on device_id prefix
    pub fn category_color(&self, device_id: &str) -> Color32 {
        if device_id.starts_with("test_") {
            self.cat_generators
        } else if device_id.starts_with("paik_abe_") {
            self.cat_paik_abe
        } else if device_id.starts_with("sandin_") {
            self.cat_sandin
        } else if device_id.starts_with("jones_") {
            self.cat_jones
        } else {
            self.aluminum // Default gray
        }
    }
}

/// Global theme instance for easy access
static THEME: std::sync::OnceLock<PhosphluxTheme> = std::sync::OnceLock::new();

/// Get the global theme instance
pub fn theme() -> &'static PhosphluxTheme {
    THEME.get_or_init(PhosphluxTheme::default)
}

/// Apply the Phosphlux theme to an egui context
pub fn apply_theme(ctx: &egui::Context) {
    let t = theme();

    let mut style = Style::default();

    // Build dark visuals with our colors
    let mut visuals = Visuals::dark();

    // Window/panel backgrounds
    visuals.panel_fill = t.bg_panel;
    visuals.window_fill = t.bg_panel;
    visuals.extreme_bg_color = t.bg_dark;
    visuals.faint_bg_color = t.bg_widget;

    // Text colors
    visuals.override_text_color = Some(t.text_primary);

    // Widget styling
    visuals.widgets.noninteractive.bg_fill = t.bg_widget;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, t.text_secondary);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, t.aluminum.linear_multiply(0.3));
    visuals.widgets.noninteractive.rounding = Rounding::same(2.0);

    visuals.widgets.inactive.bg_fill = t.bg_widget;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, t.text_primary);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, t.aluminum.linear_multiply(0.4));
    visuals.widgets.inactive.rounding = Rounding::same(2.0);

    visuals.widgets.hovered.bg_fill = Color32::from_rgb(0x3A, 0x3A, 0x3A);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, t.phosphor_glow);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, t.phosphor_fade);
    visuals.widgets.hovered.rounding = Rounding::same(2.0);

    visuals.widgets.active.bg_fill = Color32::from_rgb(0x40, 0x40, 0x40);
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, t.phosphor_bright);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, t.phosphor_glow);
    visuals.widgets.active.rounding = Rounding::same(2.0);

    visuals.widgets.open.bg_fill = t.bg_widget;
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, t.text_primary);
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, t.aluminum.linear_multiply(0.5));
    visuals.widgets.open.rounding = Rounding::same(2.0);

    // Selection
    visuals.selection.bg_fill = t.accent_selected.linear_multiply(0.3);
    visuals.selection.stroke = Stroke::new(1.0, t.accent_selected);

    // Hyperlinks
    visuals.hyperlink_color = t.accent_info;

    // Warnings/errors
    visuals.warn_fg_color = t.accent_warning;
    visuals.error_fg_color = t.accent_error;

    // Separators and window stroke
    visuals.window_stroke = Stroke::new(1.0, t.aluminum.linear_multiply(0.3));

    // Slider rail
    visuals.widgets.inactive.expansion = 0.0;

    style.visuals = visuals;

    // Spacing adjustments for denser UI
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(6.0, 2.0);

    ctx.set_style(style);
}
