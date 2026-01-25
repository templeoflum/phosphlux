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

/// Connection drag mode - whether dragging from output or unplugging from input
#[derive(Clone, Copy, Debug)]
pub enum ConnectionDragMode {
    /// Creating new connection from output socket
    FromOutput { node_id: u32, output_idx: usize },
    /// Unplugging existing connection from input socket
    FromInput { original_from_node: u32, original_from_output: usize, to_node: u32, to_input: usize },
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
    /// Connection being dragged (mode + current mouse position)
    pending_connection: Option<(ConnectionDragMode, Pos2)>,
    /// Is the canvas being panned
    is_panning: bool,
    /// Last mouse position for delta calculation
    last_mouse_pos: Option<Pos2>,
    /// Selected aspect ratio for preview
    pub aspect_ratio: AspectRatio,
    /// Hovered node ID (for visual feedback)
    hovered_node: Option<u32>,
    /// Hovered socket (node_id, is_output, index)
    hovered_socket: Option<(u32, bool, usize)>,
    /// Hovered connection index (for click-to-delete)
    hovered_connection: Option<usize>,
    /// Flag set when the graph has been modified (for GPU update)
    pub graph_modified: bool,
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
            hovered_node: None,
            hovered_socket: None,
            hovered_connection: None,
            graph_modified: false,
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

    /// Get socket radius for hit testing
    fn socket_radius(&self) -> f32 {
        10.0 * self.zoom
    }

    /// Check if a point is near a bezier curve (for connection hovering)
    fn point_near_bezier(&self, point: Pos2, start: Pos2, end: Pos2, threshold: f32) -> bool {
        let control_offset = ((end.x - start.x).abs() * 0.5).max(50.0 * self.zoom);
        let ctrl1 = Pos2::new(start.x + control_offset, start.y);
        let ctrl2 = Pos2::new(end.x - control_offset, end.y);

        // Sample points along the bezier and check distance
        for i in 0..=20 {
            let t = i as f32 / 20.0;
            let t2 = t * t;
            let t3 = t2 * t;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let mt3 = mt2 * mt;

            let x = mt3 * start.x + 3.0 * mt2 * t * ctrl1.x + 3.0 * mt * t2 * ctrl2.x + t3 * end.x;
            let y = mt3 * start.y + 3.0 * mt2 * t * ctrl1.y + 3.0 * mt * t2 * ctrl2.y + t3 * end.y;

            if (point - Pos2::new(x, y)).length() < threshold {
                return true;
            }
        }
        false
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
    if let Some((mode, end_pos)) = editor.pending_connection {
        let (from_node, from_output) = match mode {
            ConnectionDragMode::FromOutput { node_id, output_idx } => (node_id, output_idx),
            ConnectionDragMode::FromInput { original_from_node, original_from_output, .. } => {
                (original_from_node, original_from_output)
            }
        };
        if let Some(node) = patch.get_node(from_node) {
            let start = editor.socket_pos(node, true, from_output, canvas_rect);
            draw_bezier_connection(&painter, start, end_pos, Color32::from_rgb(255, 220, 100), editor.zoom, true);
        }
    }

    // Draw nodes
    for node in &patch.nodes {
        draw_node(&painter, ui, editor, node, canvas_rect, patch);
    }

    // Show zoom level and hints
    painter.text(
        canvas_rect.min + Vec2::new(10.0, 10.0),
        egui::Align2::LEFT_TOP,
        format!("Zoom: {:.0}%", editor.zoom * 100.0),
        egui::FontId::proportional(12.0),
        Color32::from_rgb(150, 150, 150),
    );

    // Show contextual hints based on hover state
    let hint = if editor.hovered_connection.is_some() {
        "Right-click to delete connection"
    } else if editor.hovered_socket.is_some() {
        if let Some((_, is_output, _)) = editor.hovered_socket {
            if is_output {
                "Drag to connect"
            } else {
                "Drag to unplug / Right-click to disconnect"
            }
        } else {
            ""
        }
    } else if editor.hovered_node.is_some() {
        "Drag to move • Right-click to delete • Del key"
    } else {
        "Middle-drag to pan • Scroll to zoom • Home to reset"
    };

    painter.text(
        Pos2::new(canvas_rect.min.x + 10.0, canvas_rect.max.y - 20.0),
        egui::Align2::LEFT_TOP,
        hint,
        egui::FontId::proportional(11.0),
        Color32::from_rgb(120, 120, 130),
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

    // Reset hover states
    editor.hovered_node = None;
    editor.hovered_socket = None;
    editor.hovered_connection = None;

    // Scroll to zoom (with zoom-to-cursor)
    if response.hovered() {
        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_delta != 0.0 && pointer_pos.is_some() {
            let pos = pointer_pos.unwrap();
            let old_zoom = editor.zoom;
            let zoom_delta = scroll_delta * 0.001;
            editor.zoom = (editor.zoom + zoom_delta).clamp(0.25, 2.0);

            // Zoom toward cursor position
            let zoom_ratio = editor.zoom / old_zoom;
            let relative = pos - canvas_rect.min;
            editor.pan = relative - (relative - editor.pan) * zoom_ratio;
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

    // Update hover states for visual feedback
    if let Some(pos) = pointer_pos {
        if response.hovered() {
            // Check socket hover (highest priority)
            'socket_check: for node in &patch.nodes {
                // Check output socket
                if node.node_type.has_outputs() {
                    let socket_pos = editor.socket_pos(node, true, 0, canvas_rect);
                    if (pos - socket_pos).length() < editor.socket_radius() {
                        editor.hovered_socket = Some((node.id, true, 0));
                        break 'socket_check;
                    }
                }
                // Check input sockets
                if node.node_type.has_inputs() {
                    for i in 0..node.node_type.input_count() {
                        let socket_pos = editor.socket_pos(node, false, i, canvas_rect);
                        if (pos - socket_pos).length() < editor.socket_radius() {
                            editor.hovered_socket = Some((node.id, false, i));
                            break 'socket_check;
                        }
                    }
                }
            }

            // Check connection hover (for click-to-delete)
            if editor.hovered_socket.is_none() {
                for (idx, conn) in patch.connections.iter().enumerate() {
                    if let (Some(from), Some(to)) = (patch.get_node(conn.from_node), patch.get_node(conn.to_node)) {
                        let start = editor.socket_pos(from, true, conn.from_output, canvas_rect);
                        let end = editor.socket_pos(to, false, conn.to_input, canvas_rect);
                        if editor.point_near_bezier(pos, start, end, 6.0 * editor.zoom) {
                            editor.hovered_connection = Some(idx);
                            break;
                        }
                    }
                }
            }

            // Check node hover
            if editor.hovered_socket.is_none() && editor.hovered_connection.is_none() {
                for node in &patch.nodes {
                    let rect = editor.node_rect(node, canvas_rect);
                    if rect.contains(pos) {
                        editor.hovered_node = Some(node.id);
                        break;
                    }
                }
            }
        }
    }

    // Handle keyboard shortcuts
    if response.hovered() {
        // Delete key to remove selected node
        let delete_pressed = ui.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace));
        if delete_pressed {
            if let Some(node_id) = editor.selected_node {
                if let Some(node) = patch.get_node(node_id) {
                    if node.node_type != NodeType::MainOut && node.node_type != NodeType::FeedbackOut {
                        patch.remove_node(node_id);
                        editor.selected_node = None;
                        editor.graph_modified = true;
                    }
                }
            }
        }

        // Home key to reset view
        if ui.input(|i| i.key_pressed(egui::Key::Home)) {
            editor.pan = Vec2::new(100.0, 100.0);
            editor.zoom = 1.0;
        }

        // F key to frame selected node
        if ui.input(|i| i.key_pressed(egui::Key::F)) {
            if let Some(node_id) = editor.selected_node {
                if let Some(node) = patch.get_node(node_id) {
                    let center = canvas_rect.center();
                    editor.pan = Vec2::new(
                        center.x - canvas_rect.min.x - node.position.0 * editor.zoom,
                        center.y - canvas_rect.min.y - node.position.1 * editor.zoom,
                    );
                }
            }
        }
    }

    // Check for node interactions
    if let Some(pos) = pointer_pos {
        // On mouse press - immediately start interaction
        if primary_pressed && response.hovered() {
            let mut started_interaction = false;

            // First check output sockets (for creating new connections)
            for node in &patch.nodes {
                if node.node_type.has_outputs() {
                    let socket_pos = editor.socket_pos(node, true, 0, canvas_rect);
                    if (pos - socket_pos).length() < editor.socket_radius() {
                        editor.pending_connection = Some((
                            ConnectionDragMode::FromOutput { node_id: node.id, output_idx: 0 },
                            pos
                        ));
                        started_interaction = true;
                        break;
                    }
                }
            }

            // Check input sockets (for unplugging existing connections)
            if !started_interaction {
                // First pass: find the input socket and connection info
                let mut unplug_info: Option<(u32, usize, u32, usize)> = None; // (to_node, to_input, from_node, from_output)

                for node in &patch.nodes {
                    if node.node_type.has_inputs() {
                        for input_idx in 0..node.node_type.input_count() {
                            let socket_pos = editor.socket_pos(node, false, input_idx, canvas_rect);
                            if (pos - socket_pos).length() < editor.socket_radius() {
                                // Check if there's a connection to this input
                                if let Some(conn) = patch.connections.iter().find(|c| c.to_node == node.id && c.to_input == input_idx) {
                                    unplug_info = Some((node.id, input_idx, conn.from_node, conn.from_output));
                                    break;
                                }
                            }
                        }
                    }
                    if unplug_info.is_some() {
                        break;
                    }
                }

                // Second pass: perform the unplug if we found one
                if let Some((to_node, to_input, from_node, from_output)) = unplug_info {
                    editor.pending_connection = Some((
                        ConnectionDragMode::FromInput {
                            original_from_node: from_node,
                            original_from_output: from_output,
                            to_node,
                            to_input,
                        },
                        pos
                    ));
                    // Remove the connection (we're unplugging it)
                    patch.disconnect(to_node, to_input);
                    editor.graph_modified = true;
                    started_interaction = true;
                }
            }

            // Check for node selection/drag
            if !started_interaction {
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
            if let Some((_, ref mut end_pos)) = editor.pending_connection {
                *end_pos = pos;
            }
        }

        // On mouse release - complete interactions
        if primary_released {
            // Check if completing a connection
            if let Some((mode, _)) = editor.pending_connection.take() {
                // Get the source node info based on drag mode
                let (from_node, from_output) = match mode {
                    ConnectionDragMode::FromOutput { node_id, output_idx } => (node_id, output_idx),
                    ConnectionDragMode::FromInput { original_from_node, original_from_output, .. } => {
                        (original_from_node, original_from_output)
                    }
                };

                // Find target input socket
                let mut target: Option<(u32, usize)> = None;
                for node in &patch.nodes {
                    if !node.node_type.has_inputs() || node.id == from_node {
                        continue;
                    }

                    for input_idx in 0..node.node_type.input_count() {
                        let socket_pos = editor.socket_pos(node, false, input_idx, canvas_rect);
                        if (pos - socket_pos).length() < editor.socket_radius() + 2.0 {
                            target = Some((node.id, input_idx));
                            break;
                        }
                    }
                    if target.is_some() {
                        break;
                    }
                }

                // Connect if we found a target
                if let Some((to_node, to_input)) = target {
                    patch.connect(from_node, from_output, to_node, to_input);
                    editor.graph_modified = true;
                }
            }

            editor.dragging_node = None;
            editor.drag_offset = Vec2::ZERO;
        }
    }

    // Right-click context actions
    if response.secondary_clicked() {
        if let Some(pos) = pointer_pos {
            // First check if clicking on a connection to delete it
            if let Some(conn_idx) = editor.hovered_connection {
                if conn_idx < patch.connections.len() {
                    patch.connections.remove(conn_idx);
                    editor.graph_modified = true;
                    return;
                }
            }

            // Check if clicking on a node to delete
            for node in &patch.nodes {
                let rect = editor.node_rect(node, canvas_rect);
                if rect.contains(pos) {
                    // Don't delete main outputs
                    if node.node_type != NodeType::MainOut && node.node_type != NodeType::FeedbackOut {
                        let id_to_remove = node.id;
                        patch.remove_node(id_to_remove);
                        editor.graph_modified = true;
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
    for (idx, conn) in patch.connections.iter().enumerate() {
        let from_node = patch.get_node(conn.from_node);
        let to_node = patch.get_node(conn.to_node);

        if let (Some(from), Some(to)) = (from_node, to_node) {
            let start = editor.socket_pos(from, true, conn.from_output, canvas_rect);
            let end = editor.socket_pos(to, false, conn.to_input, canvas_rect);

            let is_hovered = editor.hovered_connection == Some(idx);
            let base_color = node_color(&from.node_type);
            let color = if is_hovered {
                Color32::from_rgb(255, 100, 100) // Red highlight for hovered connection
            } else {
                base_color
            };
            draw_bezier_connection(painter, start, end, color, editor.zoom, is_hovered);
        }
    }
}

/// Draw a bezier curve connection
fn draw_bezier_connection(painter: &egui::Painter, start: Pos2, end: Pos2, color: Color32, zoom: f32, highlighted: bool) {
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

    let stroke_width = if highlighted { 3.5 * zoom } else { 2.0 * zoom };
    for i in 0..segments {
        painter.line_segment(
            [points[i], points[i + 1]],
            Stroke::new(stroke_width, color),
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
    patch: &SignalPatch,
) {
    let rect = editor.node_rect(node, canvas_rect);
    let is_selected = editor.selected_node == Some(node.id);
    let is_hovered = editor.hovered_node == Some(node.id);

    let base_color = node_color(&node.node_type);
    let header_color = if is_hovered && !is_selected {
        // Lighten header on hover
        Color32::from_rgb(
            (base_color.r() as u16 + 30).min(255) as u8,
            (base_color.g() as u16 + 30).min(255) as u8,
            (base_color.b() as u16 + 30).min(255) as u8,
        )
    } else {
        base_color
    };
    let body_color = Color32::from_rgb(35, 35, 40);

    // Node body
    painter.rect_filled(rect, 4.0 * editor.zoom, body_color);

    // Node header
    let header_rect = Rect::from_min_max(
        rect.min,
        Pos2::new(rect.max.x, rect.min.y + 20.0 * editor.zoom),
    );
    painter.rect_filled(header_rect, 4.0 * editor.zoom, header_color);

    // Selection/hover border
    if is_selected {
        painter.rect_stroke(rect, 4.0 * editor.zoom, Stroke::new(2.0, Color32::WHITE));
    } else if is_hovered {
        painter.rect_stroke(rect, 4.0 * editor.zoom, Stroke::new(1.5, Color32::from_rgb(150, 150, 160)));
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
            let is_socket_hovered = editor.hovered_socket == Some((node.id, false, i));

            // Check if this input has a connection
            let has_connection = patch.connections.iter().any(|c| c.to_node == node.id && c.to_input == i);

            let socket_color = if is_socket_hovered {
                Color32::from_rgb(200, 200, 220)
            } else if has_connection {
                Color32::from_rgb(100, 120, 140) // Slightly highlighted when connected
            } else {
                Color32::from_rgb(80, 80, 90)
            };

            let radius = if is_socket_hovered { 7.0 * editor.zoom } else { 5.0 * editor.zoom };
            painter.circle_filled(pos, radius, socket_color);
            painter.circle_stroke(pos, radius, Stroke::new(1.0, Color32::from_rgb(120, 120, 130)));
        }
    }

    // Draw output socket
    if node.node_type.has_outputs() {
        let pos = editor.socket_pos(node, true, 0, canvas_rect);
        let is_socket_hovered = editor.hovered_socket == Some((node.id, true, 0));

        let socket_color = if is_socket_hovered {
            Color32::from_rgb(
                (base_color.r() as u16 + 60).min(255) as u8,
                (base_color.g() as u16 + 60).min(255) as u8,
                (base_color.b() as u16 + 60).min(255) as u8,
            )
        } else {
            base_color
        };

        let radius = if is_socket_hovered { 7.0 * editor.zoom } else { 5.0 * editor.zoom };
        painter.circle_filled(pos, radius, socket_color);
        painter.circle_stroke(pos, radius, Stroke::new(1.0, Color32::WHITE));
    }
}

/// Render the node palette (left panel)
pub fn render_node_palette(ui: &mut egui::Ui, editor: &mut NodeEditor, patch: &mut SignalPatch) {
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
                        editor.graph_modified = true;
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
pub fn render_inspector(ui: &mut egui::Ui, editor: &mut NodeEditor, patch: &mut SignalPatch) {
    ui.heading("Inspector");
    ui.separator();

    if let Some(node_id) = editor.selected_node {
        // Get node info first (read-only)
        let node_info = patch.get_node(node_id).map(|n| {
            (n.node_type.display_name().to_string(),
             n.node_type.category().to_string(),
             n.node_type.input_count(),
             n.node_type.has_outputs(),
             n.params.iter().map(|(k, v)| (k.clone(), *v)).collect::<Vec<_>>())
        });

        if let Some((name, category, input_count, has_output, params_clone)) = node_info {
            ui.horizontal(|ui| {
                ui.strong(&name);
                ui.label(format!("({})", category));
            });
            ui.separator();

            // Parameters section
            if !params_clone.is_empty() {
                ui.label("Parameters:");
                for (key, value) in params_clone {
                    let mut val = value;
                    let range = get_param_range(&key);

                    if ui.add(egui::Slider::new(&mut val, range).text(&key)).changed() {
                        if let Some(node) = patch.get_node_mut(node_id) {
                            node.params.insert(key, val);
                            editor.graph_modified = true;
                        }
                    }
                }
                ui.separator();
            }

            // Connections section
            ui.label("Connections:");

            // Show input connections
            if input_count > 0 {
                let input_connections: Vec<_> = patch.connections.iter()
                    .filter(|c| c.to_node == node_id)
                    .map(|c| (c.to_input, c.from_node))
                    .collect();

                for input_idx in 0..input_count {
                    ui.horizontal(|ui| {
                        ui.label(format!("  Input {}:", input_idx));
                        if let Some((_, from_id)) = input_connections.iter().find(|(idx, _)| *idx == input_idx) {
                            if let Some(from_node) = patch.get_node(*from_id) {
                                ui.label(from_node.node_type.display_name());
                                if ui.small_button("×").on_hover_text("Disconnect").clicked() {
                                    patch.disconnect(node_id, input_idx);
                                    editor.graph_modified = true;
                                }
                            }
                        } else {
                            ui.weak("(none)");
                        }
                    });
                }
            }

            // Show output connections
            if has_output {
                let output_connections: Vec<_> = patch.connections.iter()
                    .filter(|c| c.from_node == node_id)
                    .map(|c| c.to_node)
                    .collect();

                ui.horizontal(|ui| {
                    ui.label("  Output:");
                    if output_connections.is_empty() {
                        ui.weak("(none)");
                    } else {
                        let names: Vec<_> = output_connections.iter()
                            .filter_map(|id| patch.get_node(*id).map(|n| n.node_type.display_name()))
                            .collect();
                        ui.label(names.join(", "));
                    }
                });
            }
        }
    } else {
        ui.weak("No node selected");
        ui.separator();
        ui.label("Keyboard shortcuts:");
        ui.label("  Del - Delete selected node");
        ui.label("  F - Frame selected node");
        ui.label("  Home - Reset view");
        ui.separator();
        ui.label("Mouse:");
        ui.label("  Drag output → input: Connect");
        ui.label("  Drag input: Unplug & reconnect");
        ui.label("  Right-click cable: Delete");
        ui.label("  Right-click node: Delete");
        ui.label("  Middle-drag: Pan");
        ui.label("  Scroll: Zoom");
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
