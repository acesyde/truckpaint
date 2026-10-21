//! Thumbnails of the surfaces' artwork, without their template (the rows of
//! the Textures tab), rendered in a background thread and kept until the
//! surface changes; and thumbnails of the templates, which the Project
//! space draws under the artwork.
//!
//! A thumbnail remembers the top-level objects it was rendered from. Edits
//! go through `Arc::make_mut` from the top of the tree, and the thumbnail
//! holds a reference to every object it shows, so any change to a surface
//! replaces at least one of its top-level `Arc`s: comparing pointers tells
//! whether a thumbnail is stale without walking the objects. Rendering
//! never happens on the UI thread, and only stale surfaces are rendered.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, TryRecvError};

use egui::{ColorImage, TextureHandle, TextureOptions};
use tp_core::document::{AssetId, Object, Rgba};
use tp_core::{Asset, Project};
use tp_render::RenderOptions;
use tp_text::FontLibrary;

/// Longest side of a thumbnail, in pixels: sharp for the Project space's
/// 120-point tiles at 1×, and for a 26-point row up to 4.9×.
pub const SIDE: u32 = 128;

/// Pixel size of a thumbnail of a texture of `size`: its shape, with its
/// longest side at `side` (an 8192 × 1024 texture at 128 gives 128 × 16).
pub fn thumbnail_size(size: tp_core::kurbo::Size, side: u32) -> (u32, u32) {
    let longest = size.width.max(size.height);
    if longest.is_nan() || longest <= 0.0 {
        return (side, side);
    }
    let k = f64::from(side) / longest;
    let px = |s: f64| ((s * k).round() as u32).clamp(1, side);
    (px(size.width), px(size.height))
}

/// Delay before the UI picks up finished thumbnails.
const REPAINT_DELAY: std::time::Duration = std::time::Duration::from_millis(50);

/// What a thumbnail was rendered from.
struct Key {
    objects: Vec<Arc<Object>>,
    size: tp_core::kurbo::Size,
    /// Fonts generation (texts are laid out again when fonts load).
    fonts: u64,
}

impl Key {
    fn of(project: &Project, surface: usize, fonts: u64) -> Self {
        let s = &project.surfaces[surface];
        Self {
            objects: s.objects.clone(),
            size: s.size,
            fonts,
        }
    }

    fn matches(&self, project: &Project, surface: usize, fonts: u64) -> bool {
        let s = &project.surfaces[surface];
        self.size == s.size
            && self.fonts == fonts
            && self.objects.len() == s.objects.len()
            && self
                .objects
                .iter()
                .zip(&s.objects)
                .all(|(a, b)| Arc::ptr_eq(a, b))
    }
}

struct Thumbnail {
    key: Key,
    texture: TextureHandle,
    image: ColorImage,
}

type Rendered = Vec<(usize, Key, ColorImage)>;

/// Templates read by the worker; `None` for one that can't be read.
type Templates = Vec<(AssetId, Option<ColorImage>)>;

/// Thumbnails of a project's surfaces, by surface index.
pub struct SurfaceThumbnails {
    /// Prefix of the textures' debug names.
    name: &'static str,
    /// Longest side of the thumbnails, in pixels.
    side: u32,
    background: Option<Rgba>,
    shown: HashMap<usize, Thumbnail>,
    /// The running render: surfaces, and templates when asked for.
    job: Option<Receiver<(Rendered, Templates)>>,
    /// The renderer stopped (it could not start, or panicked): no retry.
    broken: bool,
    /// Number of surfaces rendered, for tests and diagnostics.
    pub renders: u64,
    /// Template thumbnails by asset (`None`: unreadable). Templates don't
    /// change once imported (an update brings a new asset): each is read
    /// once.
    templates: HashMap<AssetId, Option<TextureHandle>>,
}

impl Default for SurfaceThumbnails {
    fn default() -> Self {
        Self::named("surface_thumbnail")
    }
}

impl SurfaceThumbnails {
    /// Thumbnails whose textures are named "`name`_N" (for debugging).
    pub fn named(name: &'static str) -> Self {
        Self::sized(name, SIDE, None)
    }

    /// Same, with the longest side of the thumbnails at `side` pixels, over `background`.
    pub fn sized(name: &'static str, side: u32, background: Option<Rgba>) -> Self {
        Self {
            name,
            side,
            background,
            shown: HashMap::new(),
            job: None,
            broken: false,
            renders: 0,
            templates: HashMap::new(),
        }
    }

    /// Picks up finished renders, then starts rendering the surfaces whose
    /// thumbnail is missing or stale. Nothing starts while a pointer button
    /// is down (a drag edits the artwork every frame): thumbnails catch up
    /// when it ends.
    pub fn update(&mut self, ctx: &egui::Context, project: &Project, fonts: &FontLibrary) {
        self.update_with(ctx, project, fonts, false, None);
    }

    /// Same, rendering only the surfaces `only` (the others are never
    /// rendered).
    pub fn update_only(
        &mut self,
        ctx: &egui::Context,
        project: &Project,
        fonts: &FontLibrary,
        only: &[usize],
    ) {
        self.update_with(ctx, project, fonts, false, Some(only));
    }

    /// Same, also reading the templates that have no thumbnail yet (the
    /// Project space draws the artwork over them), in the same render.
    pub fn update_with_templates(
        &mut self,
        ctx: &egui::Context,
        project: &Project,
        fonts: &FontLibrary,
    ) {
        self.update_with(ctx, project, fonts, true, None);
    }

    /// Picks up a finished render without starting any; false while one is
    /// still running. Called every frame, whatever is shown, so that a
    /// render started on one screen is picked up on another.
    pub fn poll(&mut self, ctx: &egui::Context) -> bool {
        if let Some(job) = &self.job {
            match job.try_recv() {
                Ok((done, read)) => {
                    for (surface, key, image) in done {
                        let texture = ctx.load_texture(
                            format!("{}_{surface}", self.name),
                            image.clone(),
                            TextureOptions::LINEAR,
                        );
                        self.shown.insert(
                            surface,
                            Thumbnail {
                                key,
                                texture,
                                image,
                            },
                        );
                    }
                    for (asset, image) in read {
                        let texture = image.map(|image| {
                            ctx.load_texture(
                                format!("template_thumbnail_{}", asset.0),
                                image,
                                TextureOptions::LINEAR,
                            )
                        });
                        self.templates.insert(asset, texture);
                    }
                    self.job = None;
                }
                Err(TryRecvError::Empty) => return false,
                Err(TryRecvError::Disconnected) => {
                    self.job = None;
                    self.broken = true;
                }
            }
        }
        true
    }

    fn update_with(
        &mut self,
        ctx: &egui::Context,
        project: &Project,
        fonts: &FontLibrary,
        templates: bool,
        only: Option<&[usize]>,
    ) {
        if !self.poll(ctx) {
            return;
        }
        let count = project.surfaces.len();
        self.shown.retain(|i, _| *i < count);
        let used: Vec<AssetId> = project.template_assets().collect();
        self.templates.retain(|asset, _| used.contains(asset));
        if self.broken || ctx.input(|i| i.pointer.any_down()) {
            return;
        }
        let generation = fonts.generation();
        let stale: Vec<(usize, Key)> = (0..count)
            .filter(|i| only.is_none_or(|only| only.contains(i)))
            .filter(|i| {
                !self
                    .shown
                    .get(i)
                    .is_some_and(|t| t.key.matches(project, *i, generation))
            })
            .map(|i| (i, Key::of(project, i, generation)))
            .collect();
        let missing: Vec<Arc<Asset>> = if templates {
            used.iter()
                .filter(|a| !self.templates.contains_key(a))
                .filter_map(|a| project.assets.get(a).cloned())
                .collect()
        } else {
            Vec::new()
        };
        if stale.is_empty() && missing.is_empty() {
            return;
        }
        self.renders += stale.len() as u64;
        // Marked as read already: a template that fails is not read again.
        for asset in &missing {
            self.templates.insert(asset.id, None);
        }
        self.job = start(
            ctx,
            project.clone(),
            stale,
            missing,
            fonts.fork(),
            self.side,
            self.background,
        );
        self.broken = self.job.is_none();
    }

    /// The thumbnail of surface `surface`, once rendered (the previous one
    /// stays shown while it is rendered again).
    pub fn texture(&self, surface: usize) -> Option<&TextureHandle> {
        self.shown.get(&surface).map(|t| &t.texture)
    }

    /// Whether thumbnails are being rendered.
    pub fn is_rendering(&self) -> bool {
        self.job.is_some()
    }

    /// The thumbnail of template `asset`, once read.
    pub fn template(&self, asset: AssetId) -> Option<&TextureHandle> {
        self.templates.get(&asset)?.as_ref()
    }

    /// Pixel size of the thumbnail of `surface`, once rendered (tests).
    pub fn size_of(&self, surface: usize) -> Option<[usize; 2]> {
        self.shown.get(&surface).map(|t| t.image.size)
    }

    /// Color of the thumbnail of `surface` at `uv` (0..1), for tests.
    pub fn color_at(&self, surface: usize, uv: [f32; 2]) -> Option<Rgba> {
        let image = &self.shown.get(&surface)?.image;
        let [w, h] = image.size;
        let x = ((uv[0] * w as f32) as usize).min(w.saturating_sub(1));
        let y = ((uv[1] * h as f32) as usize).min(h.saturating_sub(1));
        let [r, g, b, a] = image.pixels.get(y * w + x)?.to_srgba_unmultiplied();
        Some(Rgba::with_alpha(r, g, b, a))
    }
}

/// Renders `surfaces` of `project` and reads the `templates` on a worker
/// thread; `None` when the thread cannot start.
fn start(
    ctx: &egui::Context,
    project: Project,
    surfaces: Vec<(usize, Key)>,
    templates: Vec<Arc<Asset>>,
    mut fonts: FontLibrary,
    side: u32,
    background: Option<Rgba>,
) -> Option<Receiver<(Rendered, Templates)>> {
    let (tx, rx) = mpsc::channel();
    let ctx = ctx.clone();
    let spawned = std::thread::Builder::new()
        .name("surface-thumbnails".into())
        .spawn(move || {
            let done: Rendered = surfaces
                .into_iter()
                .map(|(surface, key)| {
                    let (width, height) = thumbnail_size(key.size, side);
                    let options = RenderOptions {
                        width,
                        height,
                        background,
                    };
                    let pixmap =
                        tp_render::render(&project, surface, options, &mut fonts, &mut |_, _| true)
                            .expect("never cancelled");
                    let image = ColorImage::from_rgba_premultiplied(
                        [width as usize, height as usize],
                        pixmap.data(),
                    );
                    (surface, key, image)
                })
                .collect();
            let read: Templates = templates
                .iter()
                .map(|a| (a.id, crate::image_cache::asset_image(a, side)))
                .collect();
            let _ = tx.send((done, read));
            // Shown at the next frame, without waking the UI at once: a
            // thumbnail is not urgent, and an immediate repaint would land
            // in the middle of whatever the UI is busy with.
            ctx.request_repaint_after(REPAINT_DELAY);
        });
    match spawned {
        Ok(_) => Some(rx),
        Err(err) => {
            tracing::error!(%err, "cannot render the texture thumbnails");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use tp_core::document::{Frame, ObjectId, Paint, ShapeKind};
    use tp_core::kurbo::{Point, Size};

    use super::*;

    fn project() -> Project {
        let mut p = Project::new("Test", tp_core::TextureResolution::default());
        let side = p.surface().max_side();
        let mut o = Object::new(
            ObjectId(0),
            ShapeKind::rectangle(),
            Frame::new(
                Point::new(side / 2.0, side / 2.0),
                Size::new(side, side),
                0.0,
            ),
        );
        o.fill = Paint::Solid(Rgba::rgb(255, 0, 0));
        p.add(o);
        p
    }

    /// Updates until the render finishes.
    fn settle(cache: &mut SurfaceThumbnails, ctx: &egui::Context, p: &Project, f: &FontLibrary) {
        for _ in 0..500 {
            cache.update(ctx, p, f);
            if !cache.is_rendering() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        panic!("thumbnails never finished");
    }

    #[test]
    fn rendered_once_and_again_after_an_edit() {
        let ctx = egui::Context::default();
        let fonts = FontLibrary::bundled();
        let mut p = project();
        let mut cache = SurfaceThumbnails::default();
        settle(&mut cache, &ctx, &p, &fonts);
        assert_eq!(cache.renders, 1);
        assert_eq!(cache.color_at(0, [0.5, 0.5]), Some(Rgba::rgb(255, 0, 0)));
        // Unchanged: nothing rendered again.
        settle(&mut cache, &ctx, &p, &fonts);
        assert_eq!(cache.renders, 1);
        // An edit in place (`make_mut`) still gives a new pointer.
        Arc::make_mut(&mut p.surface_mut().objects[0]).fill = Paint::Solid(Rgba::rgb(0, 0, 255));
        settle(&mut cache, &ctx, &p, &fonts);
        assert_eq!(cache.renders, 2);
        assert_eq!(cache.color_at(0, [0.5, 0.5]), Some(Rgba::rgb(0, 0, 255)));
    }

    #[test]
    fn only_the_given_surfaces_are_rendered() {
        let ctx = egui::Context::default();
        let fonts = FontLibrary::bundled();
        let mut p = project();
        for name in ["Chassis", "Trailer"] {
            let mut s = p.surfaces[0].clone();
            s.name = name.into();
            p.surfaces.push(s);
        }
        let mut cache = SurfaceThumbnails::named("preview");
        let settle = |cache: &mut SurfaceThumbnails, p: &Project| {
            for _ in 0..500 {
                cache.update_only(&ctx, p, &fonts, &[0, 2]);
                if !cache.is_rendering() {
                    return;
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            panic!("thumbnails never finished");
        };
        settle(&mut cache, &p);
        assert_eq!(cache.renders, 2);
        assert!(cache.texture(1).is_none());
        // Only the surface whose objects changed is rendered again.
        Arc::make_mut(&mut p.surfaces[2].objects[0]).fill = Paint::Solid(Rgba::rgb(0, 0, 255));
        Arc::make_mut(&mut p.surfaces[1].objects[0]).fill = Paint::Solid(Rgba::rgb(0, 0, 255));
        settle(&mut cache, &p);
        assert_eq!(cache.renders, 3);
        assert_eq!(cache.color_at(2, [0.5, 0.5]), Some(Rgba::rgb(0, 0, 255)));
        assert_eq!(cache.color_at(0, [0.5, 0.5]), Some(Rgba::rgb(255, 0, 0)));
    }

    #[test]
    fn a_thumbnail_keeps_the_texture_shape() {
        assert_eq!(thumbnail_size(Size::new(8192.0, 1024.0), SIDE), (128, 16));
        assert_eq!(thumbnail_size(Size::new(4096.0, 4096.0), SIDE), (128, 128));
        assert_eq!(thumbnail_size(Size::new(64.0, 8192.0), SIDE), (1, 128));
        let ctx = egui::Context::default();
        let fonts = FontLibrary::bundled();
        let mut p = project();
        p.surfaces[0].size = Size::new(8192.0, 1024.0);
        let mut cache = SurfaceThumbnails::default();
        settle(&mut cache, &ctx, &p, &fonts);
        assert_eq!(cache.size_of(0), Some([128, 16]));
        // A larger instance keeps the shape too.
        let mut large = SurfaceThumbnails::sized("large", 1024, None);
        settle(&mut large, &ctx, &p, &fonts);
        assert_eq!(large.size_of(0), Some([1024, 128]));
    }
}
