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
    /// Source device instance_id
    pub from_device: u32,
    /// Source port name
    pub from_port: String,
    /// Destination device instance_id
    pub to_device: u32,
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
    /// Output device instance_id
    pub output_device: u32,
    /// Next instance ID for auto-assignment
    pub next_instance_id: u32,
}

impl DevicePatch {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            connections: Vec::new(),
            output_device: 0,
            next_instance_id: 0,
        }
    }

    /// Add a device and return its instance_id
    pub fn add_device(&mut self, device: Box<dyn ShaderDevice>, x: f32, y: f32) -> u32 {
        let id = self.next_instance_id;
        self.next_instance_id += 1;

        // Initialize default param values
        let mut params = HashMap::new();
        for p in device.parameters() {
            params.insert(p.name.clone(), p.default);
        }

        self.devices.push(DeviceInstance {
            device,
            instance_id: id,
            params,
            position: (x, y),
        });
        id
    }

    /// Remove a device by instance_id
    pub fn remove_device(&mut self, instance_id: u32) {
        self.devices.retain(|d| d.instance_id != instance_id);
        self.connections.retain(|c| c.from_device != instance_id && c.to_device != instance_id);
    }

    /// Connect two devices by instance_id and port name
    pub fn connect(&mut self, from_id: u32, from_port: &str, to_id: u32, to_port: &str) {
        // Remove any existing connection to this input
        self.connections.retain(|c| !(c.to_device == to_id && c.to_port == to_port));
        self.connections.push(Connection {
            from_device: from_id,
            from_port: from_port.to_string(),
            to_device: to_id,
            to_port: to_port.to_string(),
        });
    }

    /// Disconnect a device input
    pub fn disconnect(&mut self, to_id: u32, to_port: &str) {
        self.connections.retain(|c| !(c.to_device == to_id && c.to_port == to_port));
    }

    /// Get device by instance_id
    pub fn get_device(&self, instance_id: u32) -> Option<&DeviceInstance> {
        self.devices.iter().find(|d| d.instance_id == instance_id)
    }

    /// Get mutable device by instance_id
    pub fn get_device_mut(&mut self, instance_id: u32) -> Option<&mut DeviceInstance> {
        self.devices.iter_mut().find(|d| d.instance_id == instance_id)
    }

    /// Collect all parameter values into a flat f32 buffer matching the param layout
    pub fn collect_param_values(&self, layout: &ParamLayout) -> Vec<f32> {
        let count = layout.size / 4;
        let mut values = vec![0.0f32; count];

        for param_info in &layout.params {
            let offset_idx = param_info.offset / 4;
            if let Some(device) = self.get_device(param_info.device_id) {
                // Extract the short param name (after device_id_instanceid_ prefix)
                let prefix = format!(
                    "{}_{}_",
                    device.device.device_id(),
                    device.instance_id
                );
                let short_name = param_info.name.strip_prefix(&prefix).unwrap_or(&param_info.name);

                if let Some(&val) = device.params.get(short_name) {
                    values[offset_idx] = val;
                } else {
                    // Use default from device parameter definition
                    if let Some(dp) = device.device.parameters().iter().find(|p| p.name == short_name) {
                        values[offset_idx] = dp.default;
                    }
                }
            }
        }

        values
    }
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
#[derive(Clone, Default)]
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

        // 6. Add device shader code (deduplicated by device_id —
        //    multiple instances of the same device type share one function)
        wgsl.push_str("// === DEVICES ===\n");
        let mut included_device_ids = std::collections::HashSet::new();
        for device_inst in &patch.devices {
            let device_id = device_inst.device.device_id();
            if included_device_ids.insert(device_id) {
                wgsl.push_str(&format!(
                    "// --- {} ---\n",
                    device_inst.device.device_name(),
                ));
                wgsl.push_str(device_inst.device.shader_code());
                wgsl.push_str("\n\n");
            }
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
        for device_idx in &sorted {
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

            // Get input signals from connections (match by instance_id)
            let mut input_args = Vec::new();
            for input in device.device.inputs() {
                let mut found = false;
                for conn in &patch.connections {
                    if conn.to_device == device.instance_id && conn.to_port == input.name {
                        input_args.push(format!("signal_{}", conn.from_device));
                        found = true;
                        break;
                    }
                }
                if !found {
                    input_args.push("vec4<f32>(0.0)".to_string());
                }
            }

            // Collect parameter references from the params uniform
            let mut param_args = Vec::new();
            for param in device.device.parameters() {
                let param_name = format!(
                    "{}_{}_{}",
                    device.device.device_id(),
                    device.instance_id,
                    param.name
                );
                param_args.push(format!("params.{}", param_name));
            }

            // Build all args after uv, time
            let mut all_extra_args = Vec::new();
            all_extra_args.extend(input_args);
            all_extra_args.extend(param_args);

            // Call device function
            let fn_name = format!(
                "device_{}_process",
                device.device.device_id()
            );

            wgsl.push_str(&format!(
                "    signal_{} = {}(uv, time{}{});\n",
                device.instance_id,
                fn_name,
                if all_extra_args.is_empty() { "" } else { ", " },
                all_extra_args.join(", ")
            ));
        }

        // Return output device's signal
        let output_idx = Self::instance_to_index(patch, patch.output_device)
            .ok_or(ComposeError::MissingDevice(patch.output_device))?;
        let output_device = &patch.devices[output_idx];
        wgsl.push_str(&format!(
            "\n    return signal_{};\n",
            output_device.instance_id
        ));
        wgsl.push_str("}\n");

        Ok(wgsl)
    }

    /// Resolve instance_id to device index
    fn instance_to_index(patch: &DevicePatch, instance_id: u32) -> Option<usize> {
        patch.devices.iter().position(|d| d.instance_id == instance_id)
    }

    fn topological_sort(&self, patch: &DevicePatch) -> Result<Vec<usize>, ComposeError> {
        let n = patch.devices.len();
        let mut in_degree = vec![0usize; n];
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];

        for conn in &patch.connections {
            let from_idx = Self::instance_to_index(patch, conn.from_device)
                .ok_or(ComposeError::MissingDevice(conn.from_device))?;
            let to_idx = Self::instance_to_index(patch, conn.to_device)
                .ok_or(ComposeError::MissingDevice(conn.to_device))?;
            in_degree[to_idx] += 1;
            adj[from_idx].push(to_idx);
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
        fn parameters(&self) -> Vec<DeviceParameter> { vec![] }
        fn inputs(&self) -> Vec<Port> { vec![] }
        fn outputs(&self) -> Vec<Port> {
            vec![Port { name: "out".to_string(), signal_type: SignalType::Rgb }]
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
            next_instance_id: 1,
        };

        let mut composer = ShaderComposer::new();
        let result = composer.compose(&patch);
        assert!(result.is_ok());

        let shader = result.unwrap();
        assert!(shader.wgsl_source.contains("device_test_ramp_process"));
        assert!(shader.wgsl_source.contains("fs_main"));
    }
}
