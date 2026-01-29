//! Node Graph Editor
//!
//! Visual node-based compositor interface for device routing.
//! Adapted from v1 for v2's ShaderDevice / DevicePatch architecture.

use std::collections::HashMap;
use std::time::Instant;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use crate::gpu::{DeviceInstance, DevicePatch, CableType};
use super::device_registry;

// ─── Aspect Ratio ─────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AspectRatio {
    Free,
    Square,
    Standard,   // 4:3
    Wide,       // 16:9
    Portrait43,
    Portrait169,
    Cinemascope,
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

    pub fn fit_size(&self, available: Vec2) -> Vec2 {
        match self.ratio() {
            None => available,
            Some(target_ratio) => {
                let available_ratio = available.x / available.y;
                if available_ratio > target_ratio {
                    Vec2::new(available.y * target_ratio, available.y)
                } else {
                    Vec2::new(available.x, available.x / target_ratio)
                }
            }
        }
    }
}

// ─── Connection Drag ──────────────────────────────────────────

#[derive(Clone, Debug)]
#[allow(dead_code)]
enum ConnectionDragMode {
    FromOutput { instance_id: u32, port_idx: usize },
    FromInput {
        original_from_id: u32,
        original_from_port: String,
        to_id: u32,
        to_port: String,
    },
}

// ─── LFO State ───────────────────────────────────────────────

#[derive(Clone)]
pub struct LfoState {
    pub speed: f32, // Hz
    pub lo: f32,    // lower bound of oscillation
    pub hi: f32,    // upper bound of oscillation
}

// ─── Node Editor State ───────────────────────────────────────

pub struct NodeEditor {
    pub pan: Vec2,
    pub zoom: f32,
    pub selected_node: Option<u32>,
    pub selected_connection: Option<usize>,
    dragging_node: Option<u32>,
    drag_offset: Vec2,
    pending_connection: Option<(ConnectionDragMode, Pos2)>,
    is_panning: bool,
    last_mouse_pos: Option<Pos2>,
    pub aspect_ratio: AspectRatio,
    hovered_node: Option<u32>,
    hovered_socket: Option<(u32, bool, usize)>, // (instance_id, is_output, port_index)
    hovered_connection: Option<usize>,
    pub graph_modified: bool,
    pub params_modified: bool,
    pub lfo_active: HashMap<(u32, String), LfoState>,
    lfo_start: Instant,
}

impl Default for NodeEditor {
    fn default() -> Self {
        Self {
            pan: Vec2::new(100.0, 100.0),
            zoom: 1.0,
            selected_node: None,
            selected_connection: None,
            dragging_node: None,
            drag_offset: Vec2::ZERO,
            pending_connection: None,
            is_panning: false,
            last_mouse_pos: None,
            aspect_ratio: AspectRatio::Standard,
            hovered_node: None,
            hovered_socket: None,
            hovered_connection: None,
            graph_modified: false,
            params_modified: false,
            lfo_active: HashMap::new(),
            lfo_start: Instant::now(),
        }
    }
}

impl NodeEditor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn lfo_time(&self) -> f64 {
        self.lfo_start.elapsed().as_secs_f64()
    }

    pub fn apply_lfos(&self, patch: &mut DevicePatch, time_secs: f64) -> bool {
        if self.lfo_active.is_empty() {
            return false;
        }
        let mut changed = false;
        for ((instance_id, param_name), lfo) in &self.lfo_active {
            if let Some(device) = patch.get_device_mut(*instance_id) {
                let center = (lfo.lo + lfo.hi) / 2.0;
                let range = (lfo.hi - lfo.lo) / 2.0;
                let val = center
                    + range
                        * (time_secs as f32 * lfo.speed * std::f32::consts::TAU).sin();
                let val = val.clamp(lfo.lo, lfo.hi);
                device.params.insert(param_name.clone(), val);
                changed = true;
            }
        }
        changed
    }

    fn screen_to_canvas(&self, screen_pos: Pos2, canvas_rect: Rect) -> Pos2 {
        let relative = screen_pos - canvas_rect.min;
        Pos2::new(
            (relative.x - self.pan.x) / self.zoom,
            (relative.y - self.pan.y) / self.zoom,
        )
    }

    fn canvas_to_screen(&self, canvas_pos: Pos2, canvas_rect: Rect) -> Pos2 {
        Pos2::new(
            canvas_pos.x * self.zoom + self.pan.x + canvas_rect.min.x,
            canvas_pos.y * self.zoom + self.pan.y + canvas_rect.min.y,
        )
    }

    fn node_rect(&self, device: &DeviceInstance, canvas_rect: Rect) -> Rect {
        let pos = self.canvas_to_screen(
            Pos2::new(device.position.0, device.position.1),
            canvas_rect,
        );
        let port_count = device.device.inputs().len().max(device.device.outputs().len()).max(1);
        let height = (20.0 + port_count as f32 * 20.0) * self.zoom;
        let width = 130.0 * self.zoom;
        Rect::from_min_size(pos, Vec2::new(width, height))
    }

    fn socket_pos(&self, device: &DeviceInstance, is_output: bool, index: usize, canvas_rect: Rect) -> Pos2 {
        let rect = self.node_rect(device, canvas_rect);
        let port_count = if is_output {
            device.device.outputs().len()
        } else {
            device.device.inputs().len()
        };
        let spacing = (rect.height() - 20.0 * self.zoom) / port_count.max(1) as f32;
        let y = rect.min.y + 20.0 * self.zoom + spacing * (index as f32 + 0.5);

        if is_output {
            Pos2::new(rect.max.x, y)
        } else {
            Pos2::new(rect.min.x, y)
        }
    }

    fn socket_radius(&self) -> f32 {
        10.0 * self.zoom
    }

    fn point_near_bezier(&self, point: Pos2, start: Pos2, end: Pos2, threshold: f32) -> bool {
        let control_offset = ((end.x - start.x).abs() * 0.5).max(50.0 * self.zoom);
        let ctrl1 = Pos2::new(start.x + control_offset, start.y);
        let ctrl2 = Pos2::new(end.x - control_offset, end.y);

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

// ─── Port helpers ─────────────────────────────────────────────

fn port_name_to_index(device: &DeviceInstance, is_output: bool, port_name: &str) -> Option<usize> {
    let ports = if is_output { device.device.outputs() } else { device.device.inputs() };
    ports.iter().position(|p| p.name == port_name)
}

fn port_index_to_name(device: &DeviceInstance, is_output: bool, index: usize) -> Option<String> {
    let ports = if is_output { device.device.outputs() } else { device.device.inputs() };
    ports.get(index).map(|p| p.name.clone())
}

// ─── Node coloring ────────────────────────────────────────────

fn node_color(device_id: &str) -> Color32 {
    if device_id.starts_with("test_") {
        Color32::from_rgb(80, 140, 80)        // Green — generators
    } else if device_id.starts_with("paik_abe_") {
        Color32::from_rgb(160, 110, 60)        // Warm orange — Paik/Abe
    } else if device_id.starts_with("sandin_") {
        Color32::from_rgb(80, 100, 140)        // Blue — Sandin
    } else if device_id.starts_with("jones_") {
        Color32::from_rgb(140, 80, 140)        // Purple — Jones
    } else {
        Color32::from_rgb(100, 100, 100)       // Gray — default
    }
}

fn is_output_device(patch: &DevicePatch, instance_id: u32) -> bool {
    patch.output_device == instance_id
}

/// Color for cable type (used in both connection drawing and inspector)
fn cable_type_color(cable_type: &CableType) -> Color32 {
    match cable_type {
        CableType::Ideal => Color32::from_rgb(120, 120, 130),     // Gray - digital/ideal
        CableType::Composite => Color32::from_rgb(220, 180, 50),  // Yellow - RCA composite
        CableType::SVideo => Color32::from_rgb(80, 80, 80),       // Dark gray - S-Video
        CableType::Component => Color32::from_rgb(60, 180, 60),   // Green - component
        CableType::Rgb => Color32::from_rgb(200, 80, 80),         // Red - RGB
    }
}

fn cable_type_name(cable_type: &CableType) -> &'static str {
    match cable_type {
        CableType::Ideal => "Ideal (No Degradation)",
        CableType::Composite => "Composite (RCA)",
        CableType::SVideo => "S-Video (Y/C)",
        CableType::Component => "Component (YPbPr)",
        CableType::Rgb => "RGB (Direct)",
    }
}

// ─── Canvas rendering ─────────────────────────────────────────

pub fn render_node_canvas(
    ui: &mut egui::Ui,
    editor: &mut NodeEditor,
    patch: &mut DevicePatch,
) {
    let canvas_rect = ui.available_rect_before_wrap();
    let response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());
    let painter = ui.painter_at(canvas_rect);

    draw_grid(&painter, canvas_rect, editor.pan, editor.zoom);
    handle_canvas_input(ui, &response, editor, patch, canvas_rect);
    draw_connections(&painter, editor, patch, canvas_rect);

    // Draw pending connection
    if let Some((ref mode, end_pos)) = editor.pending_connection {
        let (from_id, from_port_idx) = match mode {
            ConnectionDragMode::FromOutput { instance_id, port_idx } => (*instance_id, *port_idx),
            ConnectionDragMode::FromInput { original_from_id, original_from_port, .. } => {
                let idx = patch.get_device(*original_from_id)
                    .and_then(|d| port_name_to_index(d, true, original_from_port))
                    .unwrap_or(0);
                (*original_from_id, idx)
            }
        };
        if let Some(device) = patch.get_device(from_id) {
            let start = editor.socket_pos(device, true, from_port_idx, canvas_rect);
            draw_bezier_connection(&painter, start, end_pos, Color32::from_rgb(255, 220, 100), editor.zoom, true);
        }
    }

    // Draw nodes
    let instance_ids: Vec<u32> = patch.devices.iter().map(|d| d.instance_id).collect();
    for id in &instance_ids {
        if let Some(device) = patch.get_device(*id) {
            draw_node(&painter, editor, device, canvas_rect, patch);
        }
    }

    // Zoom indicator
    painter.text(
        canvas_rect.min + Vec2::new(10.0, 10.0),
        egui::Align2::LEFT_TOP,
        format!("Zoom: {:.0}%", editor.zoom * 100.0),
        egui::FontId::proportional(12.0),
        Color32::from_rgb(150, 150, 150),
    );

    // Contextual hint
    let hint = if editor.hovered_connection.is_some() {
        "Right-click to delete connection"
    } else if let Some((_, is_output, _)) = editor.hovered_socket {
        if is_output { "Drag to connect" } else { "Drag to unplug" }
    } else if editor.hovered_node.is_some() {
        "Drag to move \u{2022} Right-click to delete \u{2022} Del key"
    } else {
        "Middle-drag to pan \u{2022} Scroll to zoom \u{2022} Home to reset"
    };
    painter.text(
        Pos2::new(canvas_rect.min.x + 10.0, canvas_rect.max.y - 20.0),
        egui::Align2::LEFT_TOP,
        hint,
        egui::FontId::proportional(11.0),
        Color32::from_rgb(120, 120, 130),
    );
}

// ─── Grid ─────────────────────────────────────────────────────

fn draw_grid(painter: &egui::Painter, rect: Rect, pan: Vec2, zoom: f32) {
    let grid_spacing = 50.0 * zoom;
    let grid_color = Color32::from_rgb(40, 40, 45);
    let major_grid_color = Color32::from_rgb(50, 50, 55);

    let offset_x = pan.x % grid_spacing;
    let offset_y = pan.y % grid_spacing;

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

// ─── Input handling ───────────────────────────────────────────

fn handle_canvas_input(
    ui: &egui::Ui,
    response: &egui::Response,
    editor: &mut NodeEditor,
    patch: &mut DevicePatch,
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

    // Scroll to zoom
    if response.hovered() {
        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_delta != 0.0 {
            if let Some(pos) = pointer_pos {
                let old_zoom = editor.zoom;
                let zoom_delta = scroll_delta * 0.001;
                editor.zoom = (editor.zoom + zoom_delta).clamp(0.25, 2.0);
                let zoom_ratio = editor.zoom / old_zoom;
                let relative = pos - canvas_rect.min;
                editor.pan = relative - (relative - editor.pan) * zoom_ratio;
            }
        }
    }

    // Middle mouse / Alt+Left to pan
    let is_panning = ui.input(|i| i.pointer.middle_down() || (i.modifiers.alt && i.pointer.primary_down()));
    if is_panning {
        if let Some(pos) = pointer_pos {
            if let Some(last) = editor.last_mouse_pos {
                editor.pan += pos - last;
            }
            editor.last_mouse_pos = Some(pos);
        }
        editor.is_panning = true;
        return;
    } else {
        editor.is_panning = false;
    }
    editor.last_mouse_pos = pointer_pos;

    // Hover detection
    if let Some(pos) = pointer_pos {
        if response.hovered() {
            // Check socket hover
            let ids: Vec<u32> = patch.devices.iter().map(|d| d.instance_id).collect();
            'socket_check: for id in &ids {
                if let Some(device) = patch.get_device(*id) {
                    let outputs = device.device.outputs();
                    for (oi, _) in outputs.iter().enumerate() {
                        let sp = editor.socket_pos(device, true, oi, canvas_rect);
                        if (pos - sp).length() < editor.socket_radius() {
                            editor.hovered_socket = Some((*id, true, oi));
                            break 'socket_check;
                        }
                    }
                    let inputs = device.device.inputs();
                    for (ii, _) in inputs.iter().enumerate() {
                        let sp = editor.socket_pos(device, false, ii, canvas_rect);
                        if (pos - sp).length() < editor.socket_radius() {
                            editor.hovered_socket = Some((*id, false, ii));
                            break 'socket_check;
                        }
                    }
                }
            }

            // Check connection hover
            if editor.hovered_socket.is_none() {
                for (idx, conn) in patch.connections.iter().enumerate() {
                    let from_dev = patch.get_device(conn.from_device);
                    let to_dev = patch.get_device(conn.to_device);
                    if let (Some(from), Some(to)) = (from_dev, to_dev) {
                        let from_idx = port_name_to_index(from, true, &conn.from_port).unwrap_or(0);
                        let to_idx = port_name_to_index(to, false, &conn.to_port).unwrap_or(0);
                        let start = editor.socket_pos(from, true, from_idx, canvas_rect);
                        let end = editor.socket_pos(to, false, to_idx, canvas_rect);
                        // Increased threshold for easier selection
                        if editor.point_near_bezier(pos, start, end, 10.0 * editor.zoom) {
                            editor.hovered_connection = Some(idx);
                            break;
                        }
                    }
                }
            }

            // Check node hover
            if editor.hovered_socket.is_none() && editor.hovered_connection.is_none() {
                for device in &patch.devices {
                    let rect = editor.node_rect(device, canvas_rect);
                    if rect.contains(pos) {
                        editor.hovered_node = Some(device.instance_id);
                        break;
                    }
                }
            }
        }
    }

    // Keyboard shortcuts
    if response.hovered() {
        let delete_pressed = ui.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace));
        if delete_pressed {
            if let Some(node_id) = editor.selected_node {
                if !is_output_device(patch, node_id) {
                    patch.remove_device(node_id);
                    editor.selected_node = None;
                    editor.graph_modified = true;
                }
            }
        }

        if ui.input(|i| i.key_pressed(egui::Key::Home)) {
            editor.pan = Vec2::new(100.0, 100.0);
            editor.zoom = 1.0;
        }

        if ui.input(|i| i.key_pressed(egui::Key::F)) {
            if let Some(node_id) = editor.selected_node {
                if let Some(device) = patch.get_device(node_id) {
                    let center = canvas_rect.center();
                    editor.pan = Vec2::new(
                        center.x - canvas_rect.min.x - device.position.0 * editor.zoom,
                        center.y - canvas_rect.min.y - device.position.1 * editor.zoom,
                    );
                }
            }
        }
    }

    // Mouse press — start interactions
    if let Some(pos) = pointer_pos {
        if primary_pressed && response.hovered() {
            let mut started_interaction = false;

            // Check output socket drag
            let ids: Vec<u32> = patch.devices.iter().map(|d| d.instance_id).collect();
            for id in &ids {
                if started_interaction { break; }
                if let Some(device) = patch.get_device(*id) {
                    let outputs = device.device.outputs();
                    for (oi, _) in outputs.iter().enumerate() {
                        let sp = editor.socket_pos(device, true, oi, canvas_rect);
                        if (pos - sp).length() < editor.socket_radius() {
                            editor.pending_connection = Some((
                                ConnectionDragMode::FromOutput { instance_id: *id, port_idx: oi },
                                pos,
                            ));
                            started_interaction = true;
                            break;
                        }
                    }
                }
            }

            // Check input socket drag (unplug)
            if !started_interaction {
                let mut unplug_info: Option<(u32, String, u32, String)> = None;
                for id in &ids {
                    if unplug_info.is_some() { break; }
                    if let Some(device) = patch.get_device(*id) {
                        let inputs = device.device.inputs();
                        for (ii, inp) in inputs.iter().enumerate() {
                            let sp = editor.socket_pos(device, false, ii, canvas_rect);
                            if (pos - sp).length() < editor.socket_radius() {
                                // Find connection to this input
                                if let Some(conn) = patch.connections.iter().find(
                                    |c| c.to_device == *id && c.to_port == inp.name
                                ) {
                                    unplug_info = Some((
                                        conn.from_device,
                                        conn.from_port.clone(),
                                        *id,
                                        inp.name.clone(),
                                    ));
                                }
                                break;
                            }
                        }
                    }
                }

                if let Some((from_id, from_port, to_id, to_port)) = unplug_info {
                    editor.pending_connection = Some((
                        ConnectionDragMode::FromInput {
                            original_from_id: from_id,
                            original_from_port: from_port,
                            to_id,
                            to_port: to_port.clone(),
                        },
                        pos,
                    ));
                    patch.disconnect(to_id, &to_port);
                    editor.graph_modified = true;
                    started_interaction = true;
                }
            }

            // Check connection selection
            if !started_interaction {
                if let Some(conn_idx) = editor.hovered_connection {
                    eprintln!("[DEBUG] Clicked on connection: idx={}", conn_idx);
                    editor.selected_connection = Some(conn_idx);
                    editor.selected_node = None;
                    started_interaction = true;
                }
            }

            // Check node selection/drag
            if !started_interaction {
                let mut found_node = false;
                for device in &patch.devices {
                    let rect = editor.node_rect(device, canvas_rect);
                    if rect.contains(pos) {
                        let node_screen = editor.canvas_to_screen(
                            Pos2::new(device.position.0, device.position.1),
                            canvas_rect,
                        );
                        editor.drag_offset = pos - node_screen;
                        editor.dragging_node = Some(device.instance_id);
                        editor.selected_node = Some(device.instance_id);
                        editor.selected_connection = None;
                        found_node = true;
                        break;
                    }
                }
                if !found_node {
                    editor.selected_node = None;
                    editor.selected_connection = None;
                }
            }
        }

        // Dragging
        if primary_down {
            if let Some(node_id) = editor.dragging_node {
                let canvas_pos = editor.screen_to_canvas(pos - editor.drag_offset, canvas_rect);
                if let Some(device) = patch.get_device_mut(node_id) {
                    device.position.0 = canvas_pos.x;
                    device.position.1 = canvas_pos.y;
                }
            }
            if let Some((_, ref mut end_pos)) = editor.pending_connection {
                *end_pos = pos;
            }
        }

        // Mouse release — complete connection
        if primary_released {
            if let Some((mode, _)) = editor.pending_connection.take() {
                let (from_id, from_port_idx) = match &mode {
                    ConnectionDragMode::FromOutput { instance_id, port_idx } => (*instance_id, *port_idx),
                    ConnectionDragMode::FromInput { original_from_id, original_from_port, .. } => {
                        let idx = patch.get_device(*original_from_id)
                            .and_then(|d| port_name_to_index(d, true, original_from_port))
                            .unwrap_or(0);
                        (*original_from_id, idx)
                    }
                };

                // Find target input socket
                let mut target: Option<(u32, String)> = None;
                let ids: Vec<u32> = patch.devices.iter().map(|d| d.instance_id).collect();
                for id in &ids {
                    if *id == from_id { continue; }
                    if target.is_some() { break; }
                    if let Some(device) = patch.get_device(*id) {
                        let inputs = device.device.inputs();
                        for (ii, inp) in inputs.iter().enumerate() {
                            let sp = editor.socket_pos(device, false, ii, canvas_rect);
                            if (pos - sp).length() < editor.socket_radius() + 2.0 {
                                target = Some((*id, inp.name.clone()));
                                break;
                            }
                        }
                    }
                }

                if let Some((to_id, to_port)) = target {
                    if let Some(from_port_name) = patch.get_device(from_id)
                        .and_then(|d| port_index_to_name(d, true, from_port_idx))
                    {
                        patch.connect(from_id, &from_port_name, to_id, &to_port);
                        editor.graph_modified = true;
                    }
                }
            }
            editor.dragging_node = None;
            editor.drag_offset = Vec2::ZERO;
        }
    }

    // Right-click actions
    if response.secondary_clicked() {
        if let Some(_pos) = pointer_pos {
            if let Some(conn_idx) = editor.hovered_connection {
                if conn_idx < patch.connections.len() {
                    patch.connections.remove(conn_idx);
                    editor.graph_modified = true;
                    return;
                }
            }

            if let Some(hovered_id) = editor.hovered_node {
                if !is_output_device(patch, hovered_id) {
                    patch.remove_device(hovered_id);
                    editor.graph_modified = true;
                    if editor.selected_node == Some(hovered_id) {
                        editor.selected_node = None;
                    }
                }
            }
        }
    }
}

// ─── Drawing ──────────────────────────────────────────────────

fn draw_connections(
    painter: &egui::Painter,
    editor: &NodeEditor,
    patch: &DevicePatch,
    canvas_rect: Rect,
) {
    for (idx, conn) in patch.connections.iter().enumerate() {
        let from_dev = patch.get_device(conn.from_device);
        let to_dev = patch.get_device(conn.to_device);
        if let (Some(from), Some(to)) = (from_dev, to_dev) {
            let from_idx = port_name_to_index(from, true, &conn.from_port).unwrap_or(0);
            let to_idx = port_name_to_index(to, false, &conn.to_port).unwrap_or(0);
            let start = editor.socket_pos(from, true, from_idx, canvas_rect);
            let end = editor.socket_pos(to, false, to_idx, canvas_rect);

            let is_hovered = editor.hovered_connection == Some(idx);
            let is_selected = editor.selected_connection == Some(idx);
            let color = if is_hovered {
                Color32::from_rgb(255, 100, 100)
            } else if is_selected {
                Color32::WHITE
            } else {
                cable_type_color(&conn.cable_type)
            };
            draw_bezier_connection(painter, start, end, color, editor.zoom, is_hovered || is_selected);
        }
    }
}

fn draw_bezier_connection(
    painter: &egui::Painter,
    start: Pos2,
    end: Pos2,
    color: Color32,
    zoom: f32,
    highlighted: bool,
) {
    let control_offset = ((end.x - start.x).abs() * 0.5).max(50.0 * zoom);
    let ctrl1 = Pos2::new(start.x + control_offset, start.y);
    let ctrl2 = Pos2::new(end.x - control_offset, end.y);

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

fn draw_node(
    painter: &egui::Painter,
    editor: &NodeEditor,
    device: &DeviceInstance,
    canvas_rect: Rect,
    patch: &DevicePatch,
) {
    let rect = editor.node_rect(device, canvas_rect);
    let is_selected = editor.selected_node == Some(device.instance_id);
    let is_hovered = editor.hovered_node == Some(device.instance_id);
    let is_output = is_output_device(patch, device.instance_id);

    let base_color = node_color(device.device.device_id());
    let header_color = if is_hovered && !is_selected {
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

    // Border
    if is_selected {
        painter.rect_stroke(rect, 4.0 * editor.zoom, Stroke::new(2.0, Color32::WHITE));
    } else if is_output {
        painter.rect_stroke(rect, 4.0 * editor.zoom, Stroke::new(1.5, Color32::from_rgb(255, 200, 80)));
    } else if is_hovered {
        painter.rect_stroke(rect, 4.0 * editor.zoom, Stroke::new(1.5, Color32::from_rgb(150, 150, 160)));
    } else {
        painter.rect_stroke(rect, 4.0 * editor.zoom, Stroke::new(1.0, Color32::from_rgb(60, 60, 65)));
    }

    // Title
    let title = if is_output {
        format!("{} [OUT]", device.device.device_name())
    } else {
        device.device.device_name().to_string()
    };
    painter.text(
        rect.min + Vec2::new(8.0 * editor.zoom, 4.0 * editor.zoom),
        egui::Align2::LEFT_TOP,
        &title,
        egui::FontId::proportional(11.0 * editor.zoom),
        Color32::WHITE,
    );

    // Input sockets with labels
    let inputs = device.device.inputs();
    for (i, inp) in inputs.iter().enumerate() {
        let sp = editor.socket_pos(device, false, i, canvas_rect);
        let is_socket_hovered = editor.hovered_socket == Some((device.instance_id, false, i));
        let has_connection = patch.connections.iter().any(
            |c| c.to_device == device.instance_id && c.to_port == inp.name
        );
        let socket_color = if is_socket_hovered {
            Color32::from_rgb(200, 200, 220)
        } else if has_connection {
            Color32::from_rgb(100, 120, 140)
        } else {
            Color32::from_rgb(80, 80, 90)
        };
        let radius = if is_socket_hovered { 7.0 * editor.zoom } else { 5.0 * editor.zoom };
        painter.circle_filled(sp, radius, socket_color);
        painter.circle_stroke(sp, radius, Stroke::new(1.0, Color32::from_rgb(120, 120, 130)));

        // Port label
        painter.text(
            sp + Vec2::new(10.0 * editor.zoom, -5.0 * editor.zoom),
            egui::Align2::LEFT_TOP,
            &inp.name,
            egui::FontId::proportional(9.0 * editor.zoom),
            Color32::from_rgb(160, 160, 170),
        );
    }

    // Output sockets with labels
    let outputs = device.device.outputs();
    for (i, out) in outputs.iter().enumerate() {
        let sp = editor.socket_pos(device, true, i, canvas_rect);
        let is_socket_hovered = editor.hovered_socket == Some((device.instance_id, true, i));
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
        painter.circle_filled(sp, radius, socket_color);
        painter.circle_stroke(sp, radius, Stroke::new(1.0, Color32::WHITE));

        // Port label
        painter.text(
            sp + Vec2::new(-10.0 * editor.zoom, -5.0 * editor.zoom),
            egui::Align2::RIGHT_TOP,
            &out.name,
            egui::FontId::proportional(9.0 * editor.zoom),
            Color32::from_rgb(160, 160, 170),
        );
    }
}

// ─── Palette (left panel) ─────────────────────────────────────

pub fn render_node_palette(ui: &mut egui::Ui, editor: &mut NodeEditor, patch: &mut DevicePatch) {
    ui.heading("Devices");
    ui.separator();

    for category in device_registry::device_categories() {
        egui::CollapsingHeader::new(category.name)
            .default_open(category.name == "Generators")
            .show(ui, |ui| {
                for entry in &category.devices {
                    if ui.button(entry.display_name).clicked() {
                        if let Some(device) = device_registry::create_device(entry.device_id) {
                            patch.add_device(device, 200.0, 200.0);
                            editor.graph_modified = true;
                        }
                    }
                }
            });
    }
}

// ─── Inspector (right panel bottom) ───────────────────────────

pub fn render_inspector(ui: &mut egui::Ui, editor: &mut NodeEditor, patch: &mut DevicePatch) {
    ui.heading("Inspector");
    ui.separator();

    // Connection inspector (cable type selector)
    if let Some(conn_idx) = editor.selected_connection {
        eprintln!("[DEBUG] Connection selected: idx={}", conn_idx);
        if conn_idx < patch.connections.len() {
            let conn = &patch.connections[conn_idx];
            let from_name = patch.get_device(conn.from_device)
                .map(|d| d.device.device_name().to_string())
                .unwrap_or_else(|| "?".to_string());
            let to_name = patch.get_device(conn.to_device)
                .map(|d| d.device.device_name().to_string())
                .unwrap_or_else(|| "?".to_string());
            let current_cable = conn.cable_type.clone();
            eprintln!("[DEBUG] Showing connection: {} -> {}, cable={:?}", from_name, to_name, current_cable);

            ui.strong("Connection");
            ui.horizontal(|ui| {
                ui.label(&from_name);
                ui.label("\u{2192}");
                ui.label(&to_name);
            });
            ui.separator();

            ui.label("Cable Type:");
            let cable_types = [
                CableType::Ideal,
                CableType::Composite,
                CableType::SVideo,
                CableType::Component,
                CableType::Rgb,
            ];

            for cable in &cable_types {
                let is_selected = current_cable == *cable;
                let color = cable_type_color(cable);
                let name = cable_type_name(cable);

                ui.horizontal(|ui| {
                    // Color indicator
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, color);

                    if ui.selectable_label(is_selected, name).clicked() {
                        if let Some(conn) = patch.connections.get_mut(conn_idx) {
                            eprintln!("[CABLE] CHANGING cable type from {:?} to {:?}", conn.cable_type, cable);
                            conn.cable_type = cable.clone();
                            editor.graph_modified = true;
                            eprintln!("[CABLE] graph_modified = true");
                        } else {
                            eprintln!("[CABLE] ERROR: get_mut returned None!");
                        }
                    }
                });
            }

            ui.separator();
            ui.weak("Cable type affects signal quality:");
            ui.weak("• Composite: dot crawl, chroma blur");
            ui.weak("• S-Video: no crosstalk, cleaner");
            ui.weak("• Component/RGB: minimal degradation");

            return;
        } else {
            // Connection was deleted
            editor.selected_connection = None;
        }
    }

    if let Some(node_id) = editor.selected_node {
        // Gather info first (can't borrow patch mutably while reading)
        let info = patch.get_device(node_id).map(|d| {
            let name = d.device.device_name().to_string();
            let device_id = d.device.device_id().to_string();
            let params = d.device.parameters();
            let inputs = d.device.inputs();
            let outputs = d.device.outputs();
            let current_values: Vec<(String, f32)> = d.params.iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect();
            let is_out = is_output_device(patch, node_id);
            (name, device_id, params, inputs, outputs, current_values, is_out)
        });

        if let Some((name, _device_id, params, inputs, outputs, current_values, is_out)) = info {
            ui.horizontal(|ui| {
                ui.strong(&name);
                if is_out {
                    ui.label("[Output Device]");
                }
            });

            // "Set as output" button
            if !is_out {
                if ui.small_button("Set as Output").clicked() {
                    patch.output_device = node_id;
                    editor.graph_modified = true;
                }
            }

            ui.separator();

            // Parameters
            if !params.is_empty() {
                ui.label("Parameters:");
                for param in &params {
                    let current = current_values.iter()
                        .find(|(k, _)| k == &param.name)
                        .map(|(_, v)| *v)
                        .unwrap_or(param.default);

                    let lfo_key = (node_id, param.name.clone());
                    let lfo_state = editor.lfo_active.get(&lfo_key).cloned();
                    let lfo_speed = lfo_state.as_ref().map(|s| s.speed);
                    let mut val = current;
                    let label = if param.unit.is_empty() {
                        param.display_name.clone()
                    } else {
                        format!("{} ({})", param.display_name, param.unit)
                    };

                    // Slider + LFO cycle button (Off → Slow → Medium → Fast → Off)
                    // Left click cycles speed, right click stops immediately
                    let mut cycle_lfo = false;
                    let mut stop_lfo = false;
                    let mut slider_changed = false;
                    ui.horizontal(|ui| {
                        let (btn_label, btn_color, hover) = match lfo_speed {
                            None => ("~", Color32::GRAY, "LFO: Off (click for Slow)"),
                            Some(s) if s < 0.15 => ("S", Color32::from_rgb(80, 160, 80), "LFO: Slow (click for Medium, right-click to stop)"),
                            Some(s) if s < 0.4 => ("M", Color32::from_rgb(180, 180, 60), "LFO: Medium (click for Fast, right-click to stop)"),
                            _ => ("F", Color32::from_rgb(100, 200, 255), "LFO: Fast (click to stop)"),
                        };
                        let btn_response = ui.small_button(egui::RichText::new(btn_label).color(btn_color).strong())
                            .on_hover_text(hover);
                        if btn_response.clicked() {
                            cycle_lfo = true;
                        }
                        if btn_response.secondary_clicked() {
                            stop_lfo = true;
                        }
                        slider_changed = ui.add(
                            egui::Slider::new(&mut val, param.min..=param.max)
                                .text(&label)
                                .drag_value_speed(0.01)
                        ).changed();
                    });

                    if stop_lfo {
                        editor.lfo_active.remove(&lfo_key);
                    } else if cycle_lfo {
                        let lo = lfo_state.as_ref().map(|s| s.lo).unwrap_or(param.min);
                        let hi = lfo_state.as_ref().map(|s| s.hi).unwrap_or(param.max);
                        match lfo_speed {
                            None => { editor.lfo_active.insert(lfo_key.clone(), LfoState { speed: 0.1, lo: param.min, hi: param.max }); }
                            Some(s) if s < 0.15 => { editor.lfo_active.insert(lfo_key.clone(), LfoState { speed: 0.25, lo, hi }); }
                            Some(s) if s < 0.4 => { editor.lfo_active.insert(lfo_key.clone(), LfoState { speed: 0.5, lo, hi }); }
                            _ => { editor.lfo_active.remove(&lfo_key); }
                        }
                    }

                    // Show lo/hi range controls when LFO is active
                    if editor.lfo_active.contains_key(&lfo_key) {
                        ui.horizontal(|ui| {
                            ui.add_space(24.0); // indent past LFO button
                            ui.weak("Range:");
                            let lfo = editor.lfo_active.get_mut(&lfo_key).unwrap();
                            let mut lo = lfo.lo;
                            let mut hi = lfo.hi;
                            ui.add(egui::DragValue::new(&mut lo)
                                .range(param.min..=hi)
                                .speed(0.01)
                                .prefix("lo: "));
                            ui.add(egui::DragValue::new(&mut hi)
                                .range(lo..=param.max)
                                .speed(0.01)
                                .prefix("hi: "));
                            lfo.lo = lo;
                            lfo.hi = hi;
                        });
                    }

                    if slider_changed {
                        if let Some(device) = patch.get_device_mut(node_id) {
                            device.params.insert(param.name.clone(), val);
                            editor.params_modified = true;
                        }
                        // Disable LFO when user manually adjusts
                        if lfo_speed.is_some() {
                            editor.lfo_active.remove(&lfo_key);
                        }
                    }
                }
                ui.separator();
            }

            // Connections info
            ui.label("Connections:");
            for inp in inputs.iter() {
                ui.horizontal(|ui| {
                    ui.label(format!("  {} \u{2190}", inp.name));
                    let conn = patch.connections.iter().find(
                        |c| c.to_device == node_id && c.to_port == inp.name
                    );
                    if let Some(c) = conn {
                        if let Some(from_dev) = patch.get_device(c.from_device) {
                            ui.label(from_dev.device.device_name());
                            if ui.small_button("\u{00D7}").on_hover_text("Disconnect").clicked() {
                                patch.disconnect(node_id, &inp.name);
                                editor.graph_modified = true;
                            }
                        }
                    } else {
                        ui.weak("(none)");
                    }
                });
            }
            for out in outputs.iter() {
                let out_conns: Vec<u32> = patch.connections.iter()
                    .filter(|c| c.from_device == node_id && c.from_port == out.name)
                    .map(|c| c.to_device)
                    .collect();
                ui.horizontal(|ui| {
                    ui.label(format!("  {} \u{2192}", out.name));
                    if out_conns.is_empty() {
                        ui.weak("(none)");
                    } else {
                        let names: Vec<String> = out_conns.iter()
                            .filter_map(|id| patch.get_device(*id).map(|d| d.device.device_name().to_string()))
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
        ui.label("  Drag output \u{2192} input: Connect");
        ui.label("  Drag input: Unplug & reconnect");
        ui.label("  Right-click cable: Delete");
        ui.label("  Right-click node: Delete");
        ui.label("  Middle-drag: Pan");
        ui.label("  Scroll: Zoom");
    }
}
