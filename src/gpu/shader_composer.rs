//! Shader Composer
//!
//! Composes device shaders at compile time instead of runtime dispatch.
//! This is the core architectural change from v1 → v2.
//!
//! v1 approach: Single monolithic shader with if-else dispatch
//!   - Every pixel evaluates every condition
//!   - GPU divergence kills performance
//!   - Adding nodes bloats the shader
//!
//! v2 approach: Compose specialized shader per patch
//!   - Only include code for devices in use
//!   - No runtime branching
//!   - Each device can have optimized code

use std::collections::HashMap;

/// Common shader code included in all composed shaders
pub const COMMON_WGSL: &str = include_str!("../shaders/common.wgsl");

/// Component library code
pub const COMPONENTS_WGSL: &str = include_str!("../shaders/components.wgsl");

/// A device that can be composed into a shader
pub trait ShaderDevice {
    /// Unique identifier for this device type
    fn device_id(&self) -> &'static str;

    /// Human-readable name
    fn device_name(&self) -> &str;

    /// WGSL code for this device's processing
    /// Should define a function: fn device_{id}_process(...) -> vec4<f32>
    fn shader_code(&self) -> &str;

    /// Parameters this device exposes
    fn parameters(&self) -> Vec<DeviceParameter>;

    /// Input ports
    fn inputs(&self) -> Vec<Port>;

    /// Output ports
    fn outputs(&self) -> Vec<Port>;

    /// Does this device need a feedback texture?
    fn needs_feedback(&self) -> bool {
        false
    }

    /// Does this device need persistent state between frames?
    fn needs_state(&self) -> bool {
        false
    }

    /// WGSL struct definition for state (if needs_state)
    fn state_struct(&self) -> Option<&str> {
        None
    }
}

/// A device parameter
#[derive(Clone, Debug)]
pub struct DeviceParameter {
    /// Parameter name (used in shader uniform)
    pub name: String,
    /// Display name for UI
    pub display_name: String,
    /// Minimum value
    pub min: f32,
    /// Maximum value
    pub max: f32,
    /// Default value
    pub default: f32,
    /// Unit (e.g., "Hz", "V", "%")
    pub unit: String,
}

impl DeviceParameter {
    pub fn new(name: &str, display: &str, min: f32, max: f32, default: f32) -> Self {
        Self {
            name: name.to_string(),
            display_name: display.to_string(),
            min,
            max,
            default,
            unit: String::new(),
        }
    }

    pub fn with_unit(mut self, unit: &str) -> Self {
        self.unit = unit.to_string();
        self
    }
}

/// A device port (input or output)
#[derive(Clone, Debug)]
pub struct Port {
    /// Port name
    pub name: String,
    /// Signal type
    pub signal_type: SignalType,
}

/// Type of signal a port carries
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SignalType {
    /// RGB video signal
    Rgb,
    /// Single-channel luma/CV
    Mono,
    /// Control voltage (typically for modulation)
    Cv,
}

/// A connection between device ports
#[derive(Clone, Debug)]
pub struct Connection {
    /// Source device index
    pub from_device: usize,
    /// Source port name
    pub from_port: String,
    /// Destination device index
    pub to_device: usize,
    /// Destination port name
    pub to_port: String,
}

/// A device instance in a patch
pub struct DeviceInstance {
    /// The device type
    pub device: Box<dyn ShaderDevice>,
    /// Instance ID (unique within patch)
    pub instance_id: u32,
    /// Parameter values (overrides defaults)
    pub params: HashMap<String, f32>,
    /// Position in UI (for editor)
    pub position: (f32, f32),
}

/// A complete device patch
pub struct DevicePatch {
    /// All device instances
    pub devices: Vec<DeviceInstance>,
    /// Connections between devices
    pub connections: Vec<Connection>,
    /// Output device index
    pub output_device: usize,
}

/// Composed shader ready for GPU
pub struct ComposedShader {
    /// Complete WGSL source
    pub wgsl_source: String,
    /// Parameter uniform layout
    pub param_layout: ParamLayout,
    /// Texture bindings needed
    pub texture_bindings: Vec<TextureBinding>,
    /// State buffer bindings needed
    pub state_bindings: Vec<StateBinding>,
}

/// Layout of parameter uniforms
#[derive(Default)]
pub struct ParamLayout {
    /// Total size in bytes
    pub size: usize,
    /// Offsets for each device's parameters
    pub device_offsets: HashMap<u32, usize>,
    /// Individual parameter info
    pub params: Vec<ParamInfo>,
}

#[derive(Clone)]
pub struct ParamInfo {
    pub name: String,
    pub device_id: u32,
    pub offset: usize,
}

/// Texture binding requirement
pub struct TextureBinding {
    pub device_id: u32,
    pub binding_type: TextureBindingType,
}

#[derive(Clone, Copy)]
pub enum TextureBindingType {
    Feedback,
    Input,
}

/// State buffer binding
pub struct StateBinding {
    pub device_id: u32,
    pub struct_name: String,
    pub size: usize,
}

/// Composes shaders from device patches
pub struct ShaderComposer {
    /// Cached compiled shaders (by patch hash)
    cache: HashMap<u64, ComposedShader>,
}

impl ShaderComposer {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Compose a shader for a device patch
    pub fn compose(&mut self, patch: &DevicePatch) -> Result<&ComposedShader, ComposeError> {
        // For now, always recompile (caching can be added later)
        let shader = self.compose_internal(patch)?;
        let hash = self.patch_hash(patch);
        self.cache.insert(hash, shader);
        Ok(self.cache.get(&hash).unwrap())
    }

    fn compose_internal(&self, patch: &DevicePatch) -> Result<ComposedShader, ComposeError> {
        let mut wgsl = String::new();

        // 1. Add common code
        wgsl.push_str("// === COMMON ===\n");
        wgsl.push_str(COMMON_WGSL);
        wgsl.push_str("\n\n");

        // 2. Add component library
        wgsl.push_str("// === COMPONENTS ===\n");
        wgsl.push_str(COMPONENTS_WGSL);
        wgsl.push_str("\n\n");

        // 3. Build parameter uniform struct
        let (param_struct, param_layout) = self.build_param_struct(patch);
        wgsl.push_str("// === PARAMETERS ===\n");
        wgsl.push_str(&param_struct);
        wgsl.push_str("\n\n");

        // 4. Add texture bindings
        let (texture_bindings_wgsl, texture_bindings) = self.build_texture_bindings(patch);
        wgsl.push_str("// === TEXTURES ===\n");
        wgsl.push_str(&texture_bindings_wgsl);
        wgsl.push_str("\n\n");

        // 5. Add state structs if needed
        let (state_wgsl, state_bindings) = self.build_state_bindings(patch);
        if !state_wgsl.is_empty() {
            wgsl.push_str("// === STATE ===\n");
            wgsl.push_str(&state_wgsl);
            wgsl.push_str("\n\n");
        }

        // 6. Add device shader code
        wgsl.push_str("// === DEVICES ===\n");
        for device_inst in &patch.devices {
            wgsl.push_str(&format!(
                "// --- {} (instance {}) ---\n",
                device_inst.device.device_name(),
                device_inst.instance_id
            ));
            wgsl.push_str(device_inst.device.shader_code());
            wgsl.push_str("\n\n");
        }

        // 7. Generate main function
        let main_fn = self.generate_main(patch)?;
        wgsl.push_str("// === MAIN ===\n");
        wgsl.push_str(&main_fn);

        Ok(ComposedShader {
            wgsl_source: wgsl,
            param_layout,
            texture_bindings,
            state_bindings,
        })
    }

    fn build_param_struct(&self, patch: &DevicePatch) -> (String, ParamLayout) {
        let mut layout = ParamLayout::default();
        let mut wgsl = String::from("struct DeviceParams {\n");
        let mut offset = 0usize;

        for device_inst in &patch.devices {
            layout.device_offsets.insert(device_inst.instance_id, offset);

            for param in device_inst.device.parameters() {
                let param_name = format!(
                    "{}_{}_{}",
                    device_inst.device.device_id(),
                    device_inst.instance_id,
                    param.name
                );
                wgsl.push_str(&format!("    {}: f32,\n", param_name));

                layout.params.push(ParamInfo {
                    name: param_name,
                    device_id: device_inst.instance_id,
                    offset,
                });

                offset += 4; // f32 = 4 bytes
            }
        }

        // Ensure 16-byte alignment
        while offset % 16 != 0 {
            wgsl.push_str(&format!("    _pad{}: f32,\n", offset / 4));
            offset += 4;
        }

        layout.size = offset;
        wgsl.push_str("}\n\n");
        wgsl.push_str("@group(2) @binding(0)\nvar<uniform> params: DeviceParams;\n");

        (wgsl, layout)
    }

    fn build_texture_bindings(&self, patch: &DevicePatch) -> (String, Vec<TextureBinding>) {
        let mut wgsl = String::new();
        let mut bindings = Vec::new();
        let mut binding_index = 0u32;

        for device_inst in &patch.devices {
            if device_inst.device.needs_feedback() {
                let name = format!(
                    "tex_{}_{}_feedback",
                    device_inst.device.device_id(),
                    device_inst.instance_id
                );
                wgsl.push_str(&format!(
                    "@group(1) @binding({})\nvar {}: texture_2d<f32>;\n",
                    binding_index, name
                ));
                wgsl.push_str(&format!(
                    "@group(1) @binding({})\nvar {}_sampler: sampler;\n",
                    binding_index + 1,
                    name
                ));

                bindings.push(TextureBinding {
                    device_id: device_inst.instance_id,
                    binding_type: TextureBindingType::Feedback,
                });

                binding_index += 2;
            }
        }

        (wgsl, bindings)
    }

    fn build_state_bindings(&self, patch: &DevicePatch) -> (String, Vec<StateBinding>) {
        let mut wgsl = String::new();
        let mut bindings = Vec::new();

        for device_inst in &patch.devices {
            if device_inst.device.needs_state() {
                if let Some(state_struct) = device_inst.device.state_struct() {
                    wgsl.push_str(state_struct);
                    wgsl.push_str("\n");

                    bindings.push(StateBinding {
                        device_id: device_inst.instance_id,
                        struct_name: format!(
                            "State_{}",
                            device_inst.device.device_id()
                        ),
                        size: 0, // Calculated later
                    });
                }
            }
        }

        (wgsl, bindings)
    }

    fn generate_main(&self, patch: &DevicePatch) -> Result<String, ComposeError> {
        let mut wgsl = String::new();

        wgsl.push_str("@fragment\n");
        wgsl.push_str("fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {\n");
        wgsl.push_str("    let uv = in.uv;\n");
        wgsl.push_str("    let time = timing.time;\n\n");

        // Topologically sort devices and generate calls
        let sorted = self.topological_sort(patch)?;

        // Declare signal variables
        for (idx, device_idx) in sorted.iter().enumerate() {
            let device = &patch.devices[*device_idx];
            wgsl.push_str(&format!(
                "    var signal_{}: vec4<f32> = vec4<f32>(0.0);\n",
                device.instance_id
            ));
        }
        wgsl.push_str("\n");

        // Generate device calls in order
        for device_idx in &sorted {
            let device = &patch.devices[*device_idx];

            // Get input signals from connections
            let mut input_args = Vec::new();
            for input in device.device.inputs() {
                let mut found = false;
                for conn in &patch.connections {
                    if conn.to_device == *device_idx && conn.to_port == input.name {
                        input_args.push(format!(
                            "signal_{}",
                            patch.devices[conn.from_device].instance_id
                        ));
                        found = true;
                        break;
                    }
                }
                if !found {
                    input_args.push("vec4<f32>(0.0)".to_string());
                }
            }

            // Call device function
            let fn_name = format!(
                "device_{}_process",
                device.device.device_id()
            );

            wgsl.push_str(&format!(
                "    signal_{} = {}(uv, time{}{});\n",
                device.instance_id,
                fn_name,
                if input_args.is_empty() { "" } else { ", " },
                input_args.join(", ")
            ));
        }

        // Return output device's signal
        let output_device = &patch.devices[patch.output_device];
        wgsl.push_str(&format!(
            "\n    return signal_{};\n",
            output_device.instance_id
        ));
        wgsl.push_str("}\n");

        Ok(wgsl)
    }

    fn topological_sort(&self, patch: &DevicePatch) -> Result<Vec<usize>, ComposeError> {
        let n = patch.devices.len();
        let mut in_degree = vec![0usize; n];
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];

        for conn in &patch.connections {
            in_degree[conn.to_device] += 1;
            adj[conn.from_device].push(conn.to_device);
        }

        let mut queue: Vec<usize> = Vec::new();
        for i in 0..n {
            if in_degree[i] == 0 {
                queue.push(i);
            }
        }

        let mut sorted = Vec::new();
        while let Some(node) = queue.pop() {
            sorted.push(node);
            for &neighbor in &adj[node] {
                in_degree[neighbor] -= 1;
                if in_degree[neighbor] == 0 {
                    queue.push(neighbor);
                }
            }
        }

        if sorted.len() != n {
            return Err(ComposeError::CyclicGraph);
        }

        Ok(sorted)
    }

    fn patch_hash(&self, patch: &DevicePatch) -> u64 {
        // Simple hash for now
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        patch.devices.len().hash(&mut hasher);
        patch.connections.len().hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for ShaderComposer {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors during shader composition
#[derive(Debug)]
pub enum ComposeError {
    CyclicGraph,
    MissingDevice(u32),
    InvalidConnection(String),
}

impl std::fmt::Display for ComposeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComposeError::CyclicGraph => write!(f, "Patch contains a cycle (use Frame Buffer for feedback)"),
            ComposeError::MissingDevice(id) => write!(f, "Device {} not found", id),
            ComposeError::InvalidConnection(msg) => write!(f, "Invalid connection: {}", msg),
        }
    }
}

impl std::error::Error for ComposeError {}

#[cfg(test)]
mod tests {
    use super::*;

    // Test device for unit tests
    struct TestRamp;

    impl ShaderDevice for TestRamp {
        fn device_id(&self) -> &'static str { "test_ramp" }
        fn device_name(&self) -> &str { "Test Ramp" }
        fn shader_code(&self) -> &str {
            r#"
fn device_test_ramp_process(uv: vec2<f32>, time: f32) -> vec4<f32> {
    return vec4<f32>(vec3<f32>(uv.x), 1.0);
}
"#
        }
        fn parameters(&self) -> &[DeviceParameter] { &[] }
        fn inputs(&self) -> &[Port] { &[] }
        fn outputs(&self) -> &[Port] {
            &[Port { name: "out".to_string(), signal_type: SignalType::Rgb }]
        }
    }

    #[test]
    fn test_compose_simple() {
        let patch = DevicePatch {
            devices: vec![
                DeviceInstance {
                    device: Box::new(TestRamp),
                    instance_id: 0,
                    params: HashMap::new(),
                    position: (0.0, 0.0),
                },
            ],
            connections: vec![],
            output_device: 0,
        };

        let mut composer = ShaderComposer::new();
        let result = composer.compose(&patch);
        assert!(result.is_ok());

        let shader = result.unwrap();
        assert!(shader.wgsl_source.contains("device_test_ramp_process"));
        assert!(shader.wgsl_source.contains("fs_main"));
    }
}
