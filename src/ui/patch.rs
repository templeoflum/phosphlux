//! Signal Patching System
//!
//! Modular signal routing inspired by analog synthesizer patch bays.
//! Allows connecting signal sources to processors to outputs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of signal nodes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    // Sources (outputs only)
    RampH,
    RampV,
    RampRadial,
    OscillatorH,
    OscillatorV,
    NoiseWhite,
    NoisePink,
    NoiseBrown,
    ShapeCircle,
    ShapeRect,
    ShapeDiamond,

    // Temporal nodes (feedback from routing topology)
    FrameBuffer,    // Stores current input, outputs previous frame (enables feedback loops)
    DelayLine,      // Multi-frame delay buffer

    // Processors (inputs and outputs)
    Folder,
    Quantizer,
    SoftClip,
    Solarize,
    LumaKey,
    ChromaKey,
    Mixer,
    Multiplier,
    Inverter,
    HueShift,

    // Colorizers (input: luma, output: RGB)
    ColorSpectrum,
    ColorThreshold,
    ColorGradient,

    // Display nodes (render targets with their own textures)
    Monitor,        // Clean preview display
    CrtDisplay,     // Full CRT emulation with phosphor, scanlines, geometry

    // Legacy destinations (for backwards compatibility during transition)
    MainOut,
}

impl NodeType {
    pub fn category(&self) -> &'static str {
        match self {
            NodeType::RampH | NodeType::RampV | NodeType::RampRadial |
            NodeType::OscillatorH | NodeType::OscillatorV |
            NodeType::NoiseWhite | NodeType::NoisePink | NodeType::NoiseBrown |
            NodeType::ShapeCircle | NodeType::ShapeRect | NodeType::ShapeDiamond => "Sources",

            NodeType::FrameBuffer | NodeType::DelayLine => "Temporal",

            NodeType::Folder | NodeType::Quantizer | NodeType::SoftClip |
            NodeType::Solarize | NodeType::Inverter => "Waveshapers",

            NodeType::LumaKey | NodeType::ChromaKey | NodeType::Mixer |
            NodeType::Multiplier => "Mixing",

            NodeType::HueShift => "Color",

            NodeType::ColorSpectrum | NodeType::ColorThreshold |
            NodeType::ColorGradient => "Colorizers",

            NodeType::Monitor | NodeType::CrtDisplay => "Displays",

            NodeType::MainOut => "Outputs",
        }
    }

    pub fn has_inputs(&self) -> bool {
        !matches!(self,
            NodeType::RampH | NodeType::RampV | NodeType::RampRadial |
            NodeType::OscillatorH | NodeType::OscillatorV |
            NodeType::NoiseWhite | NodeType::NoisePink | NodeType::NoiseBrown |
            NodeType::ShapeCircle | NodeType::ShapeRect | NodeType::ShapeDiamond
        )
    }

    pub fn has_outputs(&self) -> bool {
        // Display nodes have no outputs (they ARE the output)
        // MainOut is legacy and also has no outputs
        !matches!(self,
            NodeType::Monitor | NodeType::CrtDisplay | NodeType::MainOut
        )
    }

    /// Returns true if this node is a display/render target
    #[allow(dead_code)]
    pub fn is_display(&self) -> bool {
        matches!(self, NodeType::Monitor | NodeType::CrtDisplay)
    }

    /// Returns true if this node introduces a frame delay (for cycle breaking)
    pub fn is_temporal(&self) -> bool {
        matches!(self, NodeType::FrameBuffer | NodeType::DelayLine)
    }

    pub fn input_count(&self) -> usize {
        match self {
            // Sources have no inputs
            NodeType::RampH | NodeType::RampV | NodeType::RampRadial |
            NodeType::OscillatorH | NodeType::OscillatorV |
            NodeType::NoiseWhite | NodeType::NoisePink | NodeType::NoiseBrown |
            NodeType::ShapeCircle | NodeType::ShapeRect | NodeType::ShapeDiamond => 0,

            // Single input processors
            NodeType::Folder | NodeType::Quantizer | NodeType::SoftClip |
            NodeType::Solarize | NodeType::Inverter | NodeType::HueShift |
            NodeType::ColorSpectrum | NodeType::ColorThreshold | NodeType::ColorGradient => 1,

            // Temporal nodes (single input, single output)
            NodeType::FrameBuffer | NodeType::DelayLine => 1,

            // Two-input processors
            NodeType::LumaKey | NodeType::Mixer | NodeType::Multiplier => 2,

            // Three-input (A, B, key)
            NodeType::ChromaKey => 3,

            // Display nodes (single input - the video signal to display)
            NodeType::Monitor | NodeType::CrtDisplay => 1,

            // Legacy output
            NodeType::MainOut => 1,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            NodeType::RampH => "H Ramp",
            NodeType::RampV => "V Ramp",
            NodeType::RampRadial => "Radial",
            NodeType::OscillatorH => "H Osc",
            NodeType::OscillatorV => "V Osc",
            NodeType::NoiseWhite => "White Noise",
            NodeType::NoisePink => "Pink Noise",
            NodeType::NoiseBrown => "Brown Noise",
            NodeType::ShapeCircle => "Circle",
            NodeType::ShapeRect => "Rectangle",
            NodeType::ShapeDiamond => "Diamond",
            NodeType::FrameBuffer => "Frame Buffer",
            NodeType::DelayLine => "Delay Line",
            NodeType::Folder => "Folder",
            NodeType::Quantizer => "Quantizer",
            NodeType::SoftClip => "Soft Clip",
            NodeType::Solarize => "Solarize",
            NodeType::LumaKey => "Luma Key",
            NodeType::ChromaKey => "Chroma Key",
            NodeType::Mixer => "Mixer",
            NodeType::Multiplier => "Multiply",
            NodeType::Inverter => "Invert",
            NodeType::HueShift => "Hue Shift",
            NodeType::ColorSpectrum => "Spectrum",
            NodeType::ColorThreshold => "Threshold",
            NodeType::ColorGradient => "Gradient",
            NodeType::Monitor => "Monitor",
            NodeType::CrtDisplay => "CRT Display",
            NodeType::MainOut => "Main Out",
        }
    }

    /// Get input port names for this node type
    #[allow(dead_code)]
    pub fn input_names(&self) -> &'static [&'static str] {
        match self {
            // No inputs
            NodeType::RampH | NodeType::RampV | NodeType::RampRadial |
            NodeType::OscillatorH | NodeType::OscillatorV |
            NodeType::NoiseWhite | NodeType::NoisePink | NodeType::NoiseBrown |
            NodeType::ShapeCircle | NodeType::ShapeRect | NodeType::ShapeDiamond => &[],

            // Single "In" input
            NodeType::Folder | NodeType::Quantizer | NodeType::SoftClip |
            NodeType::Solarize | NodeType::Inverter | NodeType::HueShift |
            NodeType::ColorSpectrum | NodeType::ColorThreshold | NodeType::ColorGradient |
            NodeType::FrameBuffer | NodeType::DelayLine |
            NodeType::Monitor | NodeType::CrtDisplay | NodeType::MainOut => &["In"],

            // Two inputs: A and B
            NodeType::Mixer | NodeType::Multiplier => &["A", "B"],

            // Luma key: Video and Key
            NodeType::LumaKey => &["Video", "Key"],

            // Chroma key: A, B, and Hue reference
            NodeType::ChromaKey => &["A", "B", "Hue"],
        }
    }
}

/// A node in the signal graph
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignalNode {
    pub id: u32,
    pub node_type: NodeType,
    pub position: (f32, f32),  // Position in the patch editor
    pub params: HashMap<String, f32>,  // Node-specific parameters
}

impl SignalNode {
    pub fn new(id: u32, node_type: NodeType, x: f32, y: f32) -> Self {
        let mut params = HashMap::new();

        // Set default parameters based on node type
        match node_type {
            NodeType::OscillatorH | NodeType::OscillatorV => {
                params.insert("frequency".to_string(), 4.0);
                params.insert("phase".to_string(), 0.0);
            }
            NodeType::Folder => {
                params.insert("gain".to_string(), 2.0);
            }
            NodeType::Quantizer => {
                params.insert("levels".to_string(), 8.0);
            }
            NodeType::Solarize => {
                params.insert("threshold".to_string(), 0.5);
            }
            NodeType::LumaKey => {
                params.insert("clip_black".to_string(), 0.2);
                params.insert("clip_white".to_string(), 0.8);
            }
            NodeType::Mixer => {
                // Fader: 0.0 = full A, 1.0 = full B
                params.insert("fader".to_string(), 0.5);
                // Blend mode: 0=Mix, 1=Add, 2=Multiply, 3=Screen, 4=Overlay,
                //             5=Difference, 6=LumaKeyA, 7=LumaKeyB
                params.insert("blend_mode".to_string(), 0.0);
                // Luma key threshold (for LumaKey modes)
                params.insert("key_threshold".to_string(), 0.5);
                // Luma key softness/edge (0=hard, 1=soft)
                params.insert("key_softness".to_string(), 0.1);
                // Key invert (0=normal, 1=inverted)
                params.insert("key_invert".to_string(), 0.0);
            }
            NodeType::HueShift => {
                params.insert("amount".to_string(), 0.0);
            }
            NodeType::ColorThreshold => {
                params.insert("levels".to_string(), 8.0);
            }
            NodeType::ShapeCircle | NodeType::ShapeRect | NodeType::ShapeDiamond => {
                params.insert("size".to_string(), 0.3);
                params.insert("softness".to_string(), 0.05);
            }
            // Frame Buffer - feedback parameters that were previously global
            NodeType::FrameBuffer => {
                params.insert("mix".to_string(), 0.5);        // How much of input vs previous frame
                params.insert("zoom".to_string(), 1.0);       // Zoom per frame
                params.insert("rotation".to_string(), 0.0);   // Rotation per frame (radians)
                params.insert("hue_shift".to_string(), 0.0);  // Hue shift per frame
                params.insert("decay".to_string(), 0.98);     // Brightness decay
                params.insert("saturation".to_string(), 1.0); // Saturation multiplier
                params.insert("offset_x".to_string(), 0.0);   // X translation
                params.insert("offset_y".to_string(), 0.0);   // Y translation
            }
            // Delay Line
            NodeType::DelayLine => {
                params.insert("frames".to_string(), 4.0);     // Number of frames to delay
            }
            // CRT Display - all the CRT parameters
            NodeType::CrtDisplay => {
                params.insert("scanline_intensity".to_string(), 0.15);
                params.insert("scanline_count".to_string(), 240.0);
                params.insert("curvature".to_string(), 0.1);
                params.insert("vignette".to_string(), 0.3);
                params.insert("bloom".to_string(), 0.2);
                params.insert("phosphor_type".to_string(), 0.0); // 0=RGB, 1=Green, 2=Amber
                params.insert("convergence_x".to_string(), 0.0);
                params.insert("convergence_y".to_string(), 0.0);
                params.insert("brightness".to_string(), 1.0);
                params.insert("contrast".to_string(), 1.0);
            }
            // Monitor - clean display, minimal params
            NodeType::Monitor => {
                params.insert("brightness".to_string(), 1.0);
                params.insert("contrast".to_string(), 1.0);
            }
            _ => {}
        }

        Self {
            id,
            node_type,
            position: (x, y),
            params,
        }
    }
}

/// A connection between two nodes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Connection {
    pub from_node: u32,
    pub from_output: usize,
    pub to_node: u32,
    pub to_input: usize,
}

/// The complete signal patch
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SignalPatch {
    pub nodes: Vec<SignalNode>,
    pub connections: Vec<Connection>,
    next_id: u32,
}

impl SignalPatch {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a default patch with basic signal flow and feedback from routing
    pub fn default_patch() -> Self {
        let mut patch = Self::new();

        // Sources
        let radial = patch.add_node(NodeType::RampRadial, 50.0, 100.0);
        let osc_h = patch.add_node(NodeType::OscillatorH, 50.0, 200.0);
        let osc_v = patch.add_node(NodeType::OscillatorV, 50.0, 300.0);

        // Mix sources together
        let mixer1 = patch.add_node(NodeType::Mixer, 200.0, 150.0);
        let mixer2 = patch.add_node(NodeType::Mixer, 350.0, 200.0);

        // Process
        let folder = patch.add_node(NodeType::Folder, 500.0, 200.0);
        let colorizer = patch.add_node(NodeType::ColorSpectrum, 650.0, 200.0);

        // Feedback loop - Frame Buffer provides previous frame output
        let frame_buffer = patch.add_node(NodeType::FrameBuffer, 650.0, 400.0);
        let mixer3 = patch.add_node(NodeType::Mixer, 800.0, 250.0);

        // Display output - CRT with full emulation
        let crt = patch.add_node(NodeType::CrtDisplay, 950.0, 250.0);

        // Connect the signal chain
        patch.connect(radial, 0, mixer1, 0);
        patch.connect(osc_h, 0, mixer1, 1);
        patch.connect(mixer1, 0, mixer2, 0);
        patch.connect(osc_v, 0, mixer2, 1);
        patch.connect(mixer2, 0, folder, 0);
        patch.connect(folder, 0, colorizer, 0);

        // Mix colorized signal with feedback
        patch.connect(colorizer, 0, mixer3, 0);
        patch.connect(frame_buffer, 0, mixer3, 1);  // Frame buffer output -> mixer B

        // Route to display and back to frame buffer (creates the feedback loop!)
        patch.connect(mixer3, 0, crt, 0);
        patch.connect(mixer3, 0, frame_buffer, 0);  // Also feed into frame buffer for next frame

        patch
    }

    pub fn add_node(&mut self, node_type: NodeType, x: f32, y: f32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.push(SignalNode::new(id, node_type, x, y));
        id
    }

    pub fn remove_node(&mut self, id: u32) {
        self.nodes.retain(|n| n.id != id);
        self.connections.retain(|c| c.from_node != id && c.to_node != id);
    }

    pub fn connect(&mut self, from: u32, from_output: usize, to: u32, to_input: usize) {
        // Remove any existing connection to this input
        self.connections.retain(|c| !(c.to_node == to && c.to_input == to_input));

        self.connections.push(Connection {
            from_node: from,
            from_output,
            to_node: to,
            to_input,
        });
    }

    pub fn disconnect(&mut self, to: u32, to_input: usize) {
        self.connections.retain(|c| !(c.to_node == to && c.to_input == to_input));
    }

    pub fn get_node(&self, id: u32) -> Option<&SignalNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn get_node_mut(&mut self, id: u32) -> Option<&mut SignalNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    /// Serialize to JSON for saving
    #[allow(dead_code)]
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON for loading
    #[allow(dead_code)]
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
