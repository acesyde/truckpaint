use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use kurbo::{Point, Rect, Size, Vec2};
use serde::{Deserialize, Serialize};

use crate::brand::{BrandKit, GraphicStyle, Swatch, TextStyle};
use crate::document::SymbolId;
use crate::document::tree::{self, Hit, Placement};
use crate::document::{AssetId, Object, ObjectId, PointRef, Rgba, ShapeKind};
use crate::import::LibraryKey;
use crate::mod_settings::{INTERNAL_NAME_MAX, ModSettings, SPLIT_INTERNAL_NAME_MAX};
use crate::symbols::Symbol;

/// Name given to a project created without a name.
pub const DEFAULT_PROJECT_NAME: &str = "Untitled";
/// Name of the surface every new project starts with.
pub const MAIN_SURFACE_NAME: &str = "Main texture";
/// Name of new swatches when none is given ("Color 1", "Color 2"…).
pub const DEFAULT_SWATCH_PREFIX: &str = "Color";

/// Square texture resolution a livery is authored for.
///
/// The document itself stays vector-based; the resolution is the nominal
/// texture size used for coordinates and default export.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextureResolution {
    R2048,
    #[default]
    R4096,
    R8192,
}

impl TextureResolution {
    pub const ALL: [Self; 3] = [Self::R2048, Self::R4096, Self::R8192];

    /// Side length in pixels.
    pub fn side(self) -> u32 {
        match self {
            Self::R2048 => 2048,
            Self::R4096 => 4096,
            Self::R8192 => 8192,
        }
    }

    /// Human-readable label, e.g. `4096 × 4096`.
    pub fn label(self) -> String {
        let side = self.side();
        format!("{side} × {side}")
    }
}

/// Orientation of a guide.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    /// A horizontal line at a given y.
    Horizontal,
    /// A vertical line at a given x.
    Vertical,
}

/// A guide line across a surface, in texture pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Guide {
    pub axis: Axis,
    /// y for horizontal guides, x for vertical guides.
    pub position: f64,
}

impl Guide {
    pub fn new(axis: Axis, position: f64) -> Self {
        Self { axis, position }
    }
}

/// One texture of the vehicle: an artboard of the texture's width and
/// height holding ordered objects (later objects are drawn on top) and its
/// guides.
#[derive(Clone, Debug, PartialEq)]
pub struct Surface {
    pub name: String,
    /// Width and height in texture pixels (equal for a square texture).
    pub size: Size,
    pub objects: Vec<Arc<Object>>,
    pub guides: Vec<Guide>,
    /// The vehicle's texture layout shown over the artwork (never exported).
    pub template: Option<SurfaceTemplate>,
}

/// Where a template stands after an update to a newer package version.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TemplateStatus {
    #[default]
    Current,
    /// The texture's layout version changed: the artwork may need checking.
    LayoutChanged,
    /// The texture no longer exists in the package version; no template.
    Removed,
}

/// Whether a vehicle texture is a main texture (a cabin layout, or the
/// whole vehicle) or an accessory.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TexturePart {
    #[default]
    Main,
    Accessory,
}

/// Which texture of which vehicle a surface paints.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextureKey {
    pub package_id: String,
    pub texture_id: String,
}

/// The template of a surface: a reference image of the texture layout. It
/// also records the vehicle texture the surface belongs to.
#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceTemplate {
    /// The vehicle package the texture comes from.
    pub package_id: String,
    /// The package part (texture) it comes from.
    pub texture_id: String,
    /// Whether that texture is a main texture or an accessory.
    pub part: TexturePart,
    /// The image, stored with the project's assets.
    pub asset: AssetId,
    pub layout_version: u32,
    /// What the game calls the things the texture covers: the cabins of a
    /// main texture (none: every cabin) or the accessory ids of an
    /// accessory. Empty when the game data is unknown.
    pub game_ids: Vec<String>,
    /// A main texture's accessory ids the game paints with the same texture
    /// (a trailer body). Empty for an accessory.
    pub also_paints: Vec<String>,
    /// A main texture's position among the package's main textures (an
    /// accessory has none).
    pub main_index: Option<usize>,
    /// 0.0..=1.0; not part of the undo history.
    pub opacity: f32,
    /// Not part of the undo history.
    pub visible: bool,
    pub status: TemplateStatus,
    /// Objects an update asked to reposition; shown only while `status` is
    /// `LayoutChanged` (see [`crate::objects_on`]).
    pub flagged: BTreeSet<ObjectId>,
}

impl SurfaceTemplate {
    /// Default opacity of a new template.
    pub const DEFAULT_OPACITY: f32 = 0.6;

    /// The vehicle texture this template belongs to.
    pub fn key(&self) -> TextureKey {
        TextureKey {
            package_id: self.package_id.clone(),
            texture_id: self.texture_id.clone(),
        }
    }

    /// Whether it is texture `texture` of `package`.
    pub fn is_of(&self, package: &str, texture: &str) -> bool {
        self.package_id == package && self.texture_id == texture
    }

    /// Whether it should be drawn (shown and still in the package).
    pub fn is_drawn(&self) -> bool {
        self.visible && self.status != TemplateStatus::Removed
    }

    /// The same template ignoring opacity and visibility (undo compares
    /// templates this way).
    #[allow(clippy::type_complexity)]
    fn document_part(
        &self,
    ) -> (
        &str,
        &str,
        TexturePart,
        AssetId,
        u32,
        &[String],
        &[String],
        Option<usize>,
        TemplateStatus,
        &BTreeSet<ObjectId>,
    ) {
        (
            &self.package_id,
            &self.texture_id,
            self.part,
            self.asset,
            self.layout_version,
            &self.game_ids,
            &self.also_paints,
            self.main_index,
            self.status,
            &self.flagged,
        )
    }
}

/// A vehicle of a project: the package and version its templates come
/// from. What it paints is its surfaces (see [`Project::vehicle_range`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectVehicle {
    pub package_id: String,
    /// Semantic version of the package.
    pub version: String,
    pub name: String,
    pub brand: String,
    /// `truck` or `trailer`.
    pub kind: String,
    /// `ets2` or `ats`.
    pub game: String,
    /// What the mod export needs, recorded from the package. `None` in
    /// files written before it was recorded, until it is filled in from the
    /// installed package.
    pub game_data: Option<GameData>,
}

/// What a vehicle is in the game, recorded from its package version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameData {
    /// The vehicle's path in the game's definitions (`scania.r_2016`).
    pub path: String,
    /// Game versions the templates match, as a version range.
    pub versions: String,
    /// The paint job uses the vehicle's alternate UV set.
    pub alt_uv: bool,
    /// The paint job lets the player pick a base color.
    pub colour_picker: bool,
    /// Mods the vehicle depends on.
    pub requires: Vec<RequiredMod>,
    /// Number of main textures in the package (painted or not).
    pub main_count: usize,
}

/// A mod a vehicle depends on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequiredMod {
    pub name: String,
    /// Version range, if any.
    pub version: Option<String>,
}

impl Surface {
    pub fn new(name: impl Into<String>, size: Size) -> Self {
        Self {
            name: name.into(),
            size,
            objects: Vec::new(),
            guides: Vec::new(),
            template: None,
        }
    }

    /// The artboard rectangle in texture pixels.
    pub fn bounds(&self) -> Rect {
        self.size.to_rect()
    }

    /// The longer of the two sides, in texture pixels.
    pub fn max_side(&self) -> f64 {
        self.size.width.max(self.size.height)
    }

    /// The size as shown to the player: "4096²" or "8192 × 1024".
    pub fn size_label(&self) -> String {
        size_label(self.size)
    }

    pub fn index_of(&self, id: ObjectId) -> Option<usize> {
        self.objects.iter().position(|o| o.id == id)
    }

    /// The object with `id`, anywhere in the tree.
    pub fn get(&self, id: ObjectId) -> Option<&Arc<Object>> {
        tree::get(&self.objects, id)
    }

    /// Topmost visible, unlocked object under `point` (tolerance in texture
    /// pixels): the top-level object a click selects and the innermost one.
    pub fn hit_test(&self, point: Point, tolerance: f64) -> Option<Hit> {
        tree::hit_test(&self.objects, point, tolerance)
    }

    /// Visible, unlocked top-level objects touching `rect`, bottom to top.
    pub fn objects_in_rect(&self, rect: Rect) -> Vec<ObjectId> {
        tree::top_level_in_rect(&self.objects, rect)
    }

    /// Removes the given objects (with their subtrees); returns how many.
    pub fn remove(&mut self, ids: &[ObjectId]) -> usize {
        tree::remove(&mut self.objects, ids).len()
    }

    /// Replaces objects with the same ids, anywhere in the tree.
    pub fn replace(&mut self, objects: &[Object]) {
        tree::replace(&mut self.objects, objects);
    }

    /// Moves each selected object one step up within its parent.
    pub fn bring_forward(&mut self, ids: &[ObjectId]) {
        tree::bring_forward(&mut self.objects, ids);
    }

    /// Moves each selected object one step down within its parent.
    pub fn send_backward(&mut self, ids: &[ObjectId]) {
        tree::send_backward(&mut self.objects, ids);
    }
}

/// What kind of file an asset holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetKind {
    /// PNG or JPEG pixels.
    Raster,
    /// SVG source, rendered at any resolution.
    Svg,
}

/// An imported file shared by image objects.
#[derive(Clone, Debug, PartialEq)]
pub struct Asset {
    pub id: AssetId,
    pub name: String,
    pub kind: AssetKind,
    /// Original file content.
    pub bytes: Arc<[u8]>,
    /// Pixel size (raster) or declared size (SVG).
    pub size: Size,
    /// BLAKE3 hash of `bytes`, for deduplication.
    pub hash: [u8; 32],
}

impl Asset {
    /// An asset with its content hash computed.
    pub fn new(id: AssetId, name: &str, kind: AssetKind, bytes: Arc<[u8]>, size: Size) -> Self {
        Self {
            id,
            name: name.to_owned(),
            kind,
            hash: *blake3::hash(&bytes).as_bytes(),
            bytes,
            size,
        }
    }
}

/// A livery project: one or more surfaces of vector objects.
#[derive(Clone, Debug, PartialEq)]
pub struct Project {
    pub name: String,
    pub resolution: TextureResolution,
    pub surfaces: Vec<Surface>,
    pub active_surface: usize,
    /// Named colors, linked to by fills, strokes and gradient stops.
    pub palette: Vec<Swatch>,
    /// Named fills, strokes and opacities objects can follow.
    pub graphic_styles: Vec<GraphicStyle>,
    /// Named character styles texts can follow.
    pub text_styles: Vec<TextStyle>,
    /// Imported files, shared by image objects and templates.
    pub assets: BTreeMap<AssetId, Arc<Asset>>,
    /// The vehicles of the fleet, all of one game. Projects made by the
    /// application always have at least one; [`Project::new`] makes one
    /// without (unit tests).
    pub vehicles: Vec<ProjectVehicle>,
    /// Drawings placed as instances on the textures.
    pub symbols: Vec<Symbol>,
    /// The symbol shown and edited instead of the active texture.
    pub editing_symbol: Option<SymbolId>,
    /// What the project exports as a mod.
    pub mod_settings: ModSettings,
    /// The game versions the mod is made for, as the game writes them
    /// (`1.56.*`): the manifest's `compatible_versions`.
    pub game_versions: Vec<String>,
    /// The accessory chosen in each slot of the Side View, by vehicle (package id) and slot id; `None`
    /// is "None". Only what differs from the game's default configuration is kept. A plain setting:
    /// not in the undo history.
    pub side_view: BTreeMap<String, BTreeMap<String, Option<String>>>,
    next_id: u64,
}

impl Project {
    /// Creates a project with one empty surface, trimming the name and
    /// falling back to [`DEFAULT_PROJECT_NAME`] when it is empty.
    pub fn new(name: &str, resolution: TextureResolution) -> Self {
        let trimmed = name.trim();
        let name = if trimmed.is_empty() {
            DEFAULT_PROJECT_NAME.to_owned()
        } else {
            trimmed.to_owned()
        };
        let mod_settings = ModSettings::for_project(&name);
        Self {
            name,
            resolution,
            surfaces: vec![Surface::new(MAIN_SURFACE_NAME, {
                let side = f64::from(resolution.side());
                Size::new(side, side)
            })],
            active_surface: 0,
            palette: Vec::new(),
            graphic_styles: Vec::new(),
            text_styles: Vec::new(),
            assets: BTreeMap::new(),
            vehicles: Vec::new(),
            symbols: Vec::new(),
            editing_symbol: None,
            mod_settings,
            game_versions: Vec::new(),
            side_view: BTreeMap::new(),
            next_id: 1,
        }
    }

    /// Rebuilds a project from stored parts, keeping object and asset ids;
    /// new ids continue after the largest one found. Needs at least one
    /// surface.
    pub fn from_parts(
        name: &str,
        resolution: TextureResolution,
        surfaces: Vec<Surface>,
        active_surface: usize,
        brand: BrandKit,
        assets: BTreeMap<AssetId, Arc<Asset>>,
    ) -> Self {
        fn max_id(list: &[Arc<Object>]) -> u64 {
            list.iter()
                .map(|o| o.id.0.max(max_id(&o.children)))
                .max()
                .unwrap_or(0)
        }
        let largest = surfaces
            .iter()
            .map(|s| max_id(&s.objects))
            .chain(assets.keys().map(|a| a.0))
            .chain(brand.palette.iter().map(|s| s.id.0))
            .chain(brand.graphic_styles.iter().map(|s| s.id.0))
            .chain(brand.text_styles.iter().map(|s| s.id.0))
            .max()
            .unwrap_or(0);
        let mut project = Self::new(name, resolution);
        assert!(!surfaces.is_empty(), "a project has at least one surface");
        project.active_surface = active_surface.min(surfaces.len() - 1);
        project.surfaces = surfaces;
        project.palette = brand.palette;
        project.graphic_styles = brand.graphic_styles;
        project.text_styles = brand.text_styles;
        project.assets = assets;
        project.next_id = largest + 1;
        project
    }

    /// Texture size in pixels (width, height).
    pub fn texture_size(&self) -> (u32, u32) {
        let side = self.resolution.side();
        (side, side)
    }

    /// The surface being edited: the edited symbol's, else the active
    /// texture.
    pub fn surface(&self) -> &Surface {
        if let Some(s) = self.edited_symbol() {
            return &s.surface;
        }
        &self.surfaces[self.active_surface]
    }

    /// The symbol being edited, if any.
    pub fn edited_symbol(&self) -> Option<&Symbol> {
        let id = self.editing_symbol?;
        self.symbols.iter().find(|s| s.id == id)
    }

    /// The game of the fleet (`ets2` or `ats`): its first vehicle's.
    pub fn game(&self) -> Option<&str> {
        self.vehicles.first().map(|v| v.game.as_str())
    }

    /// The project's vehicle of `package_id`.
    pub fn vehicle(&self, package_id: &str) -> Option<&ProjectVehicle> {
        self.vehicles.iter().find(|v| v.package_id == package_id)
    }

    /// The vehicle surface `index` belongs to.
    pub fn vehicle_of(&self, index: usize) -> Option<&ProjectVehicle> {
        let t = self.surfaces.get(index)?.template.as_ref()?;
        self.vehicle(&t.package_id)
    }

    /// The surfaces of the vehicle `package`: a contiguous range (empty at
    /// the end of the project when it has none).
    pub fn vehicle_range(&self, package: &str) -> std::ops::Range<usize> {
        let of = |s: &Surface| s.template.as_ref().is_some_and(|t| t.package_id == package);
        match self.surfaces.iter().position(of) {
            Some(start) => {
                let len = self.surfaces[start..].iter().take_while(|s| of(s)).count();
                start..start + len
            }
            None => self.surfaces.len()..self.surfaces.len(),
        }
    }

    /// Vehicle and texture names of surface `index`, falling back to the
    /// package id when the vehicle is unknown.
    pub fn surface_names(&self, index: usize) -> Option<(String, String)> {
        let surface = self.surfaces.get(index)?;
        let t = surface.template.as_ref()?;
        let vehicle_name = self
            .vehicle(&t.package_id)
            .map_or_else(|| t.package_id.clone(), |v| v.name.clone());
        Some((vehicle_name, surface.name.clone()))
    }

    /// Whether any surface in `range` holds artwork.
    pub fn has_artwork(&self, range: std::ops::Range<usize>) -> bool {
        self.surfaces
            .get(range)
            .is_some_and(|s| s.iter().any(|s| !s.objects.is_empty()))
    }

    pub fn surface_mut(&mut self) -> &mut Surface {
        if let Some(id) = self.editing_symbol
            && let Some(i) = self.symbols.iter().position(|s| s.id == id)
        {
            return &mut self.symbols[i].surface;
        }
        &mut self.surfaces[self.active_surface]
    }

    /// A new, never-used object id.
    pub fn next_object_id(&mut self) -> ObjectId {
        ObjectId(self.fresh_id())
    }

    /// The next id the counter will give.
    pub(crate) fn id_counter(&self) -> u64 {
        self.next_id
    }

    /// Makes sure new ids come after `id`.
    pub(crate) fn reserve_ids_up_to(&mut self, id: u64) {
        self.next_id = self.next_id.max(id + 1);
    }

    /// A new, never-used id (objects, assets, swatches, styles and symbols
    /// share the counter).
    pub(crate) fn fresh_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Gives `object` and all its descendants fresh ids.
    pub(crate) fn assign_fresh_ids(&mut self, object: &mut Object) {
        object.id = self.next_object_id();
        for child in &mut object.children {
            let mut c = (**child).clone();
            self.assign_fresh_ids(&mut c);
            *child = Arc::new(c);
        }
    }

    /// Replaces the object `id` of the active surface, at the same place in
    /// its parent, by a group that keeps `id`; the children get fresh ids.
    /// Returns false (and changes nothing) when `id` does not exist.
    pub fn replace_with_group(&mut self, id: ObjectId, mut group: Object) -> bool {
        if self.surface().get(id).is_none() {
            return false;
        }
        for child in &mut group.children {
            let mut c = (**child).clone();
            self.assign_fresh_ids(&mut c);
            *child = Arc::new(c);
        }
        group.id = id;
        group.refresh_group_frame();
        self.surface_mut().replace(&[group]);
        true
    }

    /// Adds an object (fresh ids) on top of the active surface's top level.
    pub fn add(&mut self, object: Object) -> ObjectId {
        self.add_to(None, object)
    }

    /// Adds an object (fresh ids) at the top of `parent` (a group) or of the
    /// top level. Falls back to the top level if `parent` is not a group.
    pub fn add_to(&mut self, parent: Option<ObjectId>, mut object: Object) -> ObjectId {
        self.assign_fresh_ids(&mut object);
        object.refresh_group_frame();
        let id = object.id;
        let objects = &mut self.surface_mut().objects;
        let parent = parent.filter(|p| tree::get(objects, *p).is_some_and(|g| g.is_group()));
        let len = match parent {
            Some(p) => tree::get(objects, p).map_or(0, |g| g.children.len()),
            None => objects.len(),
        };
        tree::insert(objects, parent, len, vec![Arc::new(object)]);
        id
    }

    /// Adds copies of `objects` offset by `offset` at the top of `parent`,
    /// keeping their relative order; returns the new ids.
    pub fn add_copies(
        &mut self,
        objects: &[Object],
        offset: Vec2,
        parent: Option<ObjectId>,
    ) -> Vec<ObjectId> {
        objects
            .iter()
            .map(|o| {
                let mut copy = o.clone();
                copy.translate_deep(offset);
                self.add_to(parent, copy)
            })
            .collect()
    }

    /// Duplicates the given objects, each copy directly above its original.
    pub fn duplicate(&mut self, ids: &[ObjectId], offset: Vec2) -> Vec<ObjectId> {
        let originals = self.selected_objects(ids);
        let mut copies = Vec::new();
        for original in originals {
            let mut copy = original.clone();
            copy.translate_deep(offset);
            self.assign_fresh_ids(&mut copy);
            let id = copy.id;
            let objects = &mut self.surface_mut().objects;
            let parent = tree::parent_of(objects, original.id).flatten();
            let index = tree::find_path(objects, original.id)
                .and_then(|p| p.last().copied())
                .map_or(0, |i| i + 1);
            tree::insert(objects, parent, index, vec![Arc::new(copy)]);
            copies.push(id);
        }
        copies
    }

    /// The given objects (normalized: no descendant of another), in paint
    /// order, cloned.
    pub fn selected_objects(&self, ids: &[ObjectId]) -> Vec<Object> {
        let objects = &self.surface().objects;
        let ids = tree::normalize_selection(objects, ids);
        tree::in_paint_order(objects, &ids)
            .into_iter()
            .filter_map(|id| tree::get(objects, id).map(|o| (**o).clone()))
            .collect()
    }

    /// Wraps the given objects in a new group; returns its id.
    pub fn group(&mut self, ids: &[ObjectId]) -> Option<ObjectId> {
        let id = self.next_object_id();
        tree::group(&mut self.surface_mut().objects, ids, id)
    }

    /// Releases the given groups' children; returns their ids.
    pub fn ungroup(&mut self, ids: &[ObjectId]) -> Vec<ObjectId> {
        tree::ungroup(&mut self.surface_mut().objects, ids)
    }

    /// Moves objects in the active surface's tree.
    pub fn move_objects(&mut self, ids: &[ObjectId], placement: Placement) -> bool {
        tree::move_to(&mut self.surface_mut().objects, ids, placement)
    }

    /// Adds an asset, or returns the existing one with the same content.
    /// (See [`Asset::new`] for building one with a given id.)
    /// The bool is true when a new asset was stored.
    pub fn add_asset(
        &mut self,
        name: &str,
        kind: AssetKind,
        bytes: Arc<[u8]>,
        size: Size,
    ) -> (AssetId, bool) {
        let hash = *blake3::hash(&bytes).as_bytes();
        if let Some(existing) = self.assets.values().find(|a| a.hash == hash) {
            return (existing.id, false);
        }
        let id = AssetId(self.next_id);
        self.next_id += 1;
        self.assets.insert(
            id,
            Arc::new(Asset {
                id,
                name: name.to_owned(),
                kind,
                bytes,
                size,
                hash,
            }),
        );
        (id, true)
    }

    /// Number of image objects (on every surface) using `asset`.
    pub fn asset_usage(&self, asset: AssetId) -> usize {
        fn count(list: &[Arc<Object>], asset: AssetId) -> usize {
            list.iter()
                .map(|o| {
                    // An instance's content counts once, in its symbol.
                    let inside = if o.is_instance() {
                        0
                    } else {
                        count(&o.children, asset)
                    };
                    usize::from(o.kind == ShapeKind::Image { asset }) + inside
                })
                .sum()
        }
        let templates = self.template_assets().filter(|a| *a == asset).count();
        let mod_images = self.mod_image_assets().filter(|a| *a == asset).count();
        let symbols = self.symbols.iter().map(|s| &s.surface);
        self.surfaces
            .iter()
            .chain(symbols)
            .map(|s| count(&s.objects, asset))
            .sum::<usize>()
            + templates
            + mod_images
    }

    /// Assets used by surface templates (not listed as images).
    pub fn template_assets(&self) -> impl Iterator<Item = AssetId> + '_ {
        self.surfaces
            .iter()
            .filter_map(|s| s.template.as_ref().map(|t| t.asset))
    }

    /// Assets used by the mod settings' chosen images.
    pub fn mod_image_assets(&self) -> impl Iterator<Item = AssetId> + '_ {
        [self.mod_settings.icon, self.mod_settings.image]
            .into_iter()
            .flatten()
    }

    /// Assets that are project data rather than imported images: templates
    /// and the mod's chosen images.
    pub fn data_assets(&self) -> impl Iterator<Item = AssetId> + '_ {
        self.template_assets().chain(self.mod_image_assets())
    }

    /// Longest internal name the mod can have: shorter when a truck of the
    /// fleet has several main textures, whose paint jobs get a suffix.
    pub fn internal_name_limit(&self) -> usize {
        let split = self
            .vehicles
            .iter()
            .any(|v| v.game_data.as_ref().is_some_and(|g| g.main_count > 1));
        if split {
            SPLIT_INTERNAL_NAME_MAX
        } else {
            INTERNAL_NAME_MAX
        }
    }

    /// The mod's internal name, derived for the current fleet when the
    /// player hasn't typed one.
    pub fn internal_name(&self) -> String {
        self.mod_settings.internal_name(self.internal_name_limit())
    }

    /// Removes an unused asset; returns false if it is used or unknown.
    pub fn remove_asset(&mut self, asset: AssetId) -> bool {
        if self.asset_usage(asset) > 0 {
            return false;
        }
        self.assets.remove(&asset).is_some()
    }

    pub fn rename_asset(&mut self, asset: AssetId, name: &str) {
        if let Some(a) = self.assets.get_mut(&asset) {
            Arc::make_mut(a).name = name.to_owned();
        }
    }

    /// Adds a swatch of `color` named "Color N" unless a swatch already
    /// has it; returns whether it was added.
    pub fn add_to_palette(&mut self, color: Rgba) -> bool {
        self.add_swatch(color, DEFAULT_SWATCH_PREFIX).1
    }

    /// Adds a guide to the active surface; returns its index.
    pub fn add_guide(&mut self, guide: Guide) -> usize {
        let guides = &mut self.surface_mut().guides;
        guides.push(guide);
        guides.len() - 1
    }

    /// Moves guide `index` of the active surface along its axis.
    pub fn move_guide(&mut self, index: usize, position: f64) {
        if let Some(g) = self.surface_mut().guides.get_mut(index) {
            g.position = position;
        }
    }

    /// Removes guide `index` of the active surface.
    pub fn remove_guide(&mut self, index: usize) {
        let guides = &mut self.surface_mut().guides;
        if index < guides.len() {
            guides.remove(index);
        }
    }

    /// Removes every guide of the active surface.
    pub fn clear_guides(&mut self) {
        self.surface_mut().guides.clear();
    }

    /// Captures the document state (cheap: objects are shared).
    pub fn snapshot(&self, selection: &[ObjectId]) -> Snapshot {
        Snapshot {
            meta: self
                .surfaces
                .iter()
                .map(|s| SurfaceMeta {
                    name: s.name.clone(),
                    size: s.size,
                    template: s.template.clone(),
                })
                .collect(),
            vehicles: self.vehicles.clone(),
            surfaces: self.surfaces.iter().map(|s| s.objects.clone()).collect(),
            guides: self.surfaces.iter().map(|s| s.guides.clone()).collect(),
            active_surface: self.active_surface,
            brand: self.brand_kit(),
            symbols: self.symbols.clone(),
            editing_symbol: self.editing_symbol,
            mod_settings: self.mod_settings.clone(),
            game_versions: self.game_versions.clone(),
            assets: self.assets.clone(),
            selection: selection.to_vec(),
            points: Vec::new(),
        }
    }

    /// The library origins of the swatches, styles and symbols, by id.
    fn origins_by_id(&self) -> HashMap<u64, Option<LibraryKey>> {
        let palette = self.palette.iter().map(|s| (s.id.0, s.origin.clone()));
        let graphic = self
            .graphic_styles
            .iter()
            .map(|s| (s.id.0, s.origin.clone()));
        let text = self.text_styles.iter().map(|s| (s.id.0, s.origin.clone()));
        let symbols = self.symbols.iter().map(|s| (s.id.0, s.origin.clone()));
        palette.chain(graphic).chain(text).chain(symbols).collect()
    }

    /// Gives each swatch, style and symbol found in `origins` that origin.
    pub(crate) fn set_origins(&mut self, origins: &HashMap<u64, Option<LibraryKey>>) {
        let set = |id: u64, origin: &mut Option<LibraryKey>| {
            if let Some(o) = origins.get(&id) {
                origin.clone_from(o);
            }
        };
        for s in &mut self.palette {
            set(s.id.0, &mut s.origin);
        }
        for s in &mut self.graphic_styles {
            set(s.id.0, &mut s.origin);
        }
        for s in &mut self.text_styles {
            set(s.id.0, &mut s.origin);
        }
        for s in &mut self.symbols {
            set(s.id.0, &mut s.origin);
        }
    }

    /// Restores a snapshot and returns its selection. Ids stay unique because
    /// the id counter is never rewound.
    /// Drops the update flags of objects deleted or moved since `before`:
    /// a changed frame or path, so look and text edits keep them. Flags
    /// added since `before` are kept.
    pub fn unflag_moved(&mut self, before: &Snapshot) {
        let was_flagged: BTreeSet<ObjectId> = before
            .meta
            .iter()
            .filter_map(|m| m.template.as_ref())
            .flat_map(|t| t.flagged.iter().copied())
            .collect();
        if was_flagged.is_empty() {
            return;
        }
        let old = |id: ObjectId| before.surfaces.iter().find_map(|o| tree::get(o, id));
        for surface in &mut self.surfaces {
            let Some(t) = surface.template.as_mut() else {
                continue;
            };
            t.flagged
                .retain(|&id| match tree::get(&surface.objects, id) {
                    None => false,
                    Some(now) => match old(id) {
                        // Typing resizes a text's frame: an edit, not a move.
                        Some(then) if was_flagged.contains(&id) => {
                            (then.frame == now.frame && then.path == now.path)
                                || then.text != now.text
                        }
                        _ => true,
                    },
                });
        }
    }

    pub fn restore(&mut self, snapshot: &Snapshot) -> Vec<ObjectId> {
        // Library origins are not part of the history either: an element
        // that still exists keeps its current one.
        let origins = self.origins_by_id();
        // The surface list itself may differ (an update added surfaces).
        // Template opacity and visibility are not part of the history: keep
        // the current values of the same texture.
        let shown: Vec<(TextureKey, f32, bool)> = self
            .surfaces
            .iter()
            .filter_map(|s| s.template.as_ref())
            .map(|t| (t.key(), t.opacity, t.visible))
            .collect();
        self.surfaces = snapshot
            .meta
            .iter()
            .zip(&snapshot.surfaces)
            .zip(&snapshot.guides)
            .map(|((meta, objects), guides)| {
                let mut template = meta.template.clone();
                if let Some(t) = &mut template
                    && let Some((_, opacity, visible)) =
                        shown.iter().find(|(key, ..)| *key == t.key())
                {
                    t.opacity = *opacity;
                    t.visible = *visible;
                }
                Surface {
                    name: meta.name.clone(),
                    size: meta.size,
                    objects: objects.clone(),
                    guides: guides.clone(),
                    template,
                }
            })
            .collect();
        self.vehicles = snapshot.vehicles.clone();
        self.active_surface = snapshot.active_surface.min(self.surfaces.len() - 1);
        self.palette = snapshot.brand.palette.clone();
        self.graphic_styles = snapshot.brand.graphic_styles.clone();
        self.text_styles = snapshot.brand.text_styles.clone();
        self.symbols = snapshot.symbols.clone();
        self.editing_symbol = snapshot.editing_symbol;
        self.mod_settings = snapshot.mod_settings.clone();
        self.game_versions = snapshot.game_versions.clone();
        self.assets = snapshot.assets.clone();
        self.set_origins(&origins);
        snapshot.selection.clone()
    }
}

/// Name, size and template of a surface, in a snapshot.
#[derive(Clone, Debug, PartialEq)]
struct SurfaceMeta {
    name: String,
    size: Size,
    template: Option<SurfaceTemplate>,
}

/// Document state stored in the undo history.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    /// Name, size and template of each surface.
    meta: Vec<SurfaceMeta>,
    vehicles: Vec<ProjectVehicle>,
    surfaces: Vec<Vec<Arc<Object>>>,
    guides: Vec<Vec<Guide>>,
    active_surface: usize,
    brand: BrandKit,
    symbols: Vec<Symbol>,
    editing_symbol: Option<SymbolId>,
    mod_settings: ModSettings,
    game_versions: Vec<String>,
    assets: BTreeMap<AssetId, Arc<Asset>>,
    selection: Vec<ObjectId>,
    /// Selected path points (Direct Selection).
    points: Vec<PointRef>,
}

impl Snapshot {
    /// The same snapshot recording selected path points.
    pub fn with_points(mut self, points: impl IntoIterator<Item = PointRef>) -> Self {
        self.points = points.into_iter().collect();
        self
    }

    /// Selected path points at that time.
    pub fn points(&self) -> &[PointRef] {
        &self.points
    }

    /// The symbol being edited at that time.
    pub fn editing_symbol(&self) -> Option<SymbolId> {
        self.editing_symbol
    }

    /// Whether both snapshots hold the same document, ignoring selection
    /// (objects and points). Unchanged objects share their `Arc`, so this is mostly pointer checks.
    pub fn same_document(&self, other: &Snapshot) -> bool {
        self.active_surface == other.active_surface
            && self.vehicles == other.vehicles
            && self.meta.len() == other.meta.len()
            && self.meta.iter().zip(&other.meta).all(|(a, b)| {
                a.name == b.name
                    && a.size == b.size
                    && a.template.as_ref().map(SurfaceTemplate::document_part)
                        == b.template.as_ref().map(SurfaceTemplate::document_part)
            })
            && self.guides == other.guides
            && self.brand.same_content(&other.brand)
            && self.editing_symbol == other.editing_symbol
            && self.mod_settings == other.mod_settings
            && self.game_versions == other.game_versions
            && self.symbols.len() == other.symbols.len()
            && self
                .symbols
                .iter()
                .zip(&other.symbols)
                .all(|(a, b)| a.same_document(b))
            && self.assets.len() == other.assets.len()
            && self
                .assets
                .iter()
                .zip(&other.assets)
                .all(|((ka, a), (kb, b))| ka == kb && (Arc::ptr_eq(a, b) || a == b))
            && self.surfaces.len() == other.surfaces.len()
            && self.surfaces.iter().zip(&other.surfaces).all(|(a, b)| {
                a.len() == b.len() && a.iter().zip(b).all(|(x, y)| Arc::ptr_eq(x, y) || x == y)
            })
    }

    /// Whether both hold the same document as saved: [`Self::same_document`]
    /// and the same library origins, which the file records but the undo
    /// history doesn't.
    pub fn same_saved(&self, other: &Snapshot) -> bool {
        let symbols = |s: &Snapshot| {
            s.symbols
                .iter()
                .map(|s| s.origin.clone())
                .collect::<Vec<_>>()
        };
        self.same_document(other)
            && self.brand.origins().eq(other.brand.origins())
            && symbols(self) == symbols(other)
    }
}

/// A texture size as shown to the player: "4096²" when its sides are
/// equal, "8192 × 1024" (width × height) otherwise.
pub fn size_label(size: Size) -> String {
    if size.width == size.height {
        format!("{}²", size.width)
    } else {
        format!("{} × {}", size.width, size.height)
    }
}

#[cfg(test)]
mod tests {
    use kurbo::Size;

    use super::*;
    use crate::document::{Frame, ShapeKind};

    fn rect_at(x: f64, y: f64) -> Object {
        Object::new(
            ObjectId(0),
            ShapeKind::rectangle(),
            Frame::new(Point::new(x, y), Size::new(100.0, 100.0), 0.0),
        )
    }

    #[test]
    fn rectangular_surface() {
        let s = Surface::new("Base", Size::new(8192.0, 1024.0));
        assert_eq!(s.bounds(), Rect::new(0.0, 0.0, 8192.0, 1024.0));
        assert_eq!(s.max_side(), 8192.0);
        assert_eq!(s.size_label(), "8192 × 1024");
        let square = Surface::new("Cab", Size::new(4096.0, 4096.0));
        assert_eq!(square.bounds(), Rect::new(0.0, 0.0, 4096.0, 4096.0));
        assert_eq!(square.size_label(), "4096²");
    }

    #[test]
    fn name_is_trimmed() {
        let p = Project::new("  ACE Logistics ", TextureResolution::R2048);
        assert_eq!(p.name, "ACE Logistics");
        assert_eq!(p.texture_size(), (2048, 2048));
    }

    #[test]
    fn resolution_labels() {
        assert_eq!(TextureResolution::R2048.label(), "2048 × 2048");
        assert_eq!(TextureResolution::default().side(), 4096);
    }

    #[test]
    fn ids_are_unique_and_stable() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let a = p.add(rect_at(100.0, 100.0));
        let b = p.add(rect_at(150.0, 150.0));
        assert_ne!(a, b);
        p.surface_mut().bring_forward(&[a]);
        let mut moved = (**p.surface().get(a).unwrap()).clone();
        moved.frame.center = Point::new(500.0, 500.0);
        p.surface_mut().replace(&[moved]);
        assert!(p.surface().get(a).is_some());
        assert_eq!(p.surface().index_of(a), Some(1));
    }

    #[test]
    fn topmost_object_wins() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let _bottom = p.add(rect_at(100.0, 100.0));
        let top = p.add(rect_at(120.0, 120.0));
        assert_eq!(
            p.surface().hit_test(Point::new(110.0, 110.0), 0.0),
            Some(Hit { top, inner: top })
        );
        assert_eq!(p.surface().hit_test(Point::new(1000.0, 1000.0), 0.0), None);
    }

    #[test]
    fn marquee_selects_touching_objects() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let a = p.add(rect_at(100.0, 100.0));
        let b = p.add(rect_at(300.0, 100.0));
        let _c = p.add(rect_at(800.0, 800.0));
        let hits = p
            .surface()
            .objects_in_rect(Rect::new(140.0, 90.0, 260.0, 110.0));
        assert_eq!(hits, vec![a, b]);
    }

    #[test]
    fn marquee_misses_ellipse_corner() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let mut e = rect_at(100.0, 100.0);
        e.kind = ShapeKind::Ellipse;
        p.add(e);
        // Inside the bounding box corner, outside the ellipse.
        assert!(
            p.surface()
                .objects_in_rect(Rect::new(52.0, 52.0, 58.0, 58.0))
                .is_empty()
        );
    }

    #[test]
    fn duplicate_offsets_and_stacks_on_top() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let a = p.add(rect_at(100.0, 100.0));
        let copies = p.duplicate(&[a], Vec2::new(20.0, 20.0));
        assert_eq!(copies.len(), 1);
        let copy = p.surface().get(copies[0]).unwrap();
        assert_eq!(copy.frame.center, Point::new(120.0, 120.0));
        assert_eq!(p.surface().index_of(copies[0]), Some(1));
    }

    #[test]
    fn reorder_steps() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let a = p.add(rect_at(0.0, 0.0));
        let b = p.add(rect_at(0.0, 0.0));
        let c = p.add(rect_at(0.0, 0.0));
        p.surface_mut().bring_forward(&[a]);
        let order: Vec<_> = p.surface().objects.iter().map(|o| o.id).collect();
        assert_eq!(order, vec![b, a, c]);
        p.surface_mut().send_backward(&[c]);
        let order: Vec<_> = p.surface().objects.iter().map(|o| o.id).collect();
        assert_eq!(order, vec![b, c, a]);
    }

    #[test]
    fn palette_has_no_duplicates_and_is_snapshotted() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let red = Rgba::rgb(255, 0, 0);
        assert!(p.add_to_palette(red));
        assert!(!p.add_to_palette(red));
        let colors = |p: &Project| p.palette.iter().map(|s| s.color).collect::<Vec<_>>();
        assert_eq!(colors(&p), vec![red]);
        let snap = p.snapshot(&[]);
        p.palette.clear();
        assert!(!snap.same_document(&p.snapshot(&[])));
        p.restore(&snap);
        assert_eq!(colors(&p), vec![red]);
    }

    #[test]
    fn duplicate_inside_group_stays_in_group() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let a = p.add(rect_at(100.0, 100.0));
        let g = p.group(&[a]).unwrap();
        let copies = p.duplicate(&[a], Vec2::new(20.0, 20.0));
        let group = p.surface().get(g).unwrap();
        assert_eq!(group.children.len(), 2);
        assert_eq!(group.children[1].id, copies[0]);
        assert_eq!(group.children[1].frame.center, Point::new(120.0, 120.0));
    }

    #[test]
    fn copies_of_groups_get_fresh_ids() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let a = p.add(rect_at(100.0, 100.0));
        let b = p.add(rect_at(300.0, 100.0));
        let g = p.group(&[a, b]).unwrap();
        let copy = p.duplicate(&[g], Vec2::new(0.0, 50.0))[0];
        let copied = p.surface().get(copy).unwrap();
        assert!(copied.children.iter().all(|c| c.id != a && c.id != b));
        assert_eq!(copied.children[0].frame.center, Point::new(100.0, 150.0));
    }

    #[test]
    fn add_to_group() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let a = p.add(rect_at(100.0, 100.0));
        let g = p.group(&[a]).unwrap();
        let b = p.add_to(Some(g), rect_at(500.0, 100.0));
        let group = p.surface().get(g).unwrap();
        assert_eq!(group.children.last().unwrap().id, b);
        assert_eq!(group.frame.size.width, 500.0);
    }

    fn image(asset: AssetId) -> Object {
        Object::new(
            ObjectId(0),
            ShapeKind::Image { asset },
            Frame::new(Point::new(10.0, 10.0), Size::new(80.0, 40.0), 0.0),
        )
    }

    #[test]
    fn from_parts_keeps_ids_and_continues_after_them() {
        let mut p = Project::new("a", TextureResolution::R2048);
        p.add(Object::new(
            ObjectId(0),
            ShapeKind::Ellipse,
            crate::document::Frame::new(Point::new(1.0, 1.0), Size::new(2.0, 2.0), 0.0),
        ));
        let (asset, _) = p.add_asset(
            "logo",
            AssetKind::Svg,
            Arc::from(&b"<svg/>"[..]),
            Size::new(1.0, 1.0),
        );
        let mut q = Project::from_parts(
            &p.name,
            p.resolution,
            p.surfaces.clone(),
            0,
            crate::BrandKit {
                palette: vec![crate::Swatch {
                    id: crate::document::SwatchId(900),
                    name: "Red".into(),
                    color: Rgba::rgb(1, 2, 3),
                    origin: None,
                }],
                ..Default::default()
            },
            p.assets.clone(),
        );
        assert_eq!(q.surfaces, p.surfaces);
        assert!(q.next_object_id().0 > asset.0);
        assert!(
            q.next_object_id().0 > 900,
            "ids continue after the swatches'"
        );
    }

    #[test]
    fn assets_are_deduplicated_and_counted() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let bytes: Arc<[u8]> = Arc::from(&b"fake png"[..]);
        let (a, new) = p.add_asset(
            "logo",
            AssetKind::Raster,
            bytes.clone(),
            Size::new(800.0, 400.0),
        );
        assert!(new);
        let (b, new) = p.add_asset(
            "logo copy",
            AssetKind::Raster,
            bytes,
            Size::new(800.0, 400.0),
        );
        assert!(!new);
        assert_eq!(a, b);
        assert_eq!(p.assets.len(), 1);
        let img = p.add(image(a));
        p.duplicate(&[img], Vec2::new(10.0, 10.0));
        assert_eq!(p.asset_usage(a), 2);
        assert!(!p.remove_asset(a), "used assets cannot be removed");
    }

    #[test]
    fn removed_asset_comes_back_with_undo_snapshot() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let (a, _) = p.add_asset(
            "badge",
            AssetKind::Svg,
            Arc::from(&b"<svg/>"[..]),
            Size::new(10.0, 10.0),
        );
        let snap = p.snapshot(&[]);
        assert!(p.remove_asset(a));
        assert!(!snap.same_document(&p.snapshot(&[])));
        p.restore(&snap);
        assert!(p.assets.contains_key(&a));
        p.rename_asset(a, "Badge");
        assert_eq!(p.assets[&a].name, "Badge");
    }

    #[test]
    fn remove_and_restore() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let a = p.add(rect_at(0.0, 0.0));
        let snap = p.snapshot(&[a]);
        assert_eq!(p.surface_mut().remove(&[a]), 1);
        assert!(p.surface().objects.is_empty());
        assert_eq!(p.restore(&snap), vec![a]);
        assert_eq!(p.surface().objects.len(), 1);
        // The id counter is not rewound.
        assert_ne!(p.next_object_id(), a);
    }

    #[test]
    fn point_selection_is_kept_but_not_part_of_the_document() {
        let mut p = Project::new("P", TextureResolution::R2048);
        let id = p.add(rect_at(10.0, 10.0));
        let point = PointRef::new(id, crate::document::NodeRef::new(0, 2));
        let a = p.snapshot(&[id]).with_points([point]);
        let b = p.snapshot(&[id]);
        assert!(a.same_document(&b));
        assert_eq!(a.points(), &[point]);
        assert!(b.points().is_empty());
    }

    #[test]
    fn guides_round_trip_through_snapshots() {
        let mut p = Project::new("P", TextureResolution::R2048);
        assert!(p.surface().guides.is_empty());
        let before = p.snapshot(&[]);
        let i = p.add_guide(Guide::new(Axis::Vertical, 500.0));
        p.add_guide(Guide::new(Axis::Horizontal, 1024.0));
        let added = p.snapshot(&[]);
        assert!(!added.same_document(&before));
        p.move_guide(i, 2048.0);
        assert!(!p.snapshot(&[]).same_document(&added));
        p.restore(&added);
        assert_eq!(p.surface().guides[0], Guide::new(Axis::Vertical, 500.0));
        p.remove_guide(0);
        assert_eq!(
            p.surface().guides,
            vec![Guide::new(Axis::Horizontal, 1024.0)]
        );
        p.clear_guides();
        assert!(p.surface().guides.is_empty());
        p.restore(&before);
        assert!(p.surface().guides.is_empty());
    }

    #[test]
    fn replace_with_group_keeps_place_and_id() {
        let mut p = Project::new("P", TextureResolution::R2048);
        let a = p.add(rect_at(100.0, 100.0));
        let t = p.add(rect_at(300.0, 300.0));
        let b = p.add(rect_at(900.0, 100.0));
        let g = p.group(&[t, b]).unwrap();
        let before = p.snapshot(&[t]);
        let letters = vec![
            Arc::new(rect_at(280.0, 300.0)),
            Arc::new(rect_at(320.0, 300.0)),
        ];
        let group = Object::group(ObjectId(0), letters);
        assert!(p.replace_with_group(t, group));
        let parent = p.surface().get(g).unwrap();
        assert_eq!(parent.children.len(), 2);
        let replaced = &parent.children[0];
        assert_eq!(replaced.id, t, "same id, same index in its parent");
        assert!(replaced.is_group());
        let ids: Vec<ObjectId> = replaced.children.iter().map(|c| c.id).collect();
        for id in &ids {
            assert!(![a, t, b, g].contains(id), "fresh ids");
        }
        assert_ne!(ids[0], ids[1]);
        // The parent's bounds follow the new content.
        assert!(parent.bounding_box().x0 <= 230.0);
        p.restore(&before);
        assert!(!p.surface().get(t).unwrap().is_group());
        assert!(!p.replace_with_group(ObjectId(999), Object::group(ObjectId(0), vec![])));
    }

    fn template(texture: &str, asset: AssetId) -> SurfaceTemplate {
        keyed("a.b", texture, TexturePart::Main, asset)
    }

    fn keyed(package: &str, texture: &str, part: TexturePart, asset: AssetId) -> SurfaceTemplate {
        SurfaceTemplate {
            package_id: package.into(),
            texture_id: texture.into(),
            part,
            asset,
            layout_version: 1,
            game_ids: Vec::new(),
            also_paints: Vec::new(),
            main_index: None,
            opacity: SurfaceTemplate::DEFAULT_OPACITY,
            visible: true,
            status: TemplateStatus::Current,
            flagged: BTreeSet::new(),
        }
    }

    #[test]
    fn restoring_rebuilds_the_surface_list_and_vehicle() {
        let mut p = Project::new("T", TextureResolution::R2048);
        let (asset, _) = p.add_asset(
            "cabin",
            AssetKind::Raster,
            Arc::from(&b"png"[..]),
            Size::new(8.0, 8.0),
        );
        p.surfaces[0].template = Some(template("cabin", asset));
        let before = p.snapshot(&[]);
        // An update: a new surface and a recorded version.
        let mut chassis = Surface::new("Chassis", Size::new(1024.0, 1024.0));
        chassis.template = Some(template("chassis", asset));
        p.surfaces.push(chassis);
        p.vehicles = vec![vehicle("a.b")];
        assert!(!before.same_document(&p.snapshot(&[])));
        p.restore(&before);
        assert_eq!(p.surfaces.len(), 1);
        assert!(p.vehicles.is_empty());
        assert_eq!(p.asset_usage(asset), 1, "templates count as uses");
        assert_eq!(p.template_assets().collect::<Vec<_>>(), vec![asset]);
    }

    #[test]
    fn template_opacity_and_visibility_survive_undo() {
        let mut p = Project::new("T", TextureResolution::R2048);
        let (asset, _) = p.add_asset(
            "cabin",
            AssetKind::Raster,
            Arc::from(&b"png"[..]),
            Size::new(8.0, 8.0),
        );
        p.surfaces[0].template = Some(template("cabin", asset));
        let before = p.snapshot(&[]);
        p.add(rect_at(100.0, 100.0));
        let t = p.surfaces[0].template.as_mut().unwrap();
        t.opacity = 0.2;
        t.visible = false;
        // Changing them alone is not a document change.
        let mut q = p.clone();
        q.surfaces[0].template.as_mut().unwrap().opacity = 0.9;
        assert!(p.snapshot(&[]).same_document(&q.snapshot(&[])));
        p.restore(&before);
        assert!(p.surface().objects.is_empty());
        let t = p.surfaces[0].template.as_ref().unwrap();
        assert_eq!((t.opacity, t.visible), (0.2, false));
    }

    #[test]
    fn library_origins_survive_undo_and_count_for_saving() {
        let mut p = Project::new("T", TextureResolution::R2048);
        p.add_to_palette(Rgba::rgb(0, 128, 0));
        let before = p.snapshot(&[]);
        p.add(rect_at(100.0, 100.0));
        let key = LibraryKey("0123456789abcdef0123456789abcdef".into());
        p.palette[0].origin = Some(key.clone());
        // Recording an origin alone is no undo step, but is a change to save.
        let mut q = p.clone();
        q.palette[0].origin = None;
        assert!(p.snapshot(&[]).same_document(&q.snapshot(&[])));
        assert!(!p.snapshot(&[]).same_saved(&q.snapshot(&[])));
        assert!(p.snapshot(&[]).same_saved(&p.snapshot(&[])));
        // Restoring an older snapshot keeps the origin recorded after it.
        p.restore(&before);
        assert!(p.surface().objects.is_empty());
        assert_eq!(p.palette[0].origin, Some(key));
    }

    #[test]
    fn mod_images_are_used_assets() {
        let mut p = Project::new("ACE Logistics", TextureResolution::R2048);
        assert_eq!(p.mod_settings.name, "ACE Logistics");
        let (icon, _) = p.add_asset(
            "icon",
            AssetKind::Raster,
            Arc::from(&b"png"[..]),
            Size::new(8.0, 8.0),
        );
        p.mod_settings.icon = Some(icon);
        assert_eq!(p.asset_usage(icon), 1);
        assert!(!p.remove_asset(icon), "a chosen mod image is kept");
        assert_eq!(p.data_assets().collect::<Vec<_>>(), vec![icon]);
        p.mod_settings.icon = None;
        assert!(p.remove_asset(icon));
    }

    #[test]
    fn mod_settings_are_part_of_the_history() {
        let mut p = Project::new("ACE", TextureResolution::R2048);
        let before = p.snapshot(&[]);
        p.mod_settings.price = 9000;
        assert!(!before.same_document(&p.snapshot(&[])));
        p.restore(&before);
        assert_eq!(p.mod_settings.price, 5000);
    }

    #[test]
    fn game_versions_are_part_of_the_history() {
        let mut p = Project::new("ACE", TextureResolution::R2048);
        assert!(p.game_versions.is_empty());
        let before = p.snapshot(&[]);
        p.game_versions = vec!["1.56.*".into()];
        assert!(!before.same_document(&p.snapshot(&[])));
        p.restore(&before);
        assert!(p.game_versions.is_empty());
    }

    #[test]
    fn side_view_choices_are_not_part_of_the_history() {
        let mut p = Project::new("ACE", TextureResolution::R2048);
        let before = p.snapshot(&[]);
        p.side_view
            .entry("official.ets2.daf.xd".into())
            .or_default()
            .insert("r_deflector".into(), None);
        assert!(before.same_document(&p.snapshot(&[])));
        p.restore(&before);
        assert_eq!(p.side_view["official.ets2.daf.xd"]["r_deflector"], None);
    }

    #[test]
    fn internal_name_limit_follows_the_fleet() {
        let mut p = Project::new("ACE Logistics", TextureResolution::R2048);
        p.vehicles = vec![vehicle("a.b")];
        assert_eq!(p.internal_name(), "ace_logistic");
        p.vehicles[0].game_data = Some(GameData {
            path: "a.b".into(),
            versions: "*".into(),
            alt_uv: false,
            colour_picker: false,
            requires: Vec::new(),
            main_count: 2,
        });
        assert_eq!(p.internal_name(), "ace_logist");
    }

    fn vehicle(package: &str) -> ProjectVehicle {
        ProjectVehicle {
            package_id: package.into(),
            version: "1.0.0".into(),
            name: format!("Vehicle {package}"),
            brand: "B".into(),
            kind: "truck".into(),
            game: "ets2".into(),
            game_data: None,
        }
    }

    /// A fleet: truck a.b (main standard and high, accessory chassis) and
    /// trailer c.d (main base, accessory chassis).
    fn fleet() -> Project {
        let mut p = Project::new("F", TextureResolution::R2048);
        let (asset, _) = p.add_asset(
            "t",
            AssetKind::Raster,
            Arc::from(&b"png"[..]),
            Size::new(8.0, 8.0),
        );
        p.surfaces = [
            ("a.b", "standard", TexturePart::Main),
            ("a.b", "high", TexturePart::Main),
            ("a.b", "chassis", TexturePart::Accessory),
            ("c.d", "base", TexturePart::Main),
            ("c.d", "chassis", TexturePart::Accessory),
        ]
        .iter()
        .map(|(pkg, tex, part)| {
            let mut s = Surface::new(*tex, Size::new(1024.0, 1024.0));
            s.template = Some(keyed(pkg, tex, *part, asset));
            s
        })
        .collect();
        p.vehicles = vec![vehicle("a.b"), vehicle("c.d")];
        p
    }

    #[test]
    fn fleet_ranges_names_and_artwork() {
        let mut p = fleet();
        assert_eq!(p.game(), Some("ets2"));
        assert_eq!(p.vehicle_range("a.b"), 0..3);
        assert_eq!(p.vehicle_range("c.d"), 3..5);
        assert_eq!(p.vehicle_range("x.y"), 5..5);
        assert_eq!(p.vehicle_of(3).unwrap().package_id, "c.d");
        assert_eq!(
            p.surface_names(1),
            Some(("Vehicle a.b".into(), "high".into()))
        );
        assert!(
            p.surfaces[4]
                .template
                .as_ref()
                .unwrap()
                .is_of("c.d", "chassis")
        );
        assert!(!p.has_artwork(0..5));
        p.active_surface = 2;
        p.add(rect_at(10.0, 10.0));
        assert!(p.has_artwork(2..3));
        assert!(!p.has_artwork(0..2));
        assert!(Project::new("x", TextureResolution::R2048).game().is_none());
    }

    #[test]
    fn same_texture_id_in_two_vehicles_keeps_its_own_settings() {
        let mut p = fleet();
        let before = p.snapshot(&[]);
        p.add(rect_at(10.0, 10.0));
        p.surfaces[2].template.as_mut().unwrap().opacity = 0.1;
        p.surfaces[4].template.as_mut().unwrap().opacity = 0.9;
        p.restore(&before);
        assert_eq!(p.surfaces[2].template.as_ref().unwrap().opacity, 0.1);
        assert_eq!(p.surfaces[4].template.as_ref().unwrap().opacity, 0.9);
        // Vehicles and texture parts are part of the document.
        let mut q = p.clone();
        q.vehicles[0].version = "2.0.0".into();
        assert!(!p.snapshot(&[]).same_document(&q.snapshot(&[])));
        let mut q = p.clone();
        q.surfaces[2].template.as_mut().unwrap().part = TexturePart::Main;
        assert!(!p.snapshot(&[]).same_document(&q.snapshot(&[])));
    }

    #[test]
    fn typing_keeps_a_flag_and_moving_drops_it() {
        use crate::document::{CharStyle, TextBlock};
        let mut p = Project::new("p", TextureResolution::R2048);
        let mut text = Object::new(
            ObjectId(0),
            ShapeKind::Text,
            Frame::new(Point::new(100.0, 100.0), Size::new(80.0, 20.0), 0.0),
        );
        text.text = Some(TextBlock::new("ACE", CharStyle::default()));
        let id = p.add(text);
        let mut t = template("cabin", AssetId(1));
        t.status = TemplateStatus::LayoutChanged;
        t.flagged = BTreeSet::from([id]);
        p.surfaces[0].template = Some(t);
        let flagged = |p: &Project| {
            p.surfaces[0]
                .template
                .as_ref()
                .unwrap()
                .flagged
                .contains(&id)
        };
        let edit = |p: &mut Project, f: &dyn Fn(&mut Object)| {
            let before = p.snapshot(&[]);
            let mut o = (**p.surface().get(id).unwrap()).clone();
            f(&mut o);
            p.surface_mut().replace(&[o]);
            p.unflag_moved(&before);
        };
        // New text, wider frame: still flagged.
        edit(&mut p, &|o| {
            o.text = Some(TextBlock::new("ACE LOGISTICS", CharStyle::default()));
            o.frame.size = Size::new(300.0, 20.0);
        });
        assert!(flagged(&p));
        edit(&mut p, &|o| o.frame.center = Point::new(400.0, 100.0));
        assert!(!flagged(&p));
    }
}
