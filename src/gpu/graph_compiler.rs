//! Graph Compiler
//!
//! Compiles the SignalPatch node graph into GPU-executable operations.
//! The shader interprets these operations at runtime to evaluate the graph.

use crate::ui::patch::{SignalPatch, NodeType, Connection};
use std::collections::HashMap;

/// Maximum number of operations the GPU can execute per frame (reduced for performance)
pub const MAX_OPS: usize = 16;

/// Maximum number of signal slots for intermediate values (reduced for performance)
pub const MAX_SLOTS: usize = 16;

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
    FeedbackIn = 11,

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

    // Output markers
    MainOut = 50,
    FeedbackOut = 51,

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
            NodeType::FeedbackIn => OpCode::FeedbackIn,
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
            NodeType::MainOut => OpCode::MainOut,
            NodeType::FeedbackOut => OpCode::FeedbackOut,
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

/// Compiled graph header
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CompiledGraphHeader {
    /// Number of operations to execute
    pub op_count: u32,
    /// Slot index containing final output (MainOut)
    pub main_out_slot: u32,
    /// Slot index for feedback output (or 0xFFFFFFFF if none)
    pub feedback_out_slot: u32,
    /// Padding for alignment
    pub _pad: u32,
}

impl Default for CompiledGraphHeader {
    fn default() -> Self {
        Self {
            op_count: 0,
            main_out_slot: 0xFFFFFFFF,
            feedback_out_slot: 0xFFFFFFFF,
            _pad: 0,
        }
    }
}

/// Compiled graph ready for GPU upload
pub struct CompiledGraph {
    pub header: CompiledGraphHeader,
    pub ops: Vec<GraphOp>,
}

impl CompiledGraph {
    /// Create an empty graph (renders black)
    pub fn empty() -> Self {
        Self {
            header: CompiledGraphHeader::default(),
            ops: vec![GraphOp::default(); MAX_OPS],
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
                main_out_slot: 2,
                feedback_out_slot: 0xFFFFFFFF,
                _pad: 0,
            },
            ops,
        }
    }
}

/// Graph compiler - converts SignalPatch to GPU operations
pub struct GraphCompiler {
    /// Maps node ID to output slot
    node_slots: HashMap<u32, u32>,
    /// Next available slot
    next_slot: u32,
    /// Compiled operations
    ops: Vec<GraphOp>,
    /// Main output slot (if found)
    main_out_slot: Option<u32>,
    /// Feedback output slot (if found)
    feedback_out_slot: Option<u32>,
}

impl GraphCompiler {
    pub fn new() -> Self {
        Self {
            node_slots: HashMap::new(),
            next_slot: 0,
            ops: Vec::new(),
            main_out_slot: None,
            feedback_out_slot: None,
        }
    }

    /// Compile a SignalPatch into GPU operations
    pub fn compile(patch: &SignalPatch) -> CompiledGraph {
        if patch.nodes.is_empty() {
            return CompiledGraph::default_fallback();
        }

        let mut compiler = Self::new();

        // Build dependency graph and find execution order (topological sort)
        let sorted_nodes = compiler.topological_sort(patch);

        // Compile each node in execution order
        for node_id in sorted_nodes {
            compiler.compile_node(patch, node_id);
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
                main_out_slot: compiler.main_out_slot.unwrap_or(0xFFFFFFFF),
                feedback_out_slot: compiler.feedback_out_slot.unwrap_or(0xFFFFFFFF),
                _pad: 0,
            },
            ops,
        }
    }

    /// Topological sort using Kahn's algorithm
    fn topological_sort(&self, patch: &SignalPatch) -> Vec<u32> {
        // Build adjacency list and in-degree count
        let mut in_degree: HashMap<u32, usize> = HashMap::new();
        let mut adjacency: HashMap<u32, Vec<u32>> = HashMap::new();

        // Initialize all nodes
        for node in &patch.nodes {
            in_degree.insert(node.id, 0);
            adjacency.insert(node.id, Vec::new());
        }

        // Count edges
        for conn in &patch.connections {
            if let Some(deg) = in_degree.get_mut(&conn.to_node) {
                *deg += 1;
            }
            if let Some(adj) = adjacency.get_mut(&conn.from_node) {
                adj.push(conn.to_node);
            }
        }

        // Start with nodes that have no inputs
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

        // Get input slots from connections
        let inputs = self.get_input_slots(patch, node_id);
        let input_a = inputs.get(0).copied().unwrap_or(0xFFFFFFFF);
        let input_b = inputs.get(1).copied().unwrap_or(0xFFFFFFFF);

        // Allocate output slot
        let output_slot = self.next_slot;
        self.next_slot += 1;
        self.node_slots.insert(node_id, output_slot);

        // Get parameters from node
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

        self.ops.push(op);

        // Track output nodes
        match node.node_type {
            NodeType::MainOut => {
                self.main_out_slot = Some(output_slot);
            }
            NodeType::FeedbackOut => {
                self.feedback_out_slot = Some(output_slot);
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
