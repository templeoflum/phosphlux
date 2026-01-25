//! User Interface Module
//!
//! Provides egui-based control panels for:
//! - Parameter adjustment
//! - Effect stacks
//! - Signal routing/patching
//! - Node graph editor
//! - Presets

pub mod controls;
pub mod patch;
pub mod node_editor;

pub use controls::ControlPanel;
