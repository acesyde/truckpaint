//! Extracts official truck and owned trailer definitions and painted geometry from ETS2.

mod geometry;
mod source;
mod syntax;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// License of generated packages: the templates are SCS Software's.
pub const LICENSE: &str =
    "Personal modding only, not for sale. Game content belongs to SCS Software.";

use serde_json::{Value, json};

use source::Sources;
use syntax::Section;

/// Inputs of the `tpv extract` command.
pub struct Options {
    pub installation: PathBuf,
    pub vehicle: Option<String>,
    pub output: PathBuf,
    /// A folder of previously extracted packages: unchanged templates keep their
    /// layout version, changed ones get the next one.
    pub previous: Option<PathBuf>,
}

/// A report is still delivered when individual vehicles fail.
pub struct Batch {
    pub directory: PathBuf,
    pub failed: usize,
}

fn units(section: &Section) -> &[Section] {
    if let Some(wrapper) = section.children.iter().find(|s| s.header == "SiiNunit") {
        &wrapper.children
    } else {
        &section.children
    }
}

fn array(section: &Section, field: &str) -> Vec<String> {
    section
        .fields
        .get(field)
        .into_iter()
        .flatten()
        .map(|s| syntax::unquote(s).to_owned())
        .collect()
}

fn reserve(parent: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    // create_dir fails on an existing directory, so a batch never reuses another's.
    for attempt in 0u32.. {
        let path = parent.join(format!("ets2-{stamp}-{}-{attempt}", std::process::id()));
        match std::fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("{}: {e}", path.display())),
        }
    }
    unreachable!()
}

/// Writes beside the target then renames, so an interrupted write leaves no partial file.
fn publish(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let error = |e: std::io::Error| format!("{}: {e}", path.display());
    let name = path.file_name().ok_or("invalid output path")?;
    let partial = path.with_file_name(format!(".{}.partial", name.to_string_lossy()));
    std::fs::write(&partial, bytes)
        .and_then(|()| std::fs::rename(&partial, path))
        .map_err(|e| {
            let _ = std::fs::remove_file(&partial);
            error(e)
        })
}

fn definition(
    sources: &Sources,
    path: &str,
    evidence: &mut BTreeSet<String>,
) -> Result<Section, String> {
    let text = sources.resolved(path, &mut BTreeSet::new(), evidence)?;
    syntax::parse(&text).map_err(|e| format!("{path}: {e}"))
}

/// English game strings by key, from the localization files of the installed game.
type Names = BTreeMap<String, String>;

// DLC trailer series (Schwarzmüller) are only named in `localization.sui`.
const NAME_FILES: [&str; 3] = [
    "/locale/common.sui",
    "/locale/en_us/localization.sui",
    "/locale/en_us/accessories.sui",
];

/// Brand, series and cabin names; absent files fall back to the keys.
fn names(sources: &Sources) -> Result<Names, String> {
    let mut names = Names::new();
    for path in NAME_FILES {
        let text = match sources.read(path) {
            Ok(text) => text,
            Err(e) if e.starts_with("missing game file ") => continue,
            Err(e) => return Err(e),
        };
        // Values may be followed by a comment: keep only the leading quoted string.
        let quoted = |value: &str| {
            let value = value.trim();
            value
                .strip_prefix('"')
                .and_then(|v| v.split_once('"'))
                .map_or(value, |(inner, _)| inner)
                .to_owned()
        };
        let mut key = None;
        for line in text.lines().map(str::trim) {
            if let Some(value) = line.strip_prefix("key[]:") {
                key = Some(quoted(value));
            } else if let (Some(value), Some(key)) = (line.strip_prefix("val[]:"), key.take()) {
                names.insert(key, quoted(value));
            }
        }
    }
    Ok(names)
}

fn label(value: &str, names: &Names) -> String {
    let key = value.strip_prefix("@@").and_then(|v| v.strip_suffix("@@"));
    if let Some(name) = key.and_then(|key| names.get(key)) {
        return name.clone();
    }
    key.map(|v| {
        v.trim_start_matches("brand_")
            .trim_start_matches("series_")
            .trim_start_matches("cabin_")
            .replace('_', " ")
            .to_uppercase()
    })
    .unwrap_or_else(|| value.to_owned())
}

struct Painted {
    id: String,
    group: String,
    name: String,
    cabins: BTreeSet<String>,
    mesh: Vec<geometry::Mesh>,
    selection: Vec<Value>,
    source: String,
}

#[derive(Default)]
struct Diagnostics {
    missing_dependencies: Vec<Value>,
    dimensions: Vec<Value>,
}

fn crop_size(mask: &str) -> Option<[u32; 2]> {
    let (_, rest) = mask.split_once("_size_")?;
    let (width, height) = rest.split_once('x')?;
    let height = height
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>();
    Some([width.parse().ok()?, height.parse().ok()?])
}

/// Reads a model's descriptor and geometry and selects the painted pieces of one look and variant.
fn model_geometry(
    sources: &Sources,
    model: &str,
    look: &str,
    variant: &str,
    alt_uv: Option<bool>,
) -> Result<(Vec<geometry::Mesh>, Value), String> {
    let stem = model.trim_end_matches(".pmd");
    let (pmd, pmg) = (format!("{stem}.pmd"), format!("{stem}.pmg"));
    let descriptor = tp_scs::Descriptor::parse(&pmd, &sources.bytes(&pmd)?)?;
    let geometry = tp_scs::Model::parse(&pmg, &sources.bytes(&pmg)?)?;
    let effects = |look: &tp_scs::Look| {
        look.materials
            .iter()
            .map(|file| {
                let path = source::virtual_path(&pmd, file)?;
                tp_scs::material_effect(&path, &sources.bytes(&path)?)
            })
            .collect::<Result<Vec<_>, _>>()
    };
    let mesh = geometry::painted(&geometry, &descriptor, effects, look, variant, alt_uv)
        .map_err(|e| format!("{model}, {look}/{variant}: {e}"))?;
    let surfaces = mesh.iter().map(|m| {
        let min = std::array::from_fn::<_, 2, _>(|axis| m.uv.iter().map(|uv| uv[axis]).fold(f64::INFINITY, f64::min));
        let max = std::array::from_fn::<_, 2, _>(|axis| m.uv.iter().map(|uv| uv[axis]).fold(f64::NEG_INFINITY, f64::max));
        let outside = m.uv.iter().filter(|uv| uv.iter().any(|v| !(0.0..=1.0).contains(v))).count();
        json!({"shader": m.shader, "uv_alias": if m.alt_uv { "_TEXCOORD2" } else { "_TEXCOORD1" }, "uv_bounds": {"min": min, "max": max}, "out_of_bounds_vertices": outside})
    }).collect::<Vec<_>>();
    let selection =
        json!({"model": model, "look": look, "variant": variant, "painted_surfaces": surfaces});
    Ok((mesh, selection))
}

/// Refusal of a trailer whose paint jobs only set plain colours: reported as
/// skipped, not as a failure, since no livery is possible for it in the game.
const NOT_PAINTABLE: &str =
    "not paintable: paint jobs only set plain colours, with no mask and no override";

/// Accessory id -> paint mask -> definitions that assign it.
type Masks = BTreeMap<String, BTreeMap<String, BTreeSet<String>>>;

const MODEL_FIELDS: [&str; 3] = ["model", "detail_model", "exterior_model"];

/// The (model, look, variant) triples a unit paints: EU models first, then UK ones.
fn model_selections(unit: &Section, path: &str) -> Result<Vec<[String; 3]>, String> {
    let has = |name: &str| unit.fields.contains_key(name);
    // A UK variant may reuse the EU model rather than provide a model_uk field.
    let uk_reuses_eu = has("variant_uk") && MODEL_FIELDS.iter().all(|f| !has(&format!("{f}_uk")));
    let pick = |name: &str, uk: bool| {
        let uk_name = format!("{name}_uk");
        let key = if uk && has(&uk_name) {
            uk_name.as_str()
        } else {
            name
        };
        unit.field(key).unwrap_or("default").to_owned()
    };
    let mut result: Vec<[String; 3]> = Vec::new();
    for uk in [false, true] {
        for base in MODEL_FIELDS {
            let field = if uk && !uk_reuses_eu {
                format!("{base}_uk")
            } else {
                base.to_owned()
            };
            if !has(&field) {
                continue;
            }
            let model = source::virtual_path(path, unit.field(&field)?)?;
            if !model.ends_with(".pmd") {
                return Err(format!("unsupported model {model}"));
            }
            let selection = [model, pick("look", uk), pick("variant", uk)];
            if !result.contains(&selection) {
                result.push(selection);
            }
        }
    }
    Ok(result)
}

/// What the game defines a vehicle as: where its definitions live and how
/// they are laid out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Truck,
    Trailer,
}

impl Kind {
    /// The folder under `/def/vehicle/`.
    fn folder(self) -> &'static str {
        match self {
            Kind::Truck => "truck",
            Kind::Trailer => "trailer_owned",
        }
    }

    /// The unit of `data.sii` holding the brand and series.
    fn data_unit(self) -> &'static str {
        match self {
            Kind::Truck => "accessory_truck_data :",
            Kind::Trailer => "accessory_trailer_data :",
        }
    }

    /// Definition folders holding painted parts.
    fn painted_groups(self) -> [&'static str; 3] {
        match self {
            Kind::Truck => ["cabin/", "chassis/", "accessory/"],
            Kind::Trailer => ["body/", "chassis/", "accessory/"],
        }
    }

    fn public(self) -> tp_vehicles::Kind {
        match self {
            Kind::Truck => tp_vehicles::Kind::Truck,
            Kind::Trailer => tp_vehicles::Kind::Trailer,
        }
    }

    /// The package manifest's `kind`.
    fn manifest(self) -> &'static str {
        match self {
            Kind::Truck => "truck",
            Kind::Trailer => "trailer",
        }
    }
}

/// The accessory ids sharing each independent texture. A trailer's ids that are
/// overridden by the same masks in the same paint jobs share one texture, as the
/// game paints them from one region; every other id has its own.
fn accessory_groups(
    kind: Kind,
    sizes: &BTreeMap<String, [u32; 2]>,
    masks: &Masks,
) -> Vec<Vec<String>> {
    if kind == Kind::Truck {
        return sizes.keys().map(|id| vec![id.clone()]).collect();
    }
    let mut groups = BTreeMap::<BTreeSet<(&String, &String)>, Vec<String>>::new();
    for id in sizes.keys() {
        let signature = masks[id]
            .iter()
            .flat_map(|(mask, paths)| paths.iter().map(move |path| (path, mask)))
            .collect();
        groups.entry(signature).or_default().push(id.clone());
    }
    let mut groups = groups.into_values().collect::<Vec<_>>();
    groups.sort();
    groups
}

struct Parts {
    painted: BTreeMap<String, Painted>,
    cabins: BTreeMap<String, String>,
    seen: BTreeSet<String>,
    /// Defined accessories whose definition names no exterior model.
    modelless: BTreeSet<String>,
}

#[derive(Default)]
struct Textures {
    main: Vec<Value>,
    accessories: Vec<Value>,
    templates: BTreeMap<String, Vec<u8>>,
    coverage: Vec<Value>,
    /// A template is new, different or gone compared with the previous package.
    changed: bool,
}

/// One truck's extraction; what it gathered stays readable after a failure.
struct Build<'a> {
    sources: &'a Sources,
    names: &'a Names,
    previous: Option<&'a tp_vehicles::Package>,
    kind: Kind,
    vehicle: &'a str,
    root: String,
    evidence: BTreeSet<String>,
    stage: &'static str,
    diagnostics: Diagnostics,
}

impl<'a> Build<'a> {
    fn new(
        sources: &'a Sources,
        names: &'a Names,
        previous: Option<&'a tp_vehicles::Package>,
        kind: Kind,
        vehicle: &'a str,
    ) -> Self {
        Self {
            sources,
            names,
            previous,
            kind,
            vehicle,
            root: format!("/def/vehicle/{}/{vehicle}/", kind.folder()),
            evidence: BTreeSet::new(),
            stage: "definitions",
            diagnostics: Diagnostics::default(),
        }
    }

    fn definition(&mut self, path: &str) -> Result<Section, String> {
        definition(self.sources, path, &mut self.evidence)
    }

    fn definition_paths(&self) -> Vec<String> {
        let mut paths = self.sources.paths_under(&self.root);
        paths.retain(|p| p.ends_with(".sii"));
        paths.sort_by_key(|p| !p.starts_with(&format!("{}cabin/", self.root)));
        paths
    }

    fn identity(&mut self) -> Result<(String, String), String> {
        let data = self.definition(&format!("{}data.sii", self.root))?;
        let vehicle = units(&data)
            .iter()
            .find(|u| u.header.starts_with(self.kind.data_unit()))
            .ok_or("missing vehicle metadata")?;
        let info = array(vehicle, "info[]");
        let [brand, series, ..] = info.as_slice() else {
            return Err("missing vehicle brand or series".into());
        };
        let (brand, series) = (label(brand, self.names), label(series, self.names));
        if brand.is_empty() {
            // SCS's own trailers have no brand and abbreviated series strings ("STD"):
            // name them from the game id instead (`scs.box` -> "SCS Box").
            let (_, id_series) = self.vehicle.split_once('.').unwrap_or(("", self.vehicle));
            let words = id_series.split('_').map(|w| {
                let mut chars = w.chars();
                chars.next().map_or_else(String::new, |c| {
                    c.to_uppercase().chain(chars).collect::<String>()
                })
            });
            let readable = words.collect::<Vec<_>>().join(" ");
            return Ok(("SCS".to_owned(), format!("SCS {readable}")));
        }
        let mut name = if series.starts_with(&format!("{brand} ")) {
            series.clone()
        } else {
            format!("{brand} {series}")
        };
        // The optional short series tells apart trucks sharing a series (Iveco Stralis Hi-Way).
        if let Some(short) = info.get(4).filter(|s| !s.is_empty()) {
            let short = label(short, self.names);
            if !series.contains(&short) {
                name = format!("{name} {short}");
            }
        }
        Ok((brand, name))
    }

    /// Whether a trailer has any paint mask, main or per accessory.
    fn paintable(&mut self) -> Result<bool, String> {
        let paths = self.definition_paths();
        Ok(!self.paint_overrides(&paths)?.is_empty() || self.has_main_mask(&paths)?)
    }

    /// Whether a main paint job (`paint_job/*.sii`) assigns a mask.
    fn has_main_mask(&mut self, paths: &[String]) -> Result<bool, String> {
        let prefix = format!("{}paint_job/", self.root);
        for path in paths.iter().filter(|p| {
            p.strip_prefix(&prefix)
                .is_some_and(|rest| !rest.contains('/'))
        }) {
            let data = self.definition(path)?;
            if units(&data).iter().any(|u| {
                u.header.starts_with("accessory_paint_job_data :")
                    && u.fields.contains_key("paint_job_mask")
            }) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn paint_overrides(&mut self, paths: &[String]) -> Result<Masks, String> {
        let prefix = format!("{}paint_job/accessory/", self.root);
        let mut masks = Masks::new();
        for path in paths.iter().filter(|p| p.starts_with(&prefix)) {
            let data = self.definition(path)?;
            for unit in units(&data)
                .iter()
                .filter(|u| u.header.starts_with("simple_paint_job_data :"))
            {
                // Flake-only overrides also list accessories, but assign no independent texture.
                if !unit.fields.contains_key("paint_job_mask") {
                    continue;
                }
                let mask = unit.field("paint_job_mask")?;
                for id in array(unit, "acc_list[]") {
                    masks
                        .entry(id)
                        .or_default()
                        .entry(mask.to_owned())
                        .or_default()
                        .insert(path.clone());
                }
            }
        }
        Ok(masks)
    }

    fn unit_geometry(
        &mut self,
        unit: &Section,
        path: &str,
    ) -> Result<(Vec<geometry::Mesh>, Vec<Value>), String> {
        let (mut mesh, mut selections) = (Vec::new(), Vec::new());
        for [model, look, variant] in model_selections(unit, path)? {
            // The descriptor is the model's recorded source.
            self.evidence.insert(model.clone());
            self.stage = "geometry-selection";
            // Always the ordinary paint UVs, as the game's colour paint jobs and the
            // community templates use; `.altuv` materials also carry them.
            let (found, selection) =
                model_geometry(self.sources, &model, &look, &variant, Some(false))
                    // A missing file keeps its message: the report lists it as a dependency.
                    .map_err(|e| {
                        if e.starts_with("missing game file ") {
                            e
                        } else {
                            format!("{path}: {e}")
                        }
                    })?;
            mesh.extend(found);
            selections.push(selection);
        }
        Ok((mesh, selections))
    }

    fn painted_parts(&mut self, paths: &[String]) -> Result<Parts, String> {
        let root = self.root.clone();
        let mut parts = Parts {
            painted: BTreeMap::new(),
            cabins: BTreeMap::new(),
            seen: BTreeSet::new(),
            modelless: BTreeSet::new(),
        };
        let groups = self.kind.painted_groups();
        for path in paths.iter().filter(|p| {
            groups
                .iter()
                .any(|group| p.starts_with(&format!("{root}{group}")))
        }) {
            let data = self.definition(path)?;
            for unit in units(&data)
                .iter()
                .filter(|u| u.header.starts_with("accessory_"))
            {
                let (_, unit_id) = unit
                    .header
                    .split_once(':')
                    .ok_or("invalid accessory unit")?;
                let Some((unit_name, group)) =
                    unit_id.trim().split_once(&format!(".{}.", self.vehicle))
                else {
                    continue;
                };
                let id = format!("{group}.{unit_name}");
                if !parts.seen.insert(id.clone()) {
                    return Err(format!("duplicate accessory unit {id}"));
                }
                let cabin = group == "cabin";
                if cabin {
                    let name = unit.field("name").map(|n| label(n, self.names));
                    parts.cabins.insert(
                        unit_name.to_owned(),
                        name.unwrap_or_else(|_| unit_name.to_owned()),
                    );
                }
                let (mesh, selection) = self.unit_geometry(unit, path)?;
                if selection.is_empty() {
                    parts.modelless.insert(id.clone());
                }
                if mesh.is_empty() {
                    continue;
                }
                let suffix = format!(".{}.cabin", self.vehicle);
                let cabins = array(unit, "suitable_for[]")
                    .into_iter()
                    .filter_map(|s| s.strip_suffix(&suffix).map(str::to_owned))
                    .collect();
                parts.painted.insert(
                    id.clone(),
                    Painted {
                        id,
                        group: group.to_owned(),
                        name: unit_name.to_owned(),
                        cabins,
                        mesh,
                        selection,
                        source: path.clone(),
                    },
                );
            }
        }
        Ok(parts)
    }

    /// Texture size of every accessory with its own sized paint mask.
    fn accessory_sizes(&mut self, parts: &Parts, masks: &Masks) -> BTreeMap<String, [u32; 2]> {
        let provenance = |sources: &Sources, id: &str| {
            sources.provenance(&masks[id].values().flatten().cloned().collect())
        };
        let uncovered = masks
            .keys()
            .filter(|id| !parts.painted.contains_key(*id))
            .collect::<Vec<_>>();
        for id in &uncovered {
            // Paint DLC can reference accessories absent from the installed definitions.
            // A defined accessory with no model field (Krone Box Liner's container body),
            // or whose model has no truckpaint material (a base-colour-only paint flavor),
            // shows no livery whatever its override says. An unreadable model already
            // refused the vehicle while reading geometry.
            let (kind, disposition) = if parts.modelless.contains(*id) {
                ("accessory_without_model", "excluded")
            } else if parts.seen.contains(*id) {
                ("surface_without_livery_material", "excluded")
            } else {
                ("accessory_definition", "ignored_reference")
            };
            self.diagnostics.missing_dependencies.push(json!({
                "kind": kind,
                "disposition": disposition,
                "reference": id,
                "referenced_by": provenance(self.sources, id),
                "expected_dlc": null,
                "dlc_origin": "unknown",
            }));
        }
        let mut sizes = BTreeMap::new();
        for id in masks.keys().filter(|id| parts.painted.contains_key(*id)) {
            let found = masks[id]
                .keys()
                .filter_map(|m| crop_size(m))
                .collect::<BTreeSet<_>>();
            // Paint UVs are normalized, so any size fits: the largest declared one keeps
            // the most detail, and sides below MIN_SIDE are scaled up keeping the aspect
            // ratio (Scania 2016's 32×32 mudflap). Without a sized mask (only shared
            // fills), no size is inferred: the accessory stays on the main texture, as
            // in paint jobs that do not override it.
            let selected = found
                .iter()
                .max_by_key(|[w, h]| u64::from(*w) * u64::from(*h))
                .map(|&size| {
                    let factor = tp_vehicles::MIN_SIDE.div_ceil(size[0].min(size[1]).max(1));
                    (size, size.map(|side| side.saturating_mul(factor.max(1))))
                });
            let (reason, disposition) = match selected {
                None => ("missing_crop_dimensions", "main_texture"),
                Some(_) if found.len() > 1 => ("conflicting_crop_dimensions", "resolved_largest"),
                Some((declared, size)) if declared != size => {
                    ("below_minimum_size", "raised_to_minimum")
                }
                Some(_) => ("", ""),
            };
            if let Some((_, size)) = selected {
                sizes.insert(id.clone(), size);
            }
            if disposition.is_empty() {
                continue;
            }
            self.diagnostics.dimensions.push(json!({
                "reference": id,
                "reason": reason,
                "disposition": disposition,
                "selected": selected.map(|(_, size)| size),
                "candidates": found,
                "masks": masks[id].iter().map(|(mask, paths)| json!({"path": mask, "crop_size": crop_size(mask), "referenced_by": self.sources.provenance(paths)})).collect::<Vec<_>>(),
            }));
        }
        sizes
    }

    fn textures(
        kind: Kind,
        parts: &Parts,
        sizes: &BTreeMap<String, [u32; 2]>,
        masks: &Masks,
        previous: Option<&tp_vehicles::Package>,
    ) -> Result<Textures, String> {
        let groups = accessory_groups(kind, sizes, masks);
        let mut out = Textures::default();
        let mut part_ids = BTreeSet::new();
        let mut covered = BTreeSet::new();
        let mut add = |out: &mut Textures, id: &str, size: [u32; 2], rendered: (Vec<u8>, usize)| {
            if !part_ids.insert(id.to_owned()) {
                return Err(format!("colliding part id {id}"));
            }
            let (svg, edges) = rendered;
            // A changed template raises its layout version, so the app warns players who update.
            let found = previous.and_then(|p| {
                let (_, part) = p
                    .manifest
                    .paint_job
                    .parts()
                    .find(|(_, part)| part.id == id)?;
                Some((part.texture.layout_version, p.template(id)?.bytes == svg))
            });
            let layout_version = match found {
                Some((version, true)) => version,
                Some((version, false)) => {
                    out.changed = true;
                    version + 1
                }
                None => {
                    out.changed |= previous.is_some();
                    1
                }
            };
            let template = format!("templates/{id}.svg");
            out.templates.insert(template.clone(), svg);
            Ok((
                json!({"size": size, "template": template, "layout_version": layout_version}),
                edges,
            ))
        };
        // (main texture id, name, game ids, contributors): one per cabin for a truck,
        // a single one for a trailer.
        let mut mains = Vec::new();
        if kind == Kind::Trailer {
            let all = parts
                .painted
                .values()
                .filter(|p| !sizes.contains_key(&p.id));
            mains.push((
                "trailer".to_owned(),
                "Trailer".to_owned(),
                Vec::new(),
                all.collect(),
            ));
        }
        // The game gives some cabins the same name (8x4 versions): add the game id to tell them apart.
        let shared = |name: &String| parts.cabins.values().filter(|n| *n == name).count() > 1;
        for (cabin, cabin_name) in &parts.cabins {
            let cabin_name = if shared(cabin_name) {
                format!("{cabin_name} ({cabin})")
            } else {
                cabin_name.clone()
            };
            if !parts.painted.contains_key(&format!("cabin.{cabin}")) {
                return Err(format!("cabin has no painted geometry: {cabin}"));
            }
            let contributors = parts
                .painted
                .values()
                .filter(|p| {
                    !sizes.contains_key(&p.id)
                        && (p.group != "cabin" || p.name == *cabin)
                        && (p.cabins.is_empty() || p.cabins.contains(cabin))
                })
                .collect::<Vec<_>>();
            mains.push((cabin.clone(), cabin_name, vec![cabin.clone()], contributors));
        }
        for (id, name, game_ids, contributors) in mains {
            let meshes = contributors
                .iter()
                .flat_map(|p| p.mesh.clone())
                .collect::<Vec<_>>();
            // A trailer whose parts all have their own region keeps a plain main
            // texture for its base colour, as the game's fill mask does.
            let (size, rendered) = if kind == Kind::Trailer && meshes.is_empty() {
                ([64, 64], (geometry::blank_svg([64, 64]), 0))
            } else {
                ([4096, 4096], geometry::svg(&meshes, [4096, 4096])?)
            };
            let (texture, edges) = add(&mut out, &id, size, rendered)?;
            out.main
                .push(json!({"id": id, "name": name, "game_ids": game_ids, "texture": texture}));
            covered.extend(contributors.iter().map(|p| p.id.clone()));
            out.coverage.push(json!({"part": id, "size": size, "edges": edges, "contributors": contributors.iter().map(|p| json!({"id": p.id, "definition": p.source, "selections": p.selection})).collect::<Vec<_>>()}));
        }
        for group in groups {
            let ids = group
                .iter()
                .map(|id| &parts.painted[id])
                .collect::<Vec<_>>();
            // Grouped ids share their masks, hence their size.
            let size = sizes[&group[0]];
            let part_id = group[0].replace('.', "_");
            let meshes = ids.iter().flat_map(|p| p.mesh.clone()).collect::<Vec<_>>();
            let (texture, edges) = add(&mut out, &part_id, size, geometry::svg(&meshes, size)?)?;
            out.accessories.push(json!({"id": part_id, "name": group.join(", "), "game_ids": group, "texture": texture}));
            covered.extend(group.iter().cloned());
            let mut coverage = json!({"part": part_id, "size": size, "edges": edges, "contributors": ids.iter().map(|p| json!({"id": p.id, "definition": p.source, "selections": p.selection})).collect::<Vec<_>>()});
            if group.len() > 1 {
                coverage["shared_masks"] = json!(masks[&group[0]].keys().collect::<Vec<_>>());
            }
            out.coverage.push(coverage);
        }
        if let Some(p) = previous {
            out.changed |= p
                .manifest
                .paint_job
                .parts()
                .any(|(_, part)| !part_ids.contains(&part.id));
        }
        if let Some(id) = parts.painted.keys().find(|id| !covered.contains(*id)) {
            return Err(format!(
                "painted definition has no compatible texture: {id}"
            ));
        }
        Ok(out)
    }

    fn package(&mut self, info: &GameInfo) -> Result<(Outcome, Value), String> {
        let (brand, name) = self.identity()?;
        let paths = self.definition_paths();
        let masks = self.paint_overrides(&paths)?;
        let parts = self.painted_parts(&paths)?;
        self.stage = "coverage";
        if self.kind == Kind::Truck && parts.cabins.is_empty() {
            return Err("truck has no cabins".into());
        }
        if self.kind == Kind::Trailer && masks.is_empty() && !self.has_main_mask(&paths)? {
            return Err(NOT_PAINTABLE.into());
        }
        let sizes = self.accessory_sizes(&parts, &masks);
        let textures = Self::textures(self.kind, &parts, &sizes, &masks, self.previous)?;
        let game = info.semver()?;
        let next = game.minor.checked_add(1).ok_or("invalid game version")?;
        let mut version = semver::Version::new(game.major, game.minor, game.patch);
        if let Some(previous) = self.previous {
            if !textures.changed && previous.manifest.game.versions.matches(&version) {
                return Ok((Outcome::UpToDate, json!(textures.coverage)));
            }
            // An installed version is never replaced by a different package.
            let installed = &previous.manifest.version;
            if *installed >= version {
                version =
                    semver::Version::new(installed.major, installed.minor, installed.patch + 1);
            }
        }
        let vehicle = self.vehicle;
        let manifest = json!({"format":1,"id":format!("official.{}.{vehicle}", info.game.code()),"version":version,"name":name,"brand":brand,"kind":self.kind.manifest(),"description":format!("Generated from ETS2 {} on this computer.", info.version),"authors":["SCS Software","TruckPaint contributors"],"license":LICENSE,"game":{"id":info.game.code(),"path":vehicle,"versions":format!(">={}.{}, <{}.{next}", game.major, game.minor, game.major),"alt_uv":false,"colour_picker":false},"paint_job":{"main":textures.main,"accessories":textures.accessories}});
        self.stage = "package-validation";
        let packed =
            crate::pack_entries(manifest, textures.templates).map_err(|e| e.to_string())?;
        Ok((Outcome::Package(Box::new(packed)), json!(textures.coverage)))
    }
}

/// The newest valid package of each id in a previously published batch.
fn previous_packages(dir: &Path) -> Result<BTreeMap<String, tp_vehicles::Package>, String> {
    let mut packages = BTreeMap::<String, tp_vehicles::Package>::new();
    for path in source::files(dir)? {
        if path.extension().is_none_or(|e| e != tp_vehicles::EXTENSION) {
            continue;
        }
        let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let package =
            tp_vehicles::Package::read(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        let newer = packages
            .get(&package.manifest.id)
            .is_none_or(|known| known.manifest.version < package.manifest.version);
        if newer {
            packages.insert(package.manifest.id.clone(), package);
        }
    }
    Ok(packages)
}

/// The game a folder holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameInfo {
    pub game: tp_vehicles::Game,
    /// The game's own version, four numbers (`1.61.1.9`).
    pub version: String,
}

impl GameInfo {
    /// The first three numbers: the version of the packages made from this game.
    pub fn semver(&self) -> Result<semver::Version, String> {
        let numbers = self
            .version
            .split('.')
            .take(3)
            .map(|n| n.parse::<u64>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| format!("invalid game version {}", self.version))?;
        match numbers[..] {
            [major, minor, patch] => Ok(semver::Version::new(major, minor, patch)),
            _ => Err(format!("invalid game version {}", self.version)),
        }
    }
}

/// Why a folder is not a usable game folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InspectError {
    NotAFolder,
    /// A main archive that is not there (`base.scs`).
    Missing(String),
    /// The archives are there but hold no ETS2 or ATS version.
    NotAGame,
    Unreadable(String),
}

impl std::fmt::Display for InspectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAFolder => write!(f, "not a folder"),
            Self::Missing(file) => write!(f, "missing {file}"),
            Self::NotAGame => write!(f, "not an ETS2 or ATS folder"),
            Self::Unreadable(cause) => write!(f, "unreadable: {cause}"),
        }
    }
}

/// The main archives a game folder must hold.
const MAIN_ARCHIVES: [&str; 3] = ["base.scs", "def.scs", "version.scs"];

fn game_info(text: &str) -> Result<GameInfo, String> {
    let version = syntax::parse(text)?;
    let version = units(&version)
        .iter()
        .find(|s| s.header.starts_with("fs_pack_set :"))
        .ok_or("missing game version unit")?;
    let game = match version.field("application")? {
        "eut2" => tp_vehicles::Game::Ets2,
        "ats" => tp_vehicles::Game::Ats,
        other => return Err(format!("unknown game {other}")),
    };
    Ok(GameInfo {
        game,
        version: version.field("version")?.to_owned(),
    })
}

/// Checks that `folder` is a game folder and reads its game and version, from
/// the main archives only.
pub fn inspect(folder: &Path) -> Result<GameInfo, InspectError> {
    if !folder.is_dir() {
        return Err(InspectError::NotAFolder);
    }
    let archives = MAIN_ARCHIVES.map(|name| folder.join(name));
    for (name, path) in MAIN_ARCHIVES.iter().zip(&archives) {
        if !path.is_file() {
            return Err(InspectError::Missing((*name).to_owned()));
        }
    }
    let mount = tp_scs::Mount::open(&archives).map_err(InspectError::Unreadable)?;
    let text = mount
        .read("/version.sii")
        .map_err(InspectError::Unreadable)?
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .ok_or(InspectError::NotAGame)?;
    game_info(&text).map_err(|_| InspectError::NotAGame)
}

/// An official vehicle of the game that can be painted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vehicle {
    /// The game's id (`man.tgx_2020`); the package id is `official.<game>.<id>`.
    pub id: String,
    pub kind: tp_vehicles::Kind,
    pub name: String,
    pub brand: String,
}

/// A game folder read: its vehicles, and the opened archives (file tables only)
/// that [`generate`] reads, shared read-only between threads.
pub struct Scan {
    pub info: GameInfo,
    pub vehicles: Vec<Vehicle>,
    sources: Sources,
    names: Names,
    /// Game id -> the kinds defining it; more than one is a collision, refused when generating.
    discovered: BTreeMap<String, BTreeSet<Kind>>,
}

/// Lists the official trucks and owned trailers of an ETS2 folder, without writing anything.
pub fn scan(folder: &Path) -> Result<Scan, String> {
    let mut sources = Sources::new(folder)?;
    let kinds = [Kind::Truck, Kind::Trailer];
    let folders = kinds.map(|k| format!("/def/vehicle/{}", k.folder()));
    sources.inventory(&folders.each_ref().map(String::as_str))?;
    let info = game_info(&sources.read("/version.sii")?)?;
    if info.game != tp_vehicles::Game::Ets2 {
        return Err("only ETS2 installations are supported".into());
    }
    let names = names(&sources)?;
    let mut discovered = BTreeMap::<String, BTreeSet<Kind>>::new();
    for kind in kinds {
        let prefix = format!("/def/vehicle/{}/", kind.folder());
        for id in sources
            .paths_under(&prefix)
            .iter()
            .filter_map(|p| p.strip_prefix(&prefix)?.strip_suffix("/data.sii"))
            .filter(|p| !p.contains('/') && tp_vehicles::is_valid_game_name(p))
        {
            discovered.entry(id.to_owned()).or_default().insert(kind);
        }
    }
    if discovered.is_empty() {
        return Err("no official ETS2 vehicles discovered".into());
    }
    let mut vehicles = Vec::new();
    for (id, found) in &discovered {
        let [kind] = found.iter().copied().collect::<Vec<_>>()[..] else {
            continue;
        };
        let mut build = Build::new(&sources, &names, None, kind, id);
        // A vehicle whose data cannot be read is still listed: generating it says why.
        let (brand, name) = build
            .identity()
            .unwrap_or_else(|_| (String::new(), id.clone()));
        if kind == Kind::Trailer && !build.paintable()? {
            continue;
        }
        vehicles.push(Vehicle {
            id: id.clone(),
            kind: kind.public(),
            name,
            brand,
        });
    }
    // Trucks first, then trailers, each in id order.
    vehicles.sort_by_key(|v| (v.kind == tp_vehicles::Kind::Trailer, v.id.clone()));
    Ok(Scan {
        info,
        vehicles,
        sources,
        names,
        discovered,
    })
}

/// What generating a vehicle gives.
pub enum Outcome {
    Package(Box<crate::Packed>),
    /// No template changed and the previous package already covers the game's version.
    UpToDate,
}

/// One vehicle's build, with what the report needs after a failure.
struct Attempt {
    kind: Kind,
    result: Result<(Outcome, Value), String>,
    evidence: BTreeSet<String>,
    stage: &'static str,
    diagnostics: Diagnostics,
}

fn attempt(
    scan: &Scan,
    vehicle: &str,
    previous: Option<&tp_vehicles::Package>,
) -> Result<Attempt, String> {
    let found = scan
        .discovered
        .get(vehicle)
        .ok_or_else(|| format!("unknown or excluded vehicle {vehicle}"))?;
    let [kind] = found.iter().copied().collect::<Vec<_>>()[..] else {
        return Err("defined both as a truck and as an owned trailer".into());
    };
    let mut build = Build::new(&scan.sources, &scan.names, previous, kind, vehicle);
    let result = build.package(&scan.info);
    let Build {
        evidence,
        stage,
        diagnostics,
        ..
    } = build;
    Ok(Attempt {
        kind,
        result,
        evidence,
        stage,
        diagnostics,
    })
}

/// Builds one vehicle's package in memory. `previous` is its newest installed
/// package: unchanged templates keep their layout version, changed ones get the
/// next, and the version is raised above the installed one when needed.
pub fn generate(
    scan: &Scan,
    vehicle: &str,
    previous: Option<&tp_vehicles::Package>,
) -> Result<Outcome, String> {
    attempt(scan, vehicle, previous)?
        .result
        .map(|(outcome, _)| outcome)
}

/// Extracts a batch, continuing after per-vehicle refusals.
pub fn run(options: &Options) -> Result<Batch, String> {
    let preliminary = reserve(Path::new("target/game-extraction"))?;
    let scan = scan(&options.installation)?;
    let (sources, game_version) = (&scan.sources, &scan.info.version);
    let previous = match &options.previous {
        Some(dir) => previous_packages(dir)?,
        None => BTreeMap::new(),
    };
    let discovered = &scan.discovered;
    let mut vehicles = if let Some(vehicle) = &options.vehicle {
        if !discovered.contains_key(vehicle) {
            return Err(format!("unknown or excluded vehicle {vehicle}"));
        }
        vec![vehicle.clone()]
    } else {
        discovered.keys().cloned().collect()
    };
    // Trucks first, then trailers, each in id order.
    vehicles.sort_by_key(|v| (discovered[v].first().copied(), v.clone()));
    let output = reserve(&options.output)?;
    let extract = |vehicle: &String| -> Value {
        eprintln!("extracting {vehicle}");
        let previous_package = previous.get(&format!("official.ets2.{vehicle}"));
        let Attempt {
            kind,
            result,
            evidence,
            stage,
            mut diagnostics,
        } = match attempt(&scan, vehicle, previous_package) {
            Ok(attempt) => attempt,
            Err(error) => {
                return json!({"vehicle":vehicle,"kinds":discovered[vehicle].iter().map(|k| k.manifest()).collect::<Vec<_>>(),"status":"failure","stage":"discovery","error":error});
            }
        };
        let mut stage = stage;
        let result = result.and_then(|(outcome, coverage)| match outcome {
            Outcome::Package(packed) => {
                let filename = packed.file_name();
                match publish(&output.join(&filename), &packed.bytes) {
                    Ok(()) => Ok((Some(filename), coverage)),
                    Err(e) => {
                        stage = "publication";
                        Err(e)
                    }
                }
            }
            Outcome::UpToDate => Ok((None, coverage)),
        });
        match result {
            Ok((Some(filename), coverage)) => {
                json!({"vehicle":vehicle,"kind":kind.manifest(),"status":"success","package":filename,"technical_validation":"passed","in_game_validation":"required","missing_dependencies":diagnostics.missing_dependencies,"dimension_diagnostics":diagnostics.dimensions,"textures":coverage,"sources":sources.provenance(&evidence)})
            }
            Ok((None, _)) => {
                json!({"vehicle":vehicle,"kind":kind.manifest(),"status":"skipped","reason":"up_to_date","sources":sources.provenance(&evidence)})
            }
            Err(error) if error == NOT_PAINTABLE => {
                json!({"vehicle":vehicle,"kind":kind.manifest(),"status":"skipped","reason":"not_paintable","error":error,"sources":sources.provenance(&evidence)})
            }
            Err(error) => {
                if let Some(path) = error.strip_prefix("missing game file ") {
                    diagnostics.missing_dependencies.push(json!({"kind":"game_file","disposition":"blocking","reference":path,"expected_dlc":null,"dlc_origin":"unknown"}));
                }
                json!({"vehicle":vehicle,"kind":kind.manifest(),"status":"failure","stage":stage,"error":error,"missing_dependencies":diagnostics.missing_dependencies,"dimension_diagnostics":diagnostics.dimensions,"sources":sources.provenance(&evidence)})
            }
        }
    };
    // Half the cores, at most 8: concurrent builds must leave the desktop memory and CPU.
    const MAX_WORKERS: usize = 8;
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
    let workers = (cores / 2).min(MAX_WORKERS).clamp(1, vehicles.len().max(1));
    let next = AtomicUsize::new(0);
    let done = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            let (extract, next, done, vehicles) = (&extract, &next, &done, &vehicles);
            scope.spawn(move || {
                loop {
                    let position = next.fetch_add(1, Ordering::Relaxed);
                    let Some(vehicle) = vehicles.get(position) else {
                        break;
                    };
                    let result = extract(vehicle);
                    done.lock().unwrap().push((position, result));
                }
            });
        }
    });
    // Report entries keep the sequential order, whatever order workers finished in.
    let mut done = done.into_inner().map_err(|_| "a vehicle worker panicked")?;
    done.sort_by_key(|(position, _)| *position);
    let results = done
        .into_iter()
        .map(|(_, result)| result)
        .collect::<Vec<_>>();
    let failed = results.iter().filter(|r| r["status"] == "failure").count();
    let report = json!({"format":1,"installation":options.installation,"reader":{"name":tp_scs::READER,"version":tp_scs::VERSION},"tool_version":env!("CARGO_PKG_VERSION"),"game_version":game_version,"archives":sources.archives,"archive_precedence":"last mounted wins; base source order and sorted DLC as observed in ETS2 mount log","requested_vehicle":options.vehicle,"previous_batch":options.previous,"discovered_vehicles":discovered.keys().collect::<Vec<_>>(),"processed_vehicles":vehicles,"name_sources":NAME_FILES.iter().map(|p| sources.provenance(&BTreeSet::from([(*p).to_owned()]))[0].clone()).collect::<Vec<_>>(),"work_directory":preliminary,"work_directory_kept":failed > 0,"results":results,"failed":failed,"limits":["Truckpaint, tsnmap and tsnmapuv exterior models with optional airbrush/flipflake/altuv flavors and a/asafew.a alpha-test suffixes","All paint materials use the ordinary paint UVs (_TEXCOORD1), including altuv flavors; packages never set alternate_uvset","Finite UV overflow is preserved in SVG coordinates; the texture viewport clips the wireframe without wrapping or changing UV mode; out-of-bounds surfaces need game checks","Main templates union compatible painted options","Accessories without a truckpaint material in their selected look/variant are excluded from templates","Cabin size 4096 square; accessory dimensions from crop names; conflicting sizes use the largest, accessories with only unsized (shared fill) masks stay on the main texture","Package validation does not establish correctness in game"]});
    publish(
        &output.join("report.json"),
        &serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )?;
    // The folder of a failed batch stays for the failed vehicles' intermediates.
    if failed == 0
        && let Err(e) = std::fs::remove_dir_all(&preliminary)
    {
        eprintln!("could not remove {}: {e}", preliminary.display());
    }
    Ok(Batch {
        directory: output,
        failed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reservations_are_unique_and_publication_leaves_no_partial_file() {
        let temp = tempfile::tempdir().unwrap();
        let first = reserve(temp.path()).unwrap();
        let second = reserve(temp.path()).unwrap();
        assert_ne!(first, second);
        publish(&first.join("a.tpv"), b"package").unwrap();
        assert_eq!(std::fs::read(first.join("a.tpv")).unwrap(), b"package");
        // A failed write (missing directory) publishes nothing.
        assert!(publish(&first.join("missing/b.tpv"), b"package").is_err());
        let names = std::fs::read_dir(&first)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect::<Vec<_>>();
        assert_eq!(names, ["a.tpv"]);
    }

    use crate::synthetic::{Vehicle as Made, write_game};
    use tp_vehicles::{Game, Kind as VehicleKind};

    fn game(application: &str, version: &str, vehicles: &[Made]) -> tempfile::TempDir {
        let temp = tempfile::tempdir().unwrap();
        write_game(temp.path(), application, version, vehicles);
        temp
    }

    #[test]
    fn inspect_reads_the_game_from_the_main_archives_and_says_what_is_wrong() {
        let ets2 = game("eut2", "1.61.1.9", &[]);
        let info = inspect(ets2.path()).unwrap();
        assert_eq!((info.game, info.version.as_str()), (Game::Ets2, "1.61.1.9"));
        let ats = game("ats", "1.55.0.1", &[]);
        assert_eq!(inspect(ats.path()).unwrap().game, Game::Ats);
        std::fs::remove_file(ets2.path().join("base.scs")).unwrap();
        assert_eq!(
            inspect(ets2.path()),
            Err(InspectError::Missing("base.scs".into()))
        );
        assert_eq!(
            inspect(&ets2.path().join("nothing")),
            Err(InspectError::NotAFolder)
        );
        std::fs::write(ets2.path().join("base.scs"), b"not an archive").unwrap();
        assert!(matches!(
            inspect(ets2.path()),
            Err(InspectError::Unreadable(_))
        ));
        let other = game("other", "1.0.0.0", &[]);
        assert_eq!(inspect(other.path()), Err(InspectError::NotAGame));
    }

    #[test]
    fn scan_lists_owned_paintable_vehicles_only() {
        let vehicles = [
            Made::truck("example.alpha", "Alpha"),
            Made::trailer("example.box", "Box"),
            Made::trailer("example.flat", "Flat").in_archive("dlc_owned"),
            Made {
                paintable: false,
                ..Made::trailer("example.plain", "Plain")
            },
        ];
        let owned = game("eut2", "1.61.1.9", &vehicles);
        let scan = scan(owned.path()).unwrap();
        let listed = scan
            .vehicles
            .iter()
            .map(|v| (v.id.as_str(), v.kind, v.name.as_str(), v.brand.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(
            listed,
            [
                (
                    "example.alpha",
                    VehicleKind::Truck,
                    "Example Alpha",
                    "Example"
                ),
                (
                    "example.box",
                    VehicleKind::Trailer,
                    "Example Box",
                    "Example"
                ),
                (
                    "example.flat",
                    VehicleKind::Trailer,
                    "Example Flat",
                    "Example"
                ),
            ]
        );
        // The same game without the DLC does not list its trailer.
        let base = game("eut2", "1.61.1.9", &vehicles[..2]);
        assert_eq!(scan_ids(base.path()), ["example.alpha", "example.box"]);
        let ats = game("ats", "1.55.0.1", &vehicles[..1]);
        assert!(super::scan(ats.path()).err().unwrap().contains("only ETS2"));
    }

    fn scan_ids(folder: &Path) -> Vec<String> {
        scan(folder)
            .unwrap()
            .vehicles
            .into_iter()
            .map(|v| v.id)
            .collect()
    }

    fn package(outcome: Outcome) -> crate::Packed {
        match outcome {
            Outcome::Package(packed) => *packed,
            Outcome::UpToDate => panic!("expected a package"),
        }
    }

    fn generated(folder: &Path, previous: Option<&tp_vehicles::Package>) -> Outcome {
        generate(&scan(folder).unwrap(), "example.alpha", previous).unwrap()
    }

    #[test]
    fn versions_follow_the_game_and_never_replace_an_installed_package() {
        let alpha = [Made::truck("example.alpha", "Alpha")];
        let first = game("eut2", "1.61.1.9", &alpha);
        let packed = package(generated(first.path(), None));
        let manifest = &packed.manifest;
        assert_eq!(manifest.version.to_string(), "1.61.1");
        assert_eq!(manifest.game.versions.to_string(), ">=1.61, <1.62");
        assert_eq!(manifest.id, "official.ets2.example.alpha");
        assert_eq!(manifest.license.as_deref(), Some(LICENSE));
        assert!(manifest.homepage.is_none());
        assert!(
            manifest
                .description
                .as_deref()
                .unwrap()
                .contains("1.61.1.9 on this computer")
        );
        crate::check(&packed.bytes).unwrap();
        let installed = tp_vehicles::Package::read(&packed.bytes).unwrap();

        // Same templates, same game: nothing to install.
        assert!(matches!(
            generated(first.path(), Some(&installed)),
            Outcome::UpToDate
        ));
        // A new minor series with the same templates: a new version, same layout versions.
        let next = game("eut2", "1.62.0.3", &alpha);
        let moved = package(generated(next.path(), Some(&installed)));
        assert_eq!(moved.manifest.version.to_string(), "1.62.0");
        assert_eq!(moved.manifest.paint_job, installed.manifest.paint_job);
        // A changed template under the installed version: the next patch and layout version.
        let patched = game(
            "eut2",
            "1.61.1.4",
            &[Made {
                shifted: true,
                ..Made::truck("example.alpha", "Alpha")
            }],
        );
        let changed = package(generated(patched.path(), Some(&installed)));
        assert_eq!(changed.manifest.version.to_string(), "1.61.2");
        assert_eq!(changed.manifest.paint_job.main[0].texture.layout_version, 2);
        assert!(generate(&scan(first.path()).unwrap(), "nothing", None).is_err());
    }
}
