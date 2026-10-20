//! Reads the game's `.scs` archives, mod packages (ZIP or folder) and vehicle models, in memory
//! and without any external program.

mod city;
mod hashfs;
mod layer;
mod model;
mod mount;
mod package;
pub mod synthetic;

pub use hashfs::Archive;
pub use layer::Layer;
pub use model::{Descriptor, Look, Model, Part, Piece, Stream, Variant, material_effect};
pub use mount::Mount;

/// Names the reader in `report.json`.
pub const READER: &str = "tp-scs";

/// Recorded beside the name in `report.json`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
