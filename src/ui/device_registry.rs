//! Device Registry
//!
//! Factory pattern for creating device instances from device_id strings.
//! Provides categorized device lists for the palette UI.
//!
//! IMPORTANT: Devices must be validated in the workbench before being marked
//! as production-ready. Unvalidated devices are shown with a warning indicator.
//! See: research/validation/device_validation.json

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

/// Validation status for a device
#[derive(Clone, Copy, PartialEq)]
pub enum ValidationStatus {
    /// Device has been validated in workbench with documentation
    Validated,
    /// Device needs workbench validation before production use
    NeedsValidation,
    /// Legacy device kept for backwards compatibility
    Legacy,
    /// Artistic interpretation - not authentic to original schematics
    Prototype,
}

/// A category of devices in the palette
pub struct DeviceCategory {
    pub name: &'static str,
    pub devices: Vec<DeviceEntry>,
}

/// An entry in the device palette
pub struct DeviceEntry {
    pub device_id: &'static str,
    pub display_name: &'static str,
    pub validation: ValidationStatus,
}

impl DeviceEntry {
    /// Get display name with validation indicator
    pub fn display_with_status(&self) -> String {
        match self.validation {
            ValidationStatus::Validated => self.display_name.to_string(),
            ValidationStatus::NeedsValidation => format!("[!] {}", self.display_name),
            ValidationStatus::Legacy => format!("{} (Legacy)", self.display_name),
            ValidationStatus::Prototype => format!("{} [Proto]", self.display_name),
        }
    }
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
                DeviceEntry {
                    device_id: "test_signal",
                    display_name: "Test Signal",
                    validation: ValidationStatus::Validated, // Utility device, not a historical emulation
                },
            ],
        },
        DeviceCategory {
            name: "Paik/Abe",
            devices: vec![
                // Modular device chain - VALIDATED 2026-01-29
                DeviceEntry {
                    device_id: "paik_abe_input_matrix",
                    display_name: "Input Matrix",
                    validation: ValidationStatus::Validated,
                },
                // Prototype: Wavefolder (NOT in actual Paik/Abe schematics)
                DeviceEntry {
                    device_id: "paik_abe_folding_amp",
                    display_name: "Wavefolder",
                    validation: ValidationStatus::Prototype,
                },
                DeviceEntry {
                    device_id: "paik_abe_ped_colorizer",
                    display_name: "PED Colorizer",
                    validation: ValidationStatus::Validated,
                },
                DeviceEntry {
                    device_id: "paik_abe_output_encoder",
                    display_name: "Output Encoder",
                    validation: ValidationStatus::Validated,
                },
                // Validated devices
                DeviceEntry {
                    device_id: "paik_abe_wobulator",
                    display_name: "Wobulator",
                    validation: ValidationStatus::Validated,
                },
                // Prototype: Artistic wavefolder (NOT authentic - see research/prototypes/)
                DeviceEntry {
                    device_id: "paik_abe_colorizer",
                    display_name: "Wavefold Colorizer",
                    validation: ValidationStatus::Prototype,
                },
            ],
        },
    ]
}
