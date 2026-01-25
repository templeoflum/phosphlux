//! Node Graph Editor
//!
//! Visual node-based compositor interface for signal routing.
//! Inspired by Blender's shader editor, Nuke, and TouchDesigner.

use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use super::patch::{SignalPatch, SignalNode, NodeType, Connection};

/// Aspect ratio options for preview
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AspectRatio {
    Free,       // Fill available space
    Square,     // 1:1
    Standard,   // 4:3
    Wide,       // 16:9
    Portrait43, // 3:4
    Portrait169,// 9:16
    Cinemascope,// 2.35:1
}

impl AspectRatio {
    pub fn ratio(&self) -> Option<f32> {
        match self {
            AspectRatio::Free => None,
            AspectRatio::Square => Some(1.0),
            AspectRatio::Standard => Some(4.0 / 3.0),
            AspectRatio::Wide => Some(16.0 / 9.0),
            AspectRatio::Portrait43 => Some(3.0 / 4.0),
            AspectRatio::Portrait169 => Some(9.0 / 16.0),
            AspectRatio::Cinemascope => Some(2.35),
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AspectRatio::Free => "Free",
            AspectRatio::Square => "1:1",
            AspectRatio::Standard => "4:3",
            AspectRatio::Wide => "16:9",
            AspectRatio::Portrait43 => "3:4 (Portrait)",
            AspectRatio::Portrait169 => "9:16 (Portrait)",
            AspectRatio::Cinemascope => "2.35:1 (Cinema)",
        }
    }

    /// Calculate size that fits within available space while maintaining aspect ratio
    pub fn fit_size(&self, available: Vec2) -> Vec2 {
        match self.ratio() {
            None => available,
            Some(target_ratio) => {
                let available_ratio = available.x / available.y;
                if available_ratio > target_ratio {
                    // Available space is wider than target - fit to height
                    Vec2::new(available.y * target_ratio, available.y)
                } else {
                    // Available space is taller than target - fit to width
                    Vec2::new(available.x, available.x / target_ratio)
                }
            }
        }
    }
}

/// Visual state for the node editor
pub struct NodeEditor {
    /// Pan offset for the canvas
    pub pan: Vec2,
    /// Zoom level (1.0 = 100%)
    pub zoom: f32,
    /// Currently selected node ID
    pub selected_node: Option<u32>,
    /// Node being dragged
    dragging_node: Option<u32>,
    /// Drag offset from node origin (for smooth dragging)
    drag_offset: Vec2,
    /// Connection in progress (from_node, from_output, current_pos)
    pending_connection: Option<(u32, usize, Pos2)>,
    /// Is the canvas being panned
    is_panning: bool,
    /// Last mouse position for delta calculation
    last_mouse_pos: Option<Pos2>,
    /// Selected aspect ratio for preview
    pub aspect_ratio: AspectRatio,
}

impl Default for NodeEditor {
    fn default() -> Self {
        Self {
            pan: Vec2::new(100.0, 100.0),
            zoom: 1.0,
            selected_node: None,
            dragging_node: None,
            drag_offset: Vec2::ZERO,
            pending_connection: None,
            is_panning: false,
            last_mouse_pos: None,
            aspect_ratio: AspectRatio::Standard, // Default to 4:3
        }
    }
}

impl NodeEditor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert screen position to canvas position
    fn screen_to_canvas(&self, screen_pos: Pos2, canvas_rect: Rect) -> Pos2 {
        let relative = screen_pos - canvas_rect.min;
        Pos2::new(
            (relative.x - self.pan.x) / self.zoom,
            (relative.y - self.pan.y) / self.zoom,
        )
    }

    /// Convert canvas position to screen position
    fn canvas_to_screen(&self, canvas_pos: Pos2, canvas_rect: Rect) -> Pos2 {
        Pos2::new(
            canvas_pos.x * self.zoom + self.pan.x + canvas_rect.min.x,
            canvas_pos.y * self.zoom + self.pan.y + canvas_rect.min.y,
        )
    }

    /// Get the visual bounds of a node
    fn node_rect(&self, node: &SignalNode, canvas_rect: Rect) -> Rect {
        let pos = self.canvas_to_screen(Pos2::new(node.position.0, node.position.1), canvas_rect);
        let size = Vec2::new(120.0 * self.zoom, 60.0 * self.zoom);
        Rect::from_min_size(pos, size)
    }

    /// Get socket position for a node
    fn socket_pos(&self, node: &SignalNode, is_output: bool, index: usize, canvas_rect: Rect) -> Pos2 {
        let rect = self.node_rect(node, canvas_rect);

        if is_output {
            // Output socket on right side
            Pos2::new(rect.max.x, rect.min.y + 30.0 * self.zoom)
        } else {
            // Input sockets on left side, stacked vertically
            let input_count = node.node_type.input_count();
            let spacing = rect.height() / (input_count + 1) as f32;
            Pos2::new(rect.min.x, rect.min.y + spacing * (index + 1) as f32)
        }
    }
}

/// Node color based on category
fn node_color(node_type: &NodeType) -> Color32 {
    match node_type.category() {
        "Sources" => Color32::from_rgb(80, 140, 80),      // Green
        "Waveshapers" => Color32::from_rgb(140, 100, 80), // Orange/Brown
        "Mixing" => Color32::from_rgb(80, 100, 140),      // Blue
        "Color" => Color32::from_rgb(140, 80, 140),       // Purple
        "Colorizers" => Color32::from_rgb(140, 120, 80),  // Gold
        "Outputs" => Color32::from_rgb(140, 80, 80),      // Red
        _ => Color32::from_rgb(100, 100, 100),            // Gray
    }
}

/// Render the node graph canvas
pub fn render_node_canvas(
    ui: &mut egui::Ui,
    editor: &mut NodeEditor,
    patch: &mut SignalPatch,
) {
    let canvas_rect = ui.available_rect_before_wrap();

    // Allocate the full available space
    let response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());
    let painter = ui.painter_at(canvas_rect);

    // Draw background grid
    draw_grid(&painter, canvas_rect, editor.pan, editor.zoom);

    // Handle canvas interactions
    handle_canvas_input(ui, &response, editor, patch, canvas_rect);

    // Draw connections first (behind nodes)
    draw_connections(&painter, editor, patch, canvas_rect);

    // Draw pending connection if any
    if let Some((from_node, from_output, end_pos)) = editor.pending_connection {
        if let Some(node) = patch.get_node(from_node) {
            let start = editor.socket_pos(node, true, from_output, canvas_rect);
            draw_bezier_connection(&painter, start, end_pos, Color32::from_rgb(200, 200, 100), editor.zoom);
        }
    }

    // Draw nodes
    for node in &patch.nodes {
        draw_node(&painter, ui, editor, node, canvas_rect);
    }

    // Show zoom level
    painter.text(
        canvas_rect.min + Vec2::new(10.0, 10.0),
        egui::Align2::LEFT_TOP,
        format!("Zoom: {:.0}%", editor.zoom * 100.0),
        egui::FontId::proportional(12.0),
        Color32::from_rgb(150, 150, 150),
    );
}

/// Draw the background grid
fn draw_grid(painter: &egui::Painter, rect: Rect, pan: Vec2, zoom: f32) {
    let grid_spacing = 50.0 * zoom;
    let grid_color = Color32::from_rgb(40, 40, 45);
    let major_grid_color = Color32::from_rgb(50, 50, 55);

    // Calculate grid offset based on pan
    let offset_x = pan.x % grid_spacing;
    let offset_y = pan.y % grid_spacing;

    // Draw vertical lines
    let mut x = rect.min.x + offset_x;
    let mut i = 0;
    while x < rect.max.x {
        let color = if i % 4 == 0 { major_grid_color } else { grid_color };
        painter.line_segment(
            [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
            Stroke::new(1.0, color),
        );
        x += grid_spacing;
        i += 1;
    }

    // Draw horizontal lines
    let mut y = rect.min.y + offset_y;
    i = 0;
    while y < rect.max.y {
        let color = if i % 4 == 0 { major_grid_color } else { grid_color };
        painter.line_segment(
            [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
            Stroke::new(1.0, color),
        );
        y += grid_spacing;
        i += 1;
    }
}

/// Handle input on the canvas
fn handle_canvas_input(
    ui: &egui::Ui,
    response: &egui::Response,
    editor: &mut NodeEditor,
    patch: &mut SignalPatch,
    canvas_rect: Rect,
) {
    let pointer_pos = ui.input(|i| i.pointer.hover_pos());
    let primary_pressed = ui.input(|i| i.pointer.primary_pressed());
    let primary_down = ui.input(|i| i.pointer.primary_down());
    let primary_released = ui.input(|i| i.pointer.primary_released());

    // Scroll to zoom
    if response.hovered() {
        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_delta != 0.0 {
            let zoom_delta = scroll_delta * 0.001;
            editor.zoom = (editor.zoom + zoom_delta).clamp(0.25, 2.0);
        }
    }

    // Middle mouse or Alt+Left to pan
    let is_panning = ui.input(|i| i.pointer.middle_down() || (i.modifiers.alt && i.pointer.primary_down()));

    if is_panning {
        if let Some(pos) = pointer_pos {
            if let Some(last) = editor.last_mouse_pos {
                let delta = pos - last;
                editor.pan += delta;
            }
            editor.last_mouse_pos = Some(pos);
        }
        editor.is_panning = true;
        return; // Don't process node interactions while panning
    } else {
        editor.is_panning = false;
    }

    // Always track mouse position
    editor.last_mouse_pos = pointer_pos;

    // Check for node interactions
    if let Some(pos) = pointer_pos {
        // On mouse press - immediately start interaction
        if primary_pressed && response.hovered() {
            // First check output sockets (higher priority for connection creation)
            let mut started_connection = false;
            for node in &patch.nodes {
                if node.node_type.has_outputs() {
                    let socket_pos = editor.socket_pos(node, true, 0, canvas_rect);
                    let socket_radius = 10.0 * editor.zoom;
                    if (pos - socket_pos).length() < socket_radius {
                        editor.pending_connection = Some((node.id, 0, pos));
                        started_connection = true;
                        break;
                    }
                }
            }

            // If not starting a connection, check for node selection/drag
            if !started_connection {
                let mut found_node = false;
                for node in &patch.nodes {
                    let rect = editor.node_rect(node, canvas_rect);
                    if rect.contains(pos) {
                        // Calculate offset from node origin for smooth dragging
                        let node_screen_pos = editor.canvas_to_screen(
                            Pos2::new(node.position.0, node.position.1),
                            canvas_rect
                        );
                        editor.drag_offset = pos - node_screen_pos;
                        editor.dragging_node = Some(node.id);
                        editor.selected_node = Some(node.id);
                        found_node = true;
                        break;
                    }
                }

                // Clicked on empty space - deselect
                if !found_node {
                    editor.selected_node = None;
                }
            }
        }

        // While mouse is held - handle dragging
        if primary_down {
            // Handle node dragging with direct position setting (more responsive)
            if let Some(node_id) = editor.dragging_node {
                let canvas_pos = editor.screen_to_canvas(pos - editor.drag_offset, canvas_rect);
                if let Some(node) = patch.get_node_mut(node_id) {
                    node.position.0 = canvas_pos.x;
                    node.position.1 = canvas_pos.y;
                }
            }

            // Update pending connection endpoint
            if let Some((_, _, ref mut end_pos)) = editor.pending_connection {
                *end_pos = pos;
            }
        }

        // On mouse release - complete interactions
        if primary_released {
            // Check if completing a connection
            if let Some((from_node, from_output, _)) = editor.pending_connection.take() {
                // Find target socket - collect info first to avoid borrow conflict
                let mut target: Option<(u32, usize)> = None;

                for node in &patch.nodes {
                    if !node.node_type.has_inputs() || node.id == from_node {
                        continue;
                    }

                    for input_idx in 0..node.node_type.input_count() {
                        let socket_pos = editor.socket_pos(node, false, input_idx, canvas_rect);
                        let socket_radius = 12.0 * editor.zoom;
                        if (pos - socket_pos).length() < socket_radius {
                            target = Some((node.id, input_idx));
                            break;
                        }
                    }
                    if target.is_some() {
                        break;
                    }
                }

                // Now connect if we found a target
                if let Some((to_node, to_input)) = target {
                    patch.connect(from_node, from_output, to_node, to_input);
                }
            }

            editor.dragging_node = None;
            editor.drag_offset = Vec2::ZERO;
        }
    }

    // Right-click to disconnect or delete
    if response.secondary_clicked() {
        if let Some(pos) = pointer_pos {
            // Check if clicking on a node to delete
            for node in &patch.nodes {
                let rect = editor.node_rect(node, canvas_rect);
                if rect.contains(pos) {
                    // Don't delete main outputs
                    if node.node_type != NodeType::MainOut && node.node_type != NodeType::FeedbackOut {
                        let id_to_remove = node.id;
                        patch.remove_node(id_to_remove);
                        if editor.selected_node == Some(id_to_remove) {
                            editor.selected_node = None;
                        }
                    }
                    break;
                }
            }
        }
    }
}

/// Draw all connections
fn draw_connections(
    painter: &egui::Painter,
    editor: &NodeEditor,
    patch: &SignalPatch,
    canvas_rect: Rect,
) {
    for conn in &patch.connections {
        let from_node = patch.get_node(conn.from_node);
        let to_node = patch.get_node(conn.to_node);

        if let (Some(from), Some(to)) = (from_node, to_node) {
            let start = editor.socket_pos(from, true, conn.from_output, canvas_rect);
            let end = editor.socket_pos(to, false, conn.to_input, canvas_rect);

            let color = node_color(&from.node_type);
            draw_bezier_connection(painter, start, end, color, editor.zoom);
        }
    }
}

/// Draw a bezier curve connection
fn draw_bezier_connection(painter: &egui::Painter, start: Pos2, end: Pos2, color: Color32, zoom: f32) {
    let control_offset = ((end.x - start.x).abs() * 0.5).max(50.0 * zoom);

    let ctrl1 = Pos2::new(start.x + control_offset, start.y);
    let ctrl2 = Pos2::new(end.x - control_offset, end.y);

    // Draw bezier as line segments
    let segments = 20;
    let mut points = Vec::with_capacity(segments + 1);

    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        let x = mt3 * start.x + 3.0 * mt2 * t * ctrl1.x + 3.0 * mt * t2 * ctrl2.x + t3 * end.x;
        let y = mt3 * start.y + 3.0 * mt2 * t * ctrl1.y + 3.0 * mt * t2 * ctrl2.y + t3 * end.y;

        points.push(Pos2::new(x, y));
    }

    for i in 0..segments {
        painter.line_segment(
            [points[i], points[i + 1]],
            Stroke::new(2.0 * zoom, color),
        );
    }
}

/// Draw a single node
fn draw_node(
    painter: &egui::Painter,
    _ui: &egui::Ui,
    editor: &NodeEditor,
    node: &SignalNode,
    canvas_rect: Rect,
) {
    let rect = editor.node_rect(node, canvas_rect);
    let is_selected = editor.selected_node == Some(node.id);

    let base_color = node_color(&node.node_type);
    let header_color = base_color;
    let body_color = Color32::from_rgb(35, 35, 40);

    // Node body
    painter.rect_filled(rect, 4.0 * editor.zoom, body_color);

    // Node header
    let header_rect = Rect::from_min_max(
        rect.min,
        Pos2::new(rect.max.x, rect.min.y + 20.0 * editor.zoom),
    );
    painter.rect_filled(header_rect, 4.0 * editor.zoom, header_color);

    // Selection border
    if is_selected {
        painter.rect_stroke(rect, 4.0 * editor.zoom, Stroke::new(2.0, Color32::WHITE));
    } else {
        painter.rect_stroke(rect, 4.0 * editor.zoom, Stroke::new(1.0, Color32::from_rgb(60, 60, 65)));
    }

    // Node title
    painter.text(
        rect.min + Vec2::new(8.0 * editor.zoom, 4.0 * editor.zoom),
        egui::Align2::LEFT_TOP,
        node.node_type.display_name(),
        egui::FontId::proportional(11.0 * editor.zoom),
        Color32::WHITE,
    );

    // Draw input sockets
    if node.node_type.has_inputs() {
        for i in 0..node.node_type.input_count() {
            let pos = editor.socket_pos(node, false, i, canvas_rect);
            painter.circle_filled(pos, 5.0 * editor.zoom, Color32::from_rgb(80, 80, 90));
            painter.circle_stroke(pos, 5.0 * editor.zoom, Stroke::new(1.0, Color32::from_rgb(120, 120, 130)));
        }
    }

    // Draw output socket
    if node.node_type.has_outputs() {
        let pos = editor.socket_pos(node, true, 0, canvas_rect);
        painter.circle_filled(pos, 5.0 * editor.zoom, base_color);
        painter.circle_stroke(pos, 5.0 * editor.zoom, Stroke::new(1.0, Color32::WHITE));
    }
}

/// Render the node palette (left panel)
pub fn render_node_palette(ui: &mut egui::Ui, patch: &mut SignalPatch) {
    ui.heading("Nodes");
    ui.separator();

    let categories = ["Sources", "Waveshapers", "Mixing", "Color", "Colorizers", "Outputs"];

    for category in categories {
        egui::CollapsingHeader::new(category)
            .default_open(category == "Sources")
            .show(ui, |ui| {
                let node_types = get_node_types_for_category(category);
                for node_type in node_types {
                    if ui.button(node_type.display_name()).clicked() {
                        // Add node at a default position
                        patch.add_node(node_type, 200.0, 200.0);
                    }
                }
            });
    }
}

/// Get all node types for a category
fn get_node_types_for_category(category: &str) -> Vec<NodeType> {
    let all_types = [
        NodeType::RampH, NodeType::RampV, NodeType::RampRadial,
        NodeType::OscillatorH, NodeType::OscillatorV,
        NodeType::NoiseWhite, NodeType::NoisePink, NodeType::NoiseBrown,
        NodeType::ShapeCircle, NodeType::ShapeRect, NodeType::ShapeDiamond,
        NodeType::FeedbackIn,
        NodeType::Folder, NodeType::Quantizer, NodeType::SoftClip,
        NodeType::Solarize, NodeType::Inverter,
        NodeType::LumaKey, NodeType::ChromaKey, NodeType::Mixer, NodeType::Multiplier,
        NodeType::HueShift,
        NodeType::ColorSpectrum, NodeType::ColorThreshold, NodeType::ColorGradient,
        NodeType::MainOut, NodeType::FeedbackOut,
    ];

    all_types.iter()
        .filter(|t| t.category() == category)
        .copied()
        .collect()
}

/// Render the inspector panel (shows selected node properties)
pub fn render_inspector(ui: &mut egui::Ui, editor: &NodeEditor, patch: &mut SignalPatch) {
    ui.heading("Inspector");
    ui.separator();

    if let Some(node_id) = editor.selected_node {
        if let Some(node) = patch.get_node_mut(node_id) {
            ui.label(format!("Type: {}", node.node_type.display_name()));
            ui.label(format!("Category: {}", node.node_type.category()));
            ui.separator();

            // Render parameters based on node type
            let params_clone: Vec<(String, f32)> = node.params.iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect();

            for (key, value) in params_clone {
                let mut val = value;
                let range = get_param_range(&key);

                if ui.add(egui::Slider::new(&mut val, range).text(&key)).changed() {
                    node.params.insert(key, val);
                }
            }

            ui.separator();
            ui.label(format!("Inputs: {}", node.node_type.input_count()));
            ui.label(format!("Has Output: {}", node.node_type.has_outputs()));
        }
    } else {
        ui.label("No node selected");
        ui.separator();
        ui.label("Click a node to inspect");
        ui.label("Drag from output to input to connect");
        ui.label("Right-click node to delete");
        ui.label("Middle-drag or Alt+drag to pan");
        ui.label("Scroll to zoom");
    }
}

/// Get the range for a parameter
fn get_param_range(param: &str) -> std::ops::RangeInclusive<f32> {
    match param {
        "frequency" => 0.1..=20.0,
        "phase" => 0.0..=1.0,
        "gain" => 0.5..=5.0,
        "levels" => 2.0..=16.0,
        "threshold" => 0.0..=1.0,
        "clip_black" => 0.0..=1.0,
        "clip_white" => 0.0..=1.0,
        "mix" => 0.0..=1.0,
        "amount" => -1.0..=1.0,
        "size" => 0.05..=0.8,
        "softness" => 0.0..=0.2,
        _ => 0.0..=1.0,
    }
}
