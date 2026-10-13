//! Shader Composer
//!
//! Composes device shaders at compile time instead of runtime dispatch.
//! This is the core architectural change from v1 -> v2.
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
//!
//! v2.1 approach: Multi-pass render segments
//!   - PixelProcessors compose into single-pass segments
//!   - ScanModulators break the chain, requiring intermediate textures
//!   - Each segment gets its own WGSL shader and pipeline

use std::collections::{HashMap, HashSet};

/// Common shader code included in all composed shaders
pub const COMMON_WGSL: &str = include_str!("../shaders/common.wgsl");

/// Component library code
pub const COMPONENTS_WGSL: &str = include_str!("../shaders/components.wgsl");

/// Cable degradation functions
pub const CABLES_WGSL: &str = include_str!("../shaders/cables.wgsl");

// ─── Processing Mode ─────────────────────────────────────────

/// How a device processes its input signal.
///
/// This determines whether a device can be composed into a single-pass
/// shader with other pixel processors, or requires its own render pass
/// with texture sampling capability.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProcessingMode {
    /// Transforms pixel values: vec4 in -> vec4 out.
    /// Can be composed with other PixelProcessors in a single pass.
    PixelProcessor,

    /// Needs to sample input at arbitrary UV coordinates (e.g., displacement).
    /// Requires its own render pass with the upstream output as a texture.
    ScanModulator,
}

// ─── ShaderDevice Trait ──────────────────────────────────────

/// A device that can be composed into a shader
pub trait ShaderDevice {
    /// Unique identifier for this device type
    fn device_id(&self) -> &'static str;

    /// Human-readable name
    fn device_name(&self) -> &str;

    /// WGSL code for this device's processing.
    ///
    /// For PixelProcessor devices:
    ///   Must define: fn device_{id}_process(uv, time, video_in, params...) -> vec4<f32>
    ///
    /// For ScanModulator devices:
    ///   Must define: fn device_{id}_process(uv, time, params...) -> vec4<f32>
    ///   Can access seg_input_tex / seg_input_sampler globals for texture sampling.
    fn shader_code(&self) -> &str;

    /// Parameters this device exposes
    fn parameters(&self) -> Vec<DeviceParameter>;

    /// Input ports
    fn inputs(&self) -> Vec<Port>;

    /// Output ports.
    ///
    /// Port 0 carries `device_{id}_process`'s result. A source device (no inputs) with more
    /// outputs must also define
    ///   fn device_{id}_output(uv, time, port: u32, params...) -> vec4<f32>
    /// which the composer calls for connections from port 1 and up, within one pass.
    /// Across passes only port 0 is carried (the intermediate texture holds one signal).
    fn outputs(&self) -> Vec<Port>;

    /// How this device processes signals.
    /// PixelProcessors can be composed; ScanModulators need their own render pass.
    fn processing_mode(&self) -> ProcessingMode {
        ProcessingMode::PixelProcessor
    }

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

// ─── Device Parameter ────────────────────────────────────────

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

// ─── Port / Signal Type ──────────────────────────────────────

/// A device port (input or output)
#[derive(Clone, Debug)]
pub struct Port {
    /// Port name
    pub name: String,
    /// Signal type
    pub signal_type: SignalType,
}

/// Input binding information for multi-input support
#[derive(Clone, Debug)]
pub struct InputBinding {
    /// Port name this binding is for
    pub port_name: String,
    /// Source device instance ID (where the signal comes from)
    pub source_device_id: u32,
    /// Whether this is from a previous segment (requires texture) or same segment (use signal var)
    pub cross_segment: bool,
    /// Texture binding index (for cross-segment inputs)
    pub texture_binding_idx: Option<u32>,
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

// ─── Cable Types ─────────────────────────────────────────────

/// Cable/connector type affecting signal quality
/// Different cables introduce distinct artifacts based on their physical properties
#[derive(Clone, Debug, Default, PartialEq)]
pub enum CableType {
    /// No degradation (direct digital connection)
    #[default]
    Ideal,
    /// Composite video (single RCA): chroma/luma crosstalk, ~4MHz bandwidth, dot crawl
    Composite,
    /// S-Video (separate Y/C): no crosstalk, bandwidth-limited per channel
    SVideo,
    /// Component (YPbPr): wider bandwidth, minimal degradation, HF rolloff on long cables
    Component,
    /// RGB (direct color): best quality, cable-length dependent HF rolloff
    Rgb,
}

// ─── Connection ──────────────────────────────────────────────

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
    /// Cable type (affects signal degradation)
    pub cable_type: CableType,
}

// ─── Device Instance / Patch ─────────────────────────────────

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
        self.connections
            .retain(|c| c.from_device != instance_id && c.to_device != instance_id);
    }

    /// Connect two devices by instance_id and port name
    pub fn connect(&mut self, from_id: u32, from_port: &str, to_id: u32, to_port: &str) {
        // Remove any existing connection to this input
        self.connections
            .retain(|c| !(c.to_device == to_id && c.to_port == to_port));
        self.connections.push(Connection {
            from_device: from_id,
            from_port: from_port.to_string(),
            to_device: to_id,
            to_port: to_port.to_string(),
            cable_type: CableType::default(),
        });
    }

    /// Disconnect a device input
    pub fn disconnect(&mut self, to_id: u32, to_port: &str) {
        self.connections
            .retain(|c| !(c.to_device == to_id && c.to_port == to_port));
    }

    /// Get device by instance_id
    pub fn get_device(&self, instance_id: u32) -> Option<&DeviceInstance> {
        self.devices.iter().find(|d| d.instance_id == instance_id)
    }

    /// Get mutable device by instance_id
    pub fn get_device_mut(&mut self, instance_id: u32) -> Option<&mut DeviceInstance> {
        self.devices
            .iter_mut()
            .find(|d| d.instance_id == instance_id)
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
                let short_name = param_info
                    .name
                    .strip_prefix(&prefix)
                    .unwrap_or(&param_info.name);

                if let Some(&val) = device.params.get(short_name) {
                    values[offset_idx] = val;
                } else {
                    // Use default from device parameter definition
                    if let Some(dp) = device
                        .device
                        .parameters()
                        .iter()
                        .find(|p| p.name == short_name)
                    {
                        values[offset_idx] = dp.default;
                    }
                }
            }
        }

        values
    }
}

// ─── Render Plan (Multi-Pass) ────────────────────────────────

/// A complete render plan: ordered list of segments to execute.
/// Each segment is a render pass with its own shader and pipeline.
pub struct RenderPlan {
    /// Ordered segments to execute
    pub segments: Vec<SegmentShader>,
}

/// A single render segment with its compiled shader
pub struct SegmentShader {
    /// What kind of segment this is
    pub kind: SegmentKind,
    /// Complete WGSL source for this segment
    pub wgsl_source: String,
    /// Parameter uniform layout for this segment's devices
    pub param_layout: ParamLayout,
    /// Whether this segment reads from a previous segment's output texture
    pub needs_input_texture: bool,
    /// Number of input textures this segment needs (for multi-input support)
    pub input_texture_count: u32,
    /// Instance IDs of devices in this segment (for param collection)
    pub device_instance_ids: Vec<u32>,
    /// Maps texture slot index to intermediate texture index.
    /// e.g., [0, 2] means slot 0 reads intermediate[0], slot 1 reads intermediate[2].
    /// Used by renderer to create bind groups for multi-input segments.
    pub input_source_segments: Vec<usize>,
}

/// The kind of render segment
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SegmentKind {
    /// Multiple pixel processors composed into one pass
    Composed,
    /// A single scan modulator device with texture sampling
    ScanModulator,
}

// ─── Composed Shader (legacy single-pass) ────────────────────

/// Composed shader ready for GPU (single-pass, backward compat)
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

// ─── Shader Composer ─────────────────────────────────────────

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

    /// Compose a shader for a device patch (legacy single-pass API)
    pub fn compose(&mut self, patch: &DevicePatch) -> Result<&ComposedShader, ComposeError> {
        // For now, always recompile (caching can be added later)
        let shader = self.compose_internal(patch)?;
        let hash = self.patch_hash(patch);
        self.cache.insert(hash, shader);
        Ok(self.cache.get(&hash).unwrap())
    }

    /// Compose a multi-pass render plan for a device patch.
    ///
    /// This is the primary composition API for v2.1+.
    /// Returns a RenderPlan with one or more segments:
    /// - If the patch has no ScanModulators, returns a single Composed segment
    ///   (equivalent to the old single-pass behavior).
    /// - If the patch has ScanModulators, breaks the chain into multiple segments
    ///   with intermediate textures between them.
    pub fn compose_plan(&self, patch: &DevicePatch) -> Result<RenderPlan, ComposeError> {
        if patch.devices.is_empty() {
            return Err(ComposeError::MissingDevice(0));
        }

        let all_sorted = self.topological_sort(patch)?;

        // Filter to only devices in the signal path to the output device
        let reachable = Self::reachable_from_output(patch);
        let sorted: Vec<usize> = all_sorted
            .into_iter()
            .filter(|&idx| reachable.contains(&patch.devices[idx].instance_id))
            .collect();

        if sorted.is_empty() {
            return Err(ComposeError::MissingDevice(patch.output_device));
        }

        log::info!(
            "Render plan: {} reachable devices out of {} total (output={})",
            sorted.len(),
            patch.devices.len(),
            patch.output_device
        );

        // Segmentation: group consecutive PixelProcessors, break at ScanModulators.
        // Always uses the segment-based path (not compose_internal) so that only
        // reachable devices are included in the generated WGSL.
        let mut segments = Vec::new();
        let mut current_group: Vec<usize> = Vec::new();
        let mut all_prev_device_ids: HashSet<u32> = HashSet::new();
        let mut segment_count = 0usize;

        // Build device-to-segment mapping: tracks which segment produces each device's output.
        // This allows multi-input segments to reference non-adjacent upstream segments.
        let mut device_to_segment: HashMap<u32, usize> = HashMap::new();

        for &device_idx in &sorted {
            let device = &patch.devices[device_idx];
            let mode = device.device.processing_mode();

            match mode {
                ProcessingMode::PixelProcessor => {
                    current_group.push(device_idx);
                }
                ProcessingMode::ScanModulator => {
                    // Finalize current pixel processor group (if non-empty)
                    if !current_group.is_empty() {
                        let segment = self.build_composed_segment(
                            patch,
                            &current_group,
                            segment_count > 0, // needs input texture if not first segment
                            &all_prev_device_ids,
                            &device_to_segment,
                        )?;
                        // Record which segment produces each device in this group
                        for &idx in &current_group {
                            let dev_id = patch.devices[idx].instance_id;
                            all_prev_device_ids.insert(dev_id);
                            device_to_segment.insert(dev_id, segment_count);
                        }
                        segments.push(segment);
                        current_group.clear();
                        segment_count += 1;
                    }

                    // Create scan modulator segment
                    let segment = self.build_scan_modulator_segment(
                        patch,
                        device_idx,
                        &device_to_segment,
                    )?;
                    all_prev_device_ids.insert(device.instance_id);
                    device_to_segment.insert(device.instance_id, segment_count);
                    segments.push(segment);
                    segment_count += 1;
                }
            }
        }

        // Finalize remaining pixel processors
        if !current_group.is_empty() {
            let segment = self.build_composed_segment(
                patch,
                &current_group,
                segment_count > 0,
                &all_prev_device_ids,
                &device_to_segment,
            )?;
            segments.push(segment);
        }

        log::info!(
            "Render plan: {} segments ({} scan modulators)",
            segments.len(),
            segments
                .iter()
                .filter(|s| s.kind == SegmentKind::ScanModulator)
                .count()
        );

        Ok(RenderPlan { segments })
    }

    // ─── Segment Builders ────────────────────────────────────

    /// Build a composed segment from a group of pixel processor devices.
    fn build_composed_segment(
        &self,
        patch: &DevicePatch,
        device_indices: &[usize],
        needs_input_texture: bool,
        prev_device_ids: &HashSet<u32>,
        device_to_segment: &HashMap<u32, usize>,
    ) -> Result<SegmentShader, ComposeError> {
        let mut wgsl = String::new();

        // 1. Common + components + cables
        wgsl.push_str("// === COMMON ===\n");
        wgsl.push_str(COMMON_WGSL);
        wgsl.push_str("\n\n");
        wgsl.push_str("// === COMPONENTS ===\n");
        wgsl.push_str(COMPONENTS_WGSL);
        wgsl.push_str("\n\n");
        wgsl.push_str("// === CABLES ===\n");
        wgsl.push_str(CABLES_WGSL);
        wgsl.push_str("\n\n");

        // 2. Count cross-segment inputs and create texture bindings
        // For multi-input support, we need one texture per distinct upstream segment source
        let cross_segment_sources = self.count_cross_segment_sources(
            patch,
            device_indices,
            prev_device_ids,
        );
        let input_texture_count = if needs_input_texture {
            cross_segment_sources.len().max(1) as u32
        } else {
            0
        };

        // Compute input_source_segments: maps texture slot index to intermediate texture index
        // This allows renderer to bind the correct textures for multi-input segments
        let input_source_segments: Vec<usize> = cross_segment_sources
            .iter()
            .filter_map(|&device_id| device_to_segment.get(&device_id).copied())
            .collect();

        if needs_input_texture {
            wgsl.push_str("// === SEGMENT INPUTS ===\n");
            if cross_segment_sources.is_empty() {
                // Single input (backwards compatible)
                wgsl.push_str(
                    "@group(1) @binding(0) var seg_input_tex: texture_2d<f32>;\n",
                );
                wgsl.push_str(
                    "@group(1) @binding(1) var seg_input_sampler: sampler;\n\n",
                );
            } else {
                // Multiple inputs - each gets its own texture binding
                // Primary input (index 0) uses standard names for backwards compatibility
                wgsl.push_str(
                    "@group(1) @binding(0) var seg_input_tex: texture_2d<f32>;\n",
                );
                wgsl.push_str(
                    "@group(1) @binding(1) var seg_input_sampler: sampler;\n",
                );
                // Additional inputs get numbered names
                for i in 1..cross_segment_sources.len() {
                    let binding_base = (i as u32) * 2;
                    wgsl.push_str(&format!(
                        "@group(1) @binding({}) var seg_input_tex_{}: texture_2d<f32>;\n",
                        binding_base, i
                    ));
                    wgsl.push_str(&format!(
                        "@group(1) @binding({}) var seg_input_sampler_{}: sampler;\n",
                        binding_base + 1, i
                    ));
                }
                wgsl.push_str("\n");
            }
        }

        // 3. Parameter struct (only this segment's devices)
        let devices_in_segment: Vec<&DeviceInstance> = device_indices
            .iter()
            .map(|&idx| &patch.devices[idx])
            .collect();
        let (param_struct, param_layout) = self.build_segment_param_struct(&devices_in_segment);
        wgsl.push_str("// === PARAMETERS ===\n");
        wgsl.push_str(&param_struct);
        wgsl.push_str("\n\n");

        // 4. Device shader code (deduplicated)
        wgsl.push_str("// === DEVICES ===\n");
        let mut included_ids = HashSet::new();
        for device in &devices_in_segment {
            let device_id = device.device.device_id();
            if included_ids.insert(device_id) {
                wgsl.push_str(&format!("// --- {} ---\n", device.device.device_name()));
                wgsl.push_str(device.device.shader_code());
                wgsl.push_str("\n\n");
            }
        }

        // 5. Main function (pass cross_segment_sources for multi-input routing)
        let main_fn = self.generate_segment_main_multi_input(
            patch,
            device_indices,
            needs_input_texture,
            prev_device_ids,
            &cross_segment_sources,
        )?;
        wgsl.push_str("// === MAIN ===\n");
        wgsl.push_str(&main_fn);

        let device_instance_ids = devices_in_segment
            .iter()
            .map(|d| d.instance_id)
            .collect();

        Ok(SegmentShader {
            kind: SegmentKind::Composed,
            wgsl_source: wgsl,
            param_layout,
            needs_input_texture,
            input_texture_count,
            device_instance_ids,
            input_source_segments,
        })
    }

    /// Count distinct cross-segment source device IDs.
    /// Returns a Vec mapping source device ID to texture index.
    fn count_cross_segment_sources(
        &self,
        patch: &DevicePatch,
        device_indices: &[usize],
        prev_device_ids: &HashSet<u32>,
    ) -> Vec<u32> {
        let mut sources = Vec::new();
        for &idx in device_indices {
            let device = &patch.devices[idx];
            for input in device.device.inputs() {
                for conn in &patch.connections {
                    if conn.to_device == device.instance_id && conn.to_port == input.name {
                        if prev_device_ids.contains(&conn.from_device) {
                            if !sources.contains(&conn.from_device) {
                                sources.push(conn.from_device);
                            }
                        }
                    }
                }
            }
        }
        sources
    }

    /// Build a scan modulator segment (single device, reads from input texture).
    fn build_scan_modulator_segment(
        &self,
        patch: &DevicePatch,
        device_idx: usize,
        device_to_segment: &HashMap<u32, usize>,
    ) -> Result<SegmentShader, ComposeError> {
        let device = &patch.devices[device_idx];
        let mut wgsl = String::new();

        // Find the source segment for this scan modulator's input
        // Scan modulators read from a single upstream device (first connected input)
        let mut input_source_segments = Vec::new();
        for input in device.device.inputs() {
            for conn in &patch.connections {
                if conn.to_device == device.instance_id && conn.to_port == input.name {
                    if let Some(&seg_idx) = device_to_segment.get(&conn.from_device) {
                        if !input_source_segments.contains(&seg_idx) {
                            input_source_segments.push(seg_idx);
                        }
                    }
                    break; // Only first input for now
                }
            }
            if !input_source_segments.is_empty() {
                break; // Scan modulators currently support single input
            }
        }

        // 1. Common + components + cables
        wgsl.push_str("// === COMMON ===\n");
        wgsl.push_str(COMMON_WGSL);
        wgsl.push_str("\n\n");
        wgsl.push_str("// === COMPONENTS ===\n");
        wgsl.push_str(COMPONENTS_WGSL);
        wgsl.push_str("\n\n");
        wgsl.push_str("// === CABLES ===\n");
        wgsl.push_str(CABLES_WGSL);
        wgsl.push_str("\n\n");

        // 2. Input texture (scan modulators always read from a texture)
        wgsl.push_str("// === SEGMENT INPUT ===\n");
        wgsl.push_str("@group(1) @binding(0) var seg_input_tex: texture_2d<f32>;\n");
        wgsl.push_str("@group(1) @binding(1) var seg_input_sampler: sampler;\n\n");

        // 3. Parameters
        let devices = vec![device];
        let (param_struct, param_layout) = self.build_segment_param_struct(&devices);
        wgsl.push_str("// === PARAMETERS ===\n");
        wgsl.push_str(&param_struct);
        wgsl.push_str("\n\n");

        // 4. Device code
        wgsl.push_str("// === DEVICE ===\n");
        wgsl.push_str(&format!("// --- {} ---\n", device.device.device_name()));
        wgsl.push_str(device.device.shader_code());
        wgsl.push_str("\n\n");

        // 5. Main function — scan modulator functions don't take video_in,
        //    they sample from seg_input_tex directly
        wgsl.push_str("// === MAIN ===\n");
        wgsl.push_str("@fragment\n");
        wgsl.push_str("fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {\n");
        wgsl.push_str("    let uv = in.uv;\n");
        wgsl.push_str("    let time = timing.time;\n\n");

        // Build param args (no video_in for scan modulators)
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

        let fn_name = format!("device_{}_process", device.device.device_id());
        wgsl.push_str(&format!(
            "    return {}(uv, time{}{});\n",
            fn_name,
            if param_args.is_empty() { "" } else { ", " },
            param_args.join(", ")
        ));
        wgsl.push_str("}\n");

        Ok(SegmentShader {
            kind: SegmentKind::ScanModulator,
            wgsl_source: wgsl,
            param_layout,
            needs_input_texture: true,
            input_texture_count: 1, // Scan modulators currently support single input
            device_instance_ids: vec![device.instance_id],
            input_source_segments,
        })
    }

    /// Wrap a signal expression with cable degradation based on cable type.
    /// Returns the wrapped WGSL expression.
    fn apply_cable_degradation(signal: &str, cable_type: &CableType) -> String {
        let result = match cable_type {
            CableType::Ideal => signal.to_string(),
            CableType::Composite => format!("composite_degrade({}, uv, time)", signal),
            CableType::SVideo => format!("svideo_degrade({}, uv, time)", signal),
            CableType::Component => format!("component_degrade({}, uv, 6.0)", signal), // 6ft default
            CableType::Rgb => format!("rgb_degrade({}, uv, 6.0)", signal),             // 6ft default
        };
        if *cable_type != CableType::Ideal {
            eprintln!("[SHADER] Cable degradation applied: {:?} -> {}", cable_type, result);
        }
        result
    }

    /// Generate the main function for a composed (pixel processor) segment.
    fn generate_segment_main(
        &self,
        patch: &DevicePatch,
        device_indices: &[usize],
        needs_input_texture: bool,
        prev_device_ids: &HashSet<u32>,
    ) -> Result<String, ComposeError> {
        let mut wgsl = String::new();

        wgsl.push_str("@fragment\n");
        wgsl.push_str("fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {\n");
        wgsl.push_str("    let uv = in.uv;\n");
        wgsl.push_str("    let time = timing.time;\n\n");

        // Declare signal variables for this segment's devices
        for &idx in device_indices {
            let device = &patch.devices[idx];
            wgsl.push_str(&format!(
                "    var signal_{}: vec4<f32> = vec4<f32>(0.0);\n",
                device.instance_id
            ));
        }
        wgsl.push_str("\n");

        // Generate device calls in topological order
        for &idx in device_indices {
            let device = &patch.devices[idx];

            // Get input signals (with cable degradation applied)
            let mut input_args = Vec::new();
            for input in device.device.inputs() {
                let mut found = false;
                for conn in &patch.connections {
                    if conn.to_device == device.instance_id && conn.to_port == input.name {
                        let raw_signal = if prev_device_ids.contains(&conn.from_device) {
                            // Cross-segment input: sample from previous segment's texture
                            if needs_input_texture {
                                "textureSample(seg_input_tex, seg_input_sampler, uv)".to_string()
                            } else {
                                // Previous segment has no texture — default to black
                                "vec4<f32>(0.0)".to_string()
                            }
                        } else {
                            // Same-segment input
                            Self::same_pass_signal(patch, conn)
                        };

                        // Apply cable degradation based on connection type
                        let degraded_signal =
                            Self::apply_cable_degradation(&raw_signal, &conn.cable_type);
                        input_args.push(degraded_signal);
                        found = true;
                        break;
                    }
                }
                if !found {
                    input_args.push("vec4<f32>(0.0)".to_string());
                }
            }

            // Parameter args
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

            let mut all_args = Vec::new();
            all_args.extend(input_args);
            all_args.extend(param_args);

            let fn_name = format!("device_{}_process", device.device.device_id());

            wgsl.push_str(&format!(
                "    signal_{} = {}(uv, time{}{});\n",
                device.instance_id,
                fn_name,
                if all_args.is_empty() { "" } else { ", " },
                all_args.join(", ")
            ));
        }

        // Return the output device's signal (or last device in segment)
        let output_id = device_indices
            .iter()
            .rev()
            .find(|&&idx| patch.devices[idx].instance_id == patch.output_device)
            .map(|&idx| patch.devices[idx].instance_id)
            .unwrap_or_else(|| {
                patch.devices[*device_indices.last().unwrap()].instance_id
            });

        wgsl.push_str(&format!("\n    return signal_{};\n", output_id));
        wgsl.push_str("}\n");

        Ok(wgsl)
    }

    /// Generate the main function for a composed segment with multi-input support.
    /// cross_segment_sources maps source device IDs to their texture index.
    fn generate_segment_main_multi_input(
        &self,
        patch: &DevicePatch,
        device_indices: &[usize],
        needs_input_texture: bool,
        prev_device_ids: &HashSet<u32>,
        cross_segment_sources: &[u32],
    ) -> Result<String, ComposeError> {
        let mut wgsl = String::new();

        wgsl.push_str("@fragment\n");
        wgsl.push_str("fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {\n");
        wgsl.push_str("    let uv = in.uv;\n");
        wgsl.push_str("    let time = timing.time;\n\n");

        // Declare signal variables for this segment's devices
        for &idx in device_indices {
            let device = &patch.devices[idx];
            wgsl.push_str(&format!(
                "    var signal_{}: vec4<f32> = vec4<f32>(0.0);\n",
                device.instance_id
            ));
        }
        wgsl.push_str("\n");

        // Generate device calls in topological order
        for &idx in device_indices {
            let device = &patch.devices[idx];

            // Get input signals (with cable degradation applied)
            // Multi-input: map each input port to its correct texture or signal
            let mut input_args = Vec::new();
            for input in device.device.inputs() {
                let mut found = false;
                for conn in &patch.connections {
                    if conn.to_device == device.instance_id && conn.to_port == input.name {
                        let raw_signal = if prev_device_ids.contains(&conn.from_device) {
                            // Cross-segment input: sample from the correct texture
                            if needs_input_texture {
                                // Find which texture index this source maps to
                                let tex_idx = cross_segment_sources
                                    .iter()
                                    .position(|&id| id == conn.from_device)
                                    .unwrap_or(0);

                                if tex_idx == 0 {
                                    // Primary texture uses standard names
                                    "textureSample(seg_input_tex, seg_input_sampler, uv)".to_string()
                                } else {
                                    // Additional textures use numbered names
                                    format!(
                                        "textureSample(seg_input_tex_{}, seg_input_sampler_{}, uv)",
                                        tex_idx, tex_idx
                                    )
                                }
                            } else {
                                "vec4<f32>(0.0)".to_string()
                            }
                        } else {
                            // Same-segment input
                            Self::same_pass_signal(patch, conn)
                        };

                        // Apply cable degradation based on connection type
                        let degraded_signal =
                            Self::apply_cable_degradation(&raw_signal, &conn.cable_type);
                        input_args.push(degraded_signal);
                        found = true;
                        break;
                    }
                }
                if !found {
                    input_args.push("vec4<f32>(0.0)".to_string());
                }
            }

            // Parameter args
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

            let mut all_args = Vec::new();
            all_args.extend(input_args);
            all_args.extend(param_args);

            let fn_name = format!("device_{}_process", device.device.device_id());

            wgsl.push_str(&format!(
                "    signal_{} = {}(uv, time{}{});\n",
                device.instance_id,
                fn_name,
                if all_args.is_empty() { "" } else { ", " },
                all_args.join(", ")
            ));
        }

        // Return the output device's signal (or last device in segment)
        let output_id = device_indices
            .iter()
            .rev()
            .find(|&&idx| patch.devices[idx].instance_id == patch.output_device)
            .map(|&idx| patch.devices[idx].instance_id)
            .unwrap_or_else(|| {
                patch.devices[*device_indices.last().unwrap()].instance_id
            });

        wgsl.push_str(&format!("\n    return signal_{};\n", output_id));
        wgsl.push_str("}\n");

        Ok(wgsl)
    }

    /// Build a DeviceParams struct for a subset of devices.
    fn build_segment_param_struct(&self, devices: &[&DeviceInstance]) -> (String, ParamLayout) {
        let mut layout = ParamLayout::default();
        let mut wgsl = String::from("struct DeviceParams {\n");
        let mut offset = 0usize;

        for device in devices {
            layout.device_offsets.insert(device.instance_id, offset);

            for param in device.device.parameters() {
                let param_name = format!(
                    "{}_{}_{}",
                    device.device.device_id(),
                    device.instance_id,
                    param.name
                );
                wgsl.push_str(&format!("    {}: f32,\n", param_name));

                layout.params.push(ParamInfo {
                    name: param_name,
                    device_id: device.instance_id,
                    offset,
                });

                offset += 4;
            }
        }

        // WGSL requires at least one field in a struct
        if offset == 0 {
            wgsl.push_str("    _pad0: f32,\n");
            offset = 4;
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

    // ─── Legacy Single-Pass Composition ──────────────────────

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

        // 2b. Add cable degradation functions
        wgsl.push_str("// === CABLES ===\n");
        wgsl.push_str(CABLES_WGSL);
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

        // 6. Add device shader code (deduplicated by device_id)
        wgsl.push_str("// === DEVICES ===\n");
        let mut included_device_ids = HashSet::new();
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
            layout
                .device_offsets
                .insert(device_inst.instance_id, offset);

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

    fn build_texture_bindings(
        &self,
        patch: &DevicePatch,
    ) -> (String, Vec<TextureBinding>) {
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

    fn build_state_bindings(
        &self,
        patch: &DevicePatch,
    ) -> (String, Vec<StateBinding>) {
        let mut wgsl = String::new();
        let mut bindings = Vec::new();

        for device_inst in &patch.devices {
            if device_inst.device.needs_state() {
                if let Some(state_struct) = device_inst.device.state_struct() {
                    wgsl.push_str(state_struct);
                    wgsl.push_str("\n");

                    bindings.push(StateBinding {
                        device_id: device_inst.instance_id,
                        struct_name: format!("State_{}", device_inst.device.device_id()),
                        size: 0,
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

            // Get input signals from connections (with cable degradation)
            let mut input_args = Vec::new();
            for input in device.device.inputs() {
                let mut found = false;
                for conn in &patch.connections {
                    if conn.to_device == device.instance_id && conn.to_port == input.name {
                        let raw_signal = Self::same_pass_signal(patch, conn);
                        let degraded_signal =
                            Self::apply_cable_degradation(&raw_signal, &conn.cable_type);
                        input_args.push(degraded_signal);
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
            let fn_name = format!("device_{}_process", device.device.device_id());

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

    // ─── Utilities ───────────────────────────────────────────

    /// WGSL for the signal on `conn`'s source port, when the source runs in the same pass.
    /// Port 0 is the device's `signal_{id}`; a later port of a multi-output device is
    /// evaluated with `device_{id}_output` (see `ShaderDevice::outputs`).
    fn same_pass_signal(patch: &DevicePatch, conn: &Connection) -> String {
        if let Some(src) = patch.get_device(conn.from_device) {
            let port = src
                .device
                .outputs()
                .iter()
                .position(|p| p.name == conn.from_port);
            if let Some(port) = port.filter(|&i| i > 0) {
                let params: String = src
                    .device
                    .parameters()
                    .iter()
                    .map(|p| {
                        format!(
                            ", params.{}_{}_{}",
                            src.device.device_id(),
                            src.instance_id,
                            p.name
                        )
                    })
                    .collect();
                return format!(
                    "device_{}_output(uv, time, {}u{})",
                    src.device.device_id(),
                    port,
                    params
                );
            }
        }
        format!("signal_{}", conn.from_device)
    }

    /// Resolve instance_id to device index
    fn instance_to_index(patch: &DevicePatch, instance_id: u32) -> Option<usize> {
        patch
            .devices
            .iter()
            .position(|d| d.instance_id == instance_id)
    }

    /// Find all device instance_ids that are upstream of (feed into) the output device.
    /// Walks backwards through connections from the output device.
    fn reachable_from_output(patch: &DevicePatch) -> HashSet<u32> {
        let mut reachable = HashSet::new();
        let mut queue = vec![patch.output_device];
        reachable.insert(patch.output_device);

        while let Some(device_id) = queue.pop() {
            for conn in &patch.connections {
                if conn.to_device == device_id && !reachable.contains(&conn.from_device) {
                    reachable.insert(conn.from_device);
                    queue.push(conn.from_device);
                }
            }
        }

        reachable
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

// ─── Errors ──────────────────────────────────────────────────

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
            ComposeError::CyclicGraph => {
                write!(f, "Patch contains a cycle (use Frame Buffer for feedback)")
            }
            ComposeError::MissingDevice(id) => write!(f, "Device {} not found", id),
            ComposeError::InvalidConnection(msg) => write!(f, "Invalid connection: {}", msg),
        }
    }
}

impl std::error::Error for ComposeError {}

// ─── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Test device for unit tests
    struct TestRamp;

    impl ShaderDevice for TestRamp {
        fn device_id(&self) -> &'static str {
            "test_ramp"
        }
        fn device_name(&self) -> &str {
            "Test Ramp"
        }
        fn shader_code(&self) -> &str {
            r#"
fn device_test_ramp_process(uv: vec2<f32>, time: f32) -> vec4<f32> {
    return vec4<f32>(vec3<f32>(uv.x), 1.0);
}
"#
        }
        fn parameters(&self) -> Vec<DeviceParameter> {
            vec![]
        }
        fn inputs(&self) -> Vec<Port> {
            vec![]
        }
        fn outputs(&self) -> Vec<Port> {
            vec![Port {
                name: "out".to_string(),
                signal_type: SignalType::Rgb,
            }]
        }
    }

    #[test]
    fn test_compose_simple() {
        let patch = DevicePatch {
            devices: vec![DeviceInstance {
                device: Box::new(TestRamp),
                instance_id: 0,
                params: HashMap::new(),
                position: (0.0, 0.0),
            }],
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

    #[test]
    fn test_compose_plan_single_segment() {
        let patch = DevicePatch {
            devices: vec![DeviceInstance {
                device: Box::new(TestRamp),
                instance_id: 0,
                params: HashMap::new(),
                position: (0.0, 0.0),
            }],
            connections: vec![],
            output_device: 0,
            next_instance_id: 1,
        };

        let composer = ShaderComposer::new();
        let result = composer.compose_plan(&patch);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.segments.len(), 1);
        assert_eq!(plan.segments[0].kind, SegmentKind::Composed);
        assert!(!plan.segments[0].needs_input_texture);
    }
}
