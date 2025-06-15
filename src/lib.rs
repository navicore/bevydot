#![allow(clippy::cast_precision_loss)] // We accept precision loss for f32 conversions
#![allow(clippy::needless_pass_by_value)] // Bevy systems require owned Res parameters
#![allow(clippy::multiple_crate_versions)] // Bevy dependencies have multiple versions

pub mod camera;
pub mod search;
pub mod types;
pub mod ui;
pub mod visualization;

pub use types::{GraphData, NodeType};
