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
    FeedbackIn,

    // Channel Inputs (read from specific channels)
    ChannelKeyIn,   // Read from Key channel
    ChannelAux1In,  // Read from Aux1 channel
    ChannelAux2In,  // Read from Aux2 channel

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

    // Destinations (inputs only)
    MainOut,
    FeedbackOut,

    // Channel Outputs (write to specific channels)
    ChannelKeyOut,  // Write to Key channel
    ChannelAux1Out, // Write to Aux1 channel
    ChannelAux2Out, // Write to Aux2 channel
}

impl NodeType {
    pub fn category(&self) -> &'static str {
        match self {
            NodeType::RampH | NodeType::RampV | NodeType::RampRadial |
            NodeType::OscillatorH | NodeType::OscillatorV |
            NodeType::NoiseWhite | NodeType::NoisePink | NodeType::NoiseBrown |
            NodeType::ShapeCircle | NodeType::ShapeRect | NodeType::ShapeDiamond |
            NodeType::FeedbackIn => "Sources",

            NodeType::ChannelKeyIn | NodeType::ChannelAux1In | NodeType::ChannelAux2In => "Channels",

            NodeType::Folder | NodeType::Quantizer | NodeType::SoftClip |
            NodeType::Solarize | NodeType::Inverter => "Waveshapers",

            NodeType::LumaKey | NodeType::ChromaKey | NodeType::Mixer |
            NodeType::Multiplier => "Mixing",

            NodeType::HueShift => "Color",

            NodeType::ColorSpectrum | NodeType::ColorThreshold |
            NodeType::ColorGradient => "Colorizers",

            NodeType::MainOut | NodeType::FeedbackOut |
            NodeType::ChannelKeyOut | NodeType::ChannelAux1Out | NodeType::ChannelAux2Out => "Outputs",
        }
    }

    pub fn has_inputs(&self) -> bool {
        !matches!(self,
            NodeType::RampH | NodeType::RampV | NodeType::RampRadial |
            NodeType::OscillatorH | NodeType::OscillatorV |
            NodeType::NoiseWhite | NodeType::NoisePink | NodeType::NoiseBrown |
            NodeType::ShapeCircle | NodeType::ShapeRect | NodeType::ShapeDiamond |
            NodeType::FeedbackIn |
            NodeType::ChannelKeyIn | NodeType::ChannelAux1In | NodeType::ChannelAux2In
        )
    }

    pub fn has_outputs(&self) -> bool {
        !matches!(self,
            NodeType::MainOut | NodeType::FeedbackOut |
            NodeType::ChannelKeyOut | NodeType::ChannelAux1Out | NodeType::ChannelAux2Out
        )
    }

    pub fn input_count(&self) -> usize {
        match self {
            // Sources have no inputs (including channel inputs)
            NodeType::RampH | NodeType::RampV | NodeType::RampRadial |
            NodeType::OscillatorH | NodeType::OscillatorV |
            NodeType::NoiseWhite | NodeType::NoisePink | NodeType::NoiseBrown |
            NodeType::ShapeCircle | NodeType::ShapeRect | NodeType::ShapeDiamond |
            NodeType::FeedbackIn |
            NodeType::ChannelKeyIn | NodeType::ChannelAux1In | NodeType::ChannelAux2In => 0,

            // Single input processors
            NodeType::Folder | NodeType::Quantizer | NodeType::SoftClip |
            NodeType::Solarize | NodeType::Inverter | NodeType::HueShift |
            NodeType::ColorSpectrum | NodeType::ColorThreshold | NodeType::ColorGradient => 1,

            // Two-input processors
            NodeType::LumaKey | NodeType::Mixer | NodeType::Multiplier => 2,

            // Three-input (A, B, key)
            NodeType::ChromaKey => 3,

            // Outputs (including channel outputs)
            NodeType::MainOut | NodeType::FeedbackOut |
            NodeType::ChannelKeyOut | NodeType::ChannelAux1Out | NodeType::ChannelAux2Out => 1,
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
            NodeType::FeedbackIn => "Feedback In",
            NodeType::ChannelKeyIn => "Key In",
            NodeType::ChannelAux1In => "Aux1 In",
            NodeType::ChannelAux2In => "Aux2 In",
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
            NodeType::MainOut => "Main Out",
            NodeType::FeedbackOut => "Feedback Out",
            NodeType::ChannelKeyOut => "Key Out",
            NodeType::ChannelAux1Out => "Aux1 Out",
            NodeType::ChannelAux2Out => "Aux2 Out",
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
                params.insert("mix".to_string(), 0.5);
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

    /// Create a default patch with basic signal flow
    pub fn default_patch() -> Self {
        let mut patch = Self::new();

        // Add default nodes
        let radial = patch.add_node(NodeType::RampRadial, 50.0, 100.0);
        let osc_h = patch.add_node(NodeType::OscillatorH, 50.0, 200.0);
        let osc_v = patch.add_node(NodeType::OscillatorV, 50.0, 300.0);
        let mixer1 = patch.add_node(NodeType::Mixer, 200.0, 150.0);
        let mixer2 = patch.add_node(NodeType::Mixer, 350.0, 200.0);
        let folder = patch.add_node(NodeType::Folder, 500.0, 200.0);
        let colorizer = patch.add_node(NodeType::ColorSpectrum, 650.0, 200.0);
        let feedback = patch.add_node(NodeType::FeedbackIn, 650.0, 350.0);
        let mixer3 = patch.add_node(NodeType::Mixer, 800.0, 250.0);
        let main_out = patch.add_node(NodeType::MainOut, 950.0, 250.0);

        // Connect nodes
        patch.connect(radial, 0, mixer1, 0);
        patch.connect(osc_h, 0, mixer1, 1);
        patch.connect(mixer1, 0, mixer2, 0);
        patch.connect(osc_v, 0, mixer2, 1);
        patch.connect(mixer2, 0, folder, 0);
        patch.connect(folder, 0, colorizer, 0);
        patch.connect(colorizer, 0, mixer3, 0);
        patch.connect(feedback, 0, mixer3, 1);
        patch.connect(mixer3, 0, main_out, 0);

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
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON for loading
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
