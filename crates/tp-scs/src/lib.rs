//! Reads the game's `.scs` archives and vehicle models, in memory and without any external program.

mod city;
mod hashfs;
mod model;
mod mount;
pub mod synthetic;

pub use hashfs::Archive;
pub use model::{Descriptor, Locator, Look, Model, Part, Piece, Stream, Variant, material_effect};
pub use mount::Mount;

/// Names the reader in `report.json`.
pub const READER: &str = "tp-scs";

/// Recorded beside the name in `report.json`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
