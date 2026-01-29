//! Device Registry
//!
//! Factory pattern for creating device instances from device_id strings.
//! Provides categorized device lists for the palette UI.

use crate::gpu::ShaderDevice;
use crate::devices::{
    PaikAbeColorizer,
    PaikAbeFoldingAmp,
    PaikAbeInputMatrix,
    PaikAbeOutputEncoder,
    PaikAbePedColorizer,
    PaikAbeWobulator,
    TestSignalGenerator,
};

/// A category of devices in the palette
pub struct DeviceCategory {
    pub name: &'static str,
    pub devices: Vec<DeviceEntry>,
}

/// An entry in the device palette
pub struct DeviceEntry {
    pub device_id: &'static str,
    pub display_name: &'static str,
}

/// Create a device instance from a device_id string
pub fn create_device(device_id: &str) -> Option<Box<dyn ShaderDevice>> {
    match device_id {
        "test_signal" => Some(Box::new(TestSignalGenerator::new())),
        // Paik/Abe device chain (in signal flow order)
        "paik_abe_input_matrix" => Some(Box::new(PaikAbeInputMatrix::new())),
        "paik_abe_folding_amp" => Some(Box::new(PaikAbeFoldingAmp::new())),
        "paik_abe_ped_colorizer" => Some(Box::new(PaikAbePedColorizer::new())),
        "paik_abe_output_encoder" => Some(Box::new(PaikAbeOutputEncoder::new())),
        "paik_abe_wobulator" => Some(Box::new(PaikAbeWobulator::new())),
        // Legacy combined colorizer (backwards compatibility)
        "paik_abe_colorizer" => Some(Box::new(PaikAbeColorizer::new())),
        _ => None,
    }
}

/// Get all device categories for the palette
pub fn device_categories() -> Vec<DeviceCategory> {
    vec![
        DeviceCategory {
            name: "Generators",
            devices: vec![
                DeviceEntry { device_id: "test_signal", display_name: "Test Signal" },
            ],
        },
        DeviceCategory {
            name: "Paik/Abe",
            devices: vec![
                // New modular device chain
                DeviceEntry { device_id: "paik_abe_input_matrix", display_name: "Input Matrix" },
                DeviceEntry { device_id: "paik_abe_folding_amp", display_name: "Folding Amp" },
                DeviceEntry { device_id: "paik_abe_ped_colorizer", display_name: "PED Colorizer" },
                DeviceEntry { device_id: "paik_abe_output_encoder", display_name: "Output Encoder" },
                DeviceEntry { device_id: "paik_abe_wobulator", display_name: "Wobulator" },
                // Legacy combined device
                DeviceEntry { device_id: "paik_abe_colorizer", display_name: "Colorizer (Legacy)" },
            ],
        },
    ]
}
