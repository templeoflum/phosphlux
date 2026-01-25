//! User Interface Module
//!
//! Provides egui-based control panels for:
//! - Parameter adjustment
//! - Effect stacks
//! - Signal routing/patching
//! - Presets

pub mod controls;
pub mod patch;

pub use controls::ControlPanel;
pub use patch::{SignalNode, SignalPatch, NodeType};
