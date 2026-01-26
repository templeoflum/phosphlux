//! Device Registry
//!
//! Factory pattern for creating device instances from device_id strings.
//! Provides categorized device lists for the palette UI.

use crate::gpu::ShaderDevice;
use crate::devices::{TestSignalGenerator, PaikAbeColorizer, PaikAbeWobulator};

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
        "paik_abe_colorizer" => Some(Box::new(PaikAbeColorizer::new())),
        "paik_abe_wobulator" => Some(Box::new(PaikAbeWobulator::new())),
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
                DeviceEntry { device_id: "paik_abe_colorizer", display_name: "Colorizer" },
                DeviceEntry { device_id: "paik_abe_wobulator", display_name: "Wobulator" },
            ],
        },
    ]
}
