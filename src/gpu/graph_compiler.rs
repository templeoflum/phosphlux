//! Graph Compiler
//!
//! Compiles the SignalPatch node graph into GPU-executable operations.
//! The shader interprets these operations at runtime to evaluate the graph.
//!
//! Supports cyclic graphs through Frame Buffer nodes which act as cycle-breaking
//! points. Frame Buffers output the previous frame's data, enabling feedback loops.

use crate::ui::patch::{SignalPatch, NodeType};
use std::collections::{HashMap, HashSet};

/// Maximum number of operations the GPU can execute per frame
pub const MAX_OPS: usize = 32;

/// Maximum number of signal slots for intermediate values
pub const MAX_SLOTS: usize = 32;

/// Maximum number of frame buffer nodes supported
pub const MAX_FRAME_BUFFERS: usize = 4;

/// Maximum number of display outputs
pub const MAX_DISPLAYS: usize = 4;

/// Operation codes for the GPU shader
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OpCode {
    // Sources (output to slot, no inputs)
    RampH = 0,
    RampV = 1,
    RampRadial = 2,
    OscillatorH = 3,
    OscillatorV = 4,
    NoiseWhite = 5,
    NoisePink = 6,
    NoiseBrown = 7,
    ShapeCircle = 8,
    ShapeRect = 9,
    ShapeDiamond = 10,

    // Temporal nodes (read from frame buffer texture)
    FrameBufferRead = 11,   // Read previous frame from frame buffer
    DelayLineRead = 12,     // Read from delay line

    // Single-input processors
    Folder = 20,
    Quantizer = 21,
    SoftClip = 22,
    Solarize = 23,
    Inverter = 24,
    HueShift = 25,

    // Colorizers (luma -> RGB)
    ColorSpectrum = 30,
    ColorThreshold = 31,
    ColorGradient = 32,

    // Two-input processors
    LumaKey = 40,
    Mixer = 41,
    Multiplier = 42,
    ChromaKey = 43,

    // Frame buffer write (stores current frame for next iteration)
    FrameBufferWrite = 50,
    DelayLineWrite = 51,

    // Display outputs (render targets)
    DisplayMonitor = 60,    // Clean monitor output
    DisplayCrt = 61,        // CRT with full emulation

    // Legacy output (for transition)
    MainOut = 70,

    // No-op (empty slot)
    Nop = 255,
}

impl OpCode {
    fn from_node_type(node_type: NodeType) -> Self {
        match node_type {
            NodeType::RampH => OpCode::RampH,
            NodeType::RampV => OpCode::RampV,
            NodeType::RampRadial => OpCode::RampRadial,
            NodeType::OscillatorH => OpCode::OscillatorH,
            NodeType::OscillatorV => OpCode::OscillatorV,
            NodeType::NoiseWhite => OpCode::NoiseWhite,
            NodeType::NoisePink => OpCode::NoisePink,
            NodeType::NoiseBrown => OpCode::NoiseBrown,
            NodeType::ShapeCircle => OpCode::ShapeCircle,
            NodeType::ShapeRect => OpCode::ShapeRect,
            NodeType::ShapeDiamond => OpCode::ShapeDiamond,
            NodeType::FrameBuffer => OpCode::FrameBufferRead, // Reading is the default
            NodeType::DelayLine => OpCode::DelayLineRead,
            NodeType::Folder => OpCode::Folder,
            NodeType::Quantizer => OpCode::Quantizer,
            NodeType::SoftClip => OpCode::SoftClip,
            NodeType::Solarize => OpCode::Solarize,
            NodeType::Inverter => OpCode::Inverter,
            NodeType::HueShift => OpCode::HueShift,
            NodeType::ColorSpectrum => OpCode::ColorSpectrum,
            NodeType::ColorThreshold => OpCode::ColorThreshold,
            NodeType::ColorGradient => OpCode::ColorGradient,
            NodeType::LumaKey => OpCode::LumaKey,
            NodeType::Mixer => OpCode::Mixer,
            NodeType::Multiplier => OpCode::Multiplier,
            NodeType::ChromaKey => OpCode::ChromaKey,
            NodeType::Monitor => OpCode::DisplayMonitor,
            NodeType::CrtDisplay => OpCode::DisplayCrt,
            NodeType::MainOut => OpCode::MainOut,
        }
    }
}

/// A single GPU operation
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GraphOp {
    /// Operation code
    pub opcode: u32,
    /// First input slot (or 0xFFFFFFFF if unused)
    pub input_a: u32,
    /// Second input slot (or 0xFFFFFFFF if unused)
    pub input_b: u32,
    /// Output slot
    pub output_slot: u32,
    /// Parameter 0 (operation-specific)
    pub param0: f32,
    /// Parameter 1 (operation-specific)
    pub param1: f32,
    /// Parameter 2 (operation-specific)
    pub param2: f32,
    /// Parameter 3 (operation-specific)
    pub param3: f32,
}

impl Default for GraphOp {
    fn default() -> Self {
        Self {
            opcode: OpCode::Nop as u32,
            input_a: 0xFFFFFFFF,
            input_b: 0xFFFFFFFF,
            output_slot: 0,
            param0: 0.0,
            param1: 0.0,
            param2: 0.0,
            param3: 0.0,
        }
    }
}

/// Frame buffer info for the GPU
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FrameBufferInfo {
    /// Node ID of this frame buffer (for matching to textures)
    pub node_id: u32,
    /// Slot where the read operation outputs
    pub read_slot: u32,
    /// Slot where the write operation reads from
    pub write_slot: u32,
    /// Frame buffer index (0-3)
    pub buffer_index: u32,
    /// Mix parameter (0 = full input, 1 = full previous)
    pub mix: f32,
    /// Zoom per frame
    pub zoom: f32,
    /// Rotation per frame (radians)
    pub rotation: f32,
    /// Hue shift per frame
    pub hue_shift: f32,
    /// Brightness decay
    pub decay: f32,
    /// Saturation multiplier
    pub saturation: f32,
    /// X offset
    pub offset_x: f32,
    /// Y offset
    pub offset_y: f32,
}

/// Display output info
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DisplayInfo {
    /// Node ID of this display
    pub node_id: u32,
    /// Display type (0 = Monitor, 1 = CRT)
    pub display_type: u32,
    /// Input slot
    pub input_slot: u32,
    /// Display index (0-3)
    pub display_index: u32,
    /// CRT params (only used for CRT type)
    pub scanline_intensity: f32,
    pub scanline_count: f32,
    pub curvature: f32,
    pub vignette: f32,
    pub bloom: f32,
    pub phosphor_type: f32,
    pub brightness: f32,
    pub contrast: f32,
}

/// Compiled graph header
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CompiledGraphHeader {
    /// Number of operations to execute
    pub op_count: u32,
    /// Number of frame buffers
    pub frame_buffer_count: u32,
    /// Number of display outputs
    pub display_count: u32,
    /// Slot index containing fallback output (MainOut or first display)
    pub main_out_slot: u32,
}

impl Default for CompiledGraphHeader {
    fn default() -> Self {
        Self {
            op_count: 0,
            frame_buffer_count: 0,
            display_count: 0,
            main_out_slot: 0xFFFFFFFF,
        }
    }
}

/// Compiled graph ready for GPU upload
pub struct CompiledGraph {
    pub header: CompiledGraphHeader,
    pub ops: Vec<GraphOp>,
    pub frame_buffers: Vec<FrameBufferInfo>,
    pub displays: Vec<DisplayInfo>,
    /// Maps node ID to operation index (for parameter-only updates)
    pub node_to_op: HashMap<u32, usize>,
}

impl CompiledGraph {
    /// Create an empty graph (renders black)
    pub fn empty() -> Self {
        Self {
            header: CompiledGraphHeader::default(),
            ops: vec![GraphOp::default(); MAX_OPS],
            frame_buffers: Vec::new(),
            displays: Vec::new(),
            node_to_op: HashMap::new(),
        }
    }

    /// Create a default graph for when no nodes exist
    pub fn default_fallback() -> Self {
        let mut ops = vec![GraphOp::default(); MAX_OPS];

        // Generate radial ramp -> colorize -> output
        ops[0] = GraphOp {
            opcode: OpCode::RampRadial as u32,
            input_a: 0xFFFFFFFF,
            input_b: 0xFFFFFFFF,
            output_slot: 0,
            param0: 0.0,
            param1: 0.0,
            param2: 0.0,
            param3: 0.0,
        };

        ops[1] = GraphOp {
            opcode: OpCode::ColorSpectrum as u32,
            input_a: 0,
            input_b: 0xFFFFFFFF,
            output_slot: 1,
            param0: 0.0,
            param1: 0.0,
            param2: 0.0,
            param3: 0.0,
        };

        ops[2] = GraphOp {
            opcode: OpCode::MainOut as u32,
            input_a: 1,
            input_b: 0xFFFFFFFF,
            output_slot: 2,
            param0: 0.0,
            param1: 0.0,
            param2: 0.0,
            param3: 0.0,
        };

        Self {
            header: CompiledGraphHeader {
                op_count: 3,
                frame_buffer_count: 0,
                display_count: 0,
                main_out_slot: 2,
            },
            ops,
            frame_buffers: Vec::new(),
            displays: Vec::new(),
            node_to_op: HashMap::new(),
        }
    }

    /// Update only the parameters in the ops array from the patch
    /// Returns true if any parameters were updated
    pub fn update_params(&mut self, patch: &SignalPatch) -> bool {
        let mut updated = false;

        for (&node_id, &op_idx) in &self.node_to_op {
            if op_idx >= self.ops.len() {
                continue;
            }

            if let Some(node) = patch.get_node(node_id) {
                let op = &mut self.ops[op_idx];

                // Extract params based on node type
                let (new_param0, new_param1, new_param2, new_param3) =
                    extract_node_params(node);

                // Only mark as updated if values actually changed
                if op.param0 != new_param0 || op.param1 != new_param1 ||
                   op.param2 != new_param2 || op.param3 != new_param3 {
                    op.param0 = new_param0;
                    op.param1 = new_param1;
                    op.param2 = new_param2;
                    op.param3 = new_param3;
                    updated = true;
                }
            }
        }

        // Also update frame buffer params
        for fb in &mut self.frame_buffers {
            if let Some(node) = patch.get_node(fb.node_id) {
                fb.mix = node.params.get("mix").copied().unwrap_or(0.5);
                fb.zoom = node.params.get("zoom").copied().unwrap_or(1.0);
                fb.rotation = node.params.get("rotation").copied().unwrap_or(0.0);
                fb.hue_shift = node.params.get("hue_shift").copied().unwrap_or(0.0);
                fb.decay = node.params.get("decay").copied().unwrap_or(0.98);
                fb.saturation = node.params.get("saturation").copied().unwrap_or(1.0);
                fb.offset_x = node.params.get("offset_x").copied().unwrap_or(0.0);
                fb.offset_y = node.params.get("offset_y").copied().unwrap_or(0.0);
                updated = true;
            }
        }

        // Update display params
        for disp in &mut self.displays {
            if let Some(node) = patch.get_node(disp.node_id) {
                disp.brightness = node.params.get("brightness").copied().unwrap_or(1.0);
                disp.contrast = node.params.get("contrast").copied().unwrap_or(1.0);
                if disp.display_type == 1 { // CRT
                    disp.scanline_intensity = node.params.get("scanline_intensity").copied().unwrap_or(0.15);
                    disp.scanline_count = node.params.get("scanline_count").copied().unwrap_or(240.0);
                    disp.curvature = node.params.get("curvature").copied().unwrap_or(0.1);
                    disp.vignette = node.params.get("vignette").copied().unwrap_or(0.3);
                    disp.bloom = node.params.get("bloom").copied().unwrap_or(0.2);
                    disp.phosphor_type = node.params.get("phosphor_type").copied().unwrap_or(0.0);
                }
                updated = true;
            }
        }

        updated
    }
}

/// Extract parameters from a node based on its type
fn extract_node_params(node: &crate::ui::patch::SignalNode) -> (f32, f32, f32, f32) {
    let param0 = node.params.get("frequency")
        .or(node.params.get("gain"))
        .or(node.params.get("levels"))
        .or(node.params.get("threshold"))
        .or(node.params.get("clip_black"))
        .or(node.params.get("mix"))
        .or(node.params.get("amount"))
        .or(node.params.get("size"))
        .copied()
        .unwrap_or(0.0);

    let param1 = node.params.get("phase")
        .or(node.params.get("clip_white"))
        .or(node.params.get("softness"))
        .copied()
        .unwrap_or(0.0);

    let param2 = node.params.get("param2").copied().unwrap_or(0.0);
    let param3 = node.params.get("param3").copied().unwrap_or(0.0);

    (param0, param1, param2, param3)
}

/// Graph compiler - converts SignalPatch to GPU operations
pub struct GraphCompiler {
    /// Maps node ID to output slot
    node_slots: HashMap<u32, u32>,
    /// Maps node ID to operation index (for parameter updates)
    node_to_op: HashMap<u32, usize>,
    /// Next available slot
    next_slot: u32,
    /// Compiled operations
    ops: Vec<GraphOp>,
    /// Frame buffer info
    frame_buffers: Vec<FrameBufferInfo>,
    /// Display outputs
    displays: Vec<DisplayInfo>,
    /// Main output slot (if found)
    main_out_slot: Option<u32>,
    /// Set of frame buffer node IDs (for cycle breaking)
    frame_buffer_nodes: HashSet<u32>,
}

impl GraphCompiler {
    pub fn new() -> Self {
        Self {
            node_slots: HashMap::new(),
            node_to_op: HashMap::new(),
            next_slot: 0,
            ops: Vec::new(),
            frame_buffers: Vec::new(),
            displays: Vec::new(),
            main_out_slot: None,
            frame_buffer_nodes: HashSet::new(),
        }
    }

    /// Compile a SignalPatch into GPU operations
    pub fn compile(patch: &SignalPatch) -> CompiledGraph {
        if patch.nodes.is_empty() {
            return CompiledGraph::default_fallback();
        }

        let mut compiler = Self::new();

        // First pass: identify frame buffer nodes (cycle-breaking points)
        for node in &patch.nodes {
            if node.node_type.is_temporal() {
                compiler.frame_buffer_nodes.insert(node.id);
            }
        }

        // Pre-allocate slots for frame buffer READS (these happen first)
        // This breaks cycles by making FB outputs available before their inputs are processed
        for node in &patch.nodes {
            if node.node_type == NodeType::FrameBuffer {
                let read_slot = compiler.next_slot;
                compiler.next_slot += 1;
                compiler.node_slots.insert(node.id, read_slot);

                // Emit the read operation
                let fb_index = compiler.frame_buffers.len() as u32;
                compiler.ops.push(GraphOp {
                    opcode: OpCode::FrameBufferRead as u32,
                    input_a: fb_index, // Which frame buffer texture to read
                    input_b: 0xFFFFFFFF,
                    output_slot: read_slot,
                    param0: 0.0,
                    param1: 0.0,
                    param2: 0.0,
                    param3: 0.0,
                });

                // Record frame buffer info (will be filled in later with write slot)
                compiler.frame_buffers.push(FrameBufferInfo {
                    node_id: node.id,
                    read_slot,
                    write_slot: 0xFFFFFFFF, // Will be set during write pass
                    buffer_index: fb_index,
                    mix: node.params.get("mix").copied().unwrap_or(0.5),
                    zoom: node.params.get("zoom").copied().unwrap_or(1.0),
                    rotation: node.params.get("rotation").copied().unwrap_or(0.0),
                    hue_shift: node.params.get("hue_shift").copied().unwrap_or(0.0),
                    decay: node.params.get("decay").copied().unwrap_or(0.98),
                    saturation: node.params.get("saturation").copied().unwrap_or(1.0),
                    offset_x: node.params.get("offset_x").copied().unwrap_or(0.0),
                    offset_y: node.params.get("offset_y").copied().unwrap_or(0.0),
                });
            }
        }

        // Build dependency graph and find execution order (topological sort)
        // Frame buffers are excluded since we already emitted their reads
        let sorted_nodes = compiler.topological_sort(patch);

        // Compile each node in execution order
        for node_id in sorted_nodes {
            compiler.compile_node(patch, node_id);
        }

        // Post-pass: emit frame buffer WRITES
        // These must happen after all processing so the FB captures the final signal
        for fb in &mut compiler.frame_buffers {
            // Find what's connected to this frame buffer's input
            let mut input_slot = 0xFFFFFFFF;
            for conn in &patch.connections {
                if conn.to_node == fb.node_id && conn.to_input == 0 {
                    if let Some(&slot) = compiler.node_slots.get(&conn.from_node) {
                        input_slot = slot;
                    }
                }
            }

            if input_slot != 0xFFFFFFFF {
                fb.write_slot = input_slot;
                compiler.ops.push(GraphOp {
                    opcode: OpCode::FrameBufferWrite as u32,
                    input_a: input_slot,
                    input_b: fb.buffer_index,
                    output_slot: 0xFFFFFFFF, // Write doesn't output to a slot
                    param0: fb.mix,
                    param1: fb.zoom,
                    param2: fb.rotation,
                    param3: fb.decay,
                });
            }
        }

        // Build final compiled graph
        let mut ops = vec![GraphOp::default(); MAX_OPS];
        for (i, op) in compiler.ops.iter().enumerate() {
            if i < MAX_OPS {
                ops[i] = *op;
            }
        }

        CompiledGraph {
            header: CompiledGraphHeader {
                op_count: compiler.ops.len().min(MAX_OPS) as u32,
                frame_buffer_count: compiler.frame_buffers.len() as u32,
                display_count: compiler.displays.len() as u32,
                main_out_slot: compiler.main_out_slot.unwrap_or(
                    // If no MainOut, use first display's input slot
                    compiler.displays.first().map(|d| d.input_slot).unwrap_or(0xFFFFFFFF)
                ),
            },
            ops,
            frame_buffers: compiler.frame_buffers,
            displays: compiler.displays,
            node_to_op: compiler.node_to_op,
        }
    }

    /// Topological sort using Kahn's algorithm
    /// Treats frame buffer nodes as having no incoming edges (since we pre-emit their reads)
    fn topological_sort(&self, patch: &SignalPatch) -> Vec<u32> {
        // Build adjacency list and in-degree count
        let mut in_degree: HashMap<u32, usize> = HashMap::new();
        let mut adjacency: HashMap<u32, Vec<u32>> = HashMap::new();

        // Initialize all nodes (except frame buffers which are pre-emitted)
        for node in &patch.nodes {
            if !self.frame_buffer_nodes.contains(&node.id) {
                in_degree.insert(node.id, 0);
                adjacency.insert(node.id, Vec::new());
            }
        }

        // Count edges (ignoring edges TO frame buffers, since those are handled separately)
        for conn in &patch.connections {
            // Skip if destination is a frame buffer
            if self.frame_buffer_nodes.contains(&conn.to_node) {
                continue;
            }

            // If source is a frame buffer, it's already emitted so no dependency
            if self.frame_buffer_nodes.contains(&conn.from_node) {
                continue;
            }

            if let Some(deg) = in_degree.get_mut(&conn.to_node) {
                *deg += 1;
            }
            if let Some(adj) = adjacency.get_mut(&conn.from_node) {
                adj.push(conn.to_node);
            }
        }

        // Add edges from frame buffers (they have 0 in-degree since already emitted)
        for conn in &patch.connections {
            if self.frame_buffer_nodes.contains(&conn.from_node) {
                // Frame buffer output -> some node
                if !self.frame_buffer_nodes.contains(&conn.to_node) {
                    // No need to add to in_degree since FB is already processed
                }
            }
        }

        // Start with nodes that have no inputs (sources + nodes fed only by frame buffers)
        let mut queue: Vec<u32> = in_degree
            .iter()
            .filter(|(_, deg)| **deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut sorted = Vec::new();

        while let Some(node_id) = queue.pop() {
            sorted.push(node_id);

            if let Some(neighbors) = adjacency.get(&node_id) {
                for &neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(&neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push(neighbor);
                        }
                    }
                }
            }
        }

        sorted
    }

    /// Compile a single node
    fn compile_node(&mut self, patch: &SignalPatch, node_id: u32) {
        let node = match patch.get_node(node_id) {
            Some(n) => n,
            None => return,
        };

        // Frame buffers are handled separately (pre-emitted)
        if node.node_type.is_temporal() {
            return;
        }

        // Get input slots from connections
        let inputs = self.get_input_slots(patch, node_id);
        let input_a = inputs.get(0).copied().unwrap_or(0xFFFFFFFF);
        let input_b = inputs.get(1).copied().unwrap_or(0xFFFFFFFF);

        // Allocate output slot (displays don't output to slots used by other nodes)
        let output_slot = self.next_slot;
        self.next_slot += 1;
        self.node_slots.insert(node_id, output_slot);

        // Get parameters from node
        let (param0, param1, param2, param3) = extract_node_params(node);

        // Create operation
        let op = GraphOp {
            opcode: OpCode::from_node_type(node.node_type) as u32,
            input_a,
            input_b,
            output_slot,
            param0,
            param1,
            param2,
            param3,
        };

        // Record node_id → op_index mapping for parameter updates
        let op_index = self.ops.len();
        self.node_to_op.insert(node_id, op_index);

        self.ops.push(op);

        // Track output nodes
        match node.node_type {
            NodeType::MainOut => {
                self.main_out_slot = Some(output_slot);
            }
            NodeType::Monitor | NodeType::CrtDisplay => {
                let display_type = if node.node_type == NodeType::CrtDisplay { 1 } else { 0 };
                let display_index = self.displays.len() as u32;

                self.displays.push(DisplayInfo {
                    node_id,
                    display_type,
                    input_slot: input_a,
                    display_index,
                    scanline_intensity: node.params.get("scanline_intensity").copied().unwrap_or(0.15),
                    scanline_count: node.params.get("scanline_count").copied().unwrap_or(240.0),
                    curvature: node.params.get("curvature").copied().unwrap_or(0.1),
                    vignette: node.params.get("vignette").copied().unwrap_or(0.3),
                    bloom: node.params.get("bloom").copied().unwrap_or(0.2),
                    phosphor_type: node.params.get("phosphor_type").copied().unwrap_or(0.0),
                    brightness: node.params.get("brightness").copied().unwrap_or(1.0),
                    contrast: node.params.get("contrast").copied().unwrap_or(1.0),
                });

                // First display becomes main output if no MainOut
                if self.main_out_slot.is_none() {
                    self.main_out_slot = Some(input_a);
                }
            }
            _ => {}
        }
    }

    /// Get input slot indices for a node
    fn get_input_slots(&self, patch: &SignalPatch, node_id: u32) -> Vec<u32> {
        let mut inputs: Vec<(usize, u32)> = Vec::new();

        for conn in &patch.connections {
            if conn.to_node == node_id {
                if let Some(&slot) = self.node_slots.get(&conn.from_node) {
                    inputs.push((conn.to_input, slot));
                }
            }
        }

        // Sort by input index
        inputs.sort_by_key(|(idx, _)| *idx);

        inputs.into_iter().map(|(_, slot)| slot).collect()
    }
}

impl Default for GraphCompiler {
    fn default() -> Self {
        Self::new()
    }
}
