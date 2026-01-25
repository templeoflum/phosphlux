//! Phosphlux v2 - Analog Video Synthesizer
//!
//! A real-time GPU-accelerated emulator of historical video synthesis hardware.
//!
//! ## Architecture
//!
//! v2 uses a **device-centric** architecture:
//! - Each historical device (Paik/Abe, Sandin, Jones, etc.) is a module
//! - Devices expose authentic circuit behavior through WGSL shaders
//! - The shader composer builds specialized shaders per patch
//! - No runtime dispatch - maximum GPU efficiency
//!
//! ## Modules
//!
//! - `core`: Signal types, component models, timing
//! - `gpu`: Shader composition, rendering
//! - `devices`: Historical device emulations
//! - `ui`: Node editor (TODO)

mod core;
mod devices;
mod gpu;
mod ui;

use std::collections::HashMap;

fn main() {
    println!("Phosphlux v2 - Analog Video Synthesizer");
    println!("========================================");
    println!();

    // Demonstrate the new architecture
    demo_shader_composition();
}

/// Demonstrate shader composition with a Paik/Abe device
fn demo_shader_composition() {
    use crate::devices::PaikAbeColorizer;
    use crate::gpu::{DeviceInstance, DevicePatch, ShaderComposer};

    println!("Testing shader composition...");
    println!();

    // Create a simple patch with one device
    let colorizer = PaikAbeColorizer::new();

    let patch = DevicePatch {
        devices: vec![DeviceInstance {
            device: Box::new(colorizer),
            instance_id: 0,
            params: HashMap::new(),
            position: (0.0, 0.0),
        }],
        connections: vec![],
        output_device: 0,
    };

    // Compose the shader
    let mut composer = ShaderComposer::new();
    match composer.compose(&patch) {
        Ok(shader) => {
            println!("Shader composed successfully!");
            println!("  WGSL source: {} bytes", shader.wgsl_source.len());
            println!("  Parameter layout: {} bytes", shader.param_layout.size);
            println!("  Texture bindings: {}", shader.texture_bindings.len());
            println!();

            // Print a snippet of the generated shader
            println!("Generated shader snippet (last 500 chars):");
            println!("---");
            let snippet = if shader.wgsl_source.len() > 500 {
                &shader.wgsl_source[shader.wgsl_source.len() - 500..]
            } else {
                &shader.wgsl_source
            };
            println!("{}", snippet);
            println!("---");
        }
        Err(e) => {
            println!("Error composing shader: {}", e);
        }
    }

    println!();
    println!("Core systems ready. Next steps:");
    println!("  1. Build the device workbench app");
    println!("  2. Implement more devices (Sandin, Jones, Rutt/Etra)");
    println!("  3. Build the node editor UI");
    println!("  4. Add renderer with proper multi-texture support");
}
