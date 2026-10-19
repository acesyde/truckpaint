//! Domain types for the TruckPaint livery editor.
//!
//! This crate has no UI, filesystem or threading dependencies so it can be
//! unit-tested in isolation (and compiled to other targets later).

mod brand;
pub mod document;
pub mod import;
pub mod mod_settings;
mod project;
mod symbols;
mod texture_status;
mod usage;

/// The version of the app: the release tag without its `v` in a release
/// build (`0.2.0`, `0.2.0-beta.1`), `dev` in any other build. The release
/// workflow sets `TRUCKPAINT_VERSION` when it builds; the manifest's version
/// is not the app's.
pub const VERSION: &str = match option_env!("TRUCKPAINT_VERSION") {
    Some(version) => version,
    None => "dev",
};

pub use brand::{BrandKit, GraphicStyle, Look, Swatch, TextStyle};
pub use import::LibraryKey;
pub use kurbo;
pub use mod_settings::ModSettings;
pub use project::{
    Asset, AssetKind, Axis, DEFAULT_PROJECT_NAME, DEFAULT_SWATCH_PREFIX, GameData, Guide,
    MAIN_SURFACE_NAME, Project, ProjectVehicle, RequiredMod, Snapshot, Surface, SurfaceTemplate,
    TemplateStatus, TextureKey, TexturePart, TextureResolution, size_label,
};
pub use symbols::Symbol;
pub use texture_status::{
    CheckReason, OffPalette, TextureFilter, TextureState, is_drawn, objects_on, off_palette,
};
pub use usage::{Count, Usage};

#[cfg(test)]
mod tests {
    use super::VERSION;

    #[test]
    fn version_is_dev_unless_the_build_sets_it() {
        let expected = option_env!("TRUCKPAINT_VERSION").unwrap_or("dev");
        assert_eq!(VERSION, expected);
        assert!(!VERSION.is_empty());
    }
}
