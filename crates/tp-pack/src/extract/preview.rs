//! The preview mesh of a main texture: the cabin (or a trailer's chassis) and every painted accessory
//! option, each in its own model space, to be placed by the Side View at the locators of its slot.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};
use tp_vehicles::{
    CHASSIS_SLOT, PreviewLocator, PreviewMesh, PreviewOption, PreviewPiece, PreviewSlot, SlotParent,
};

use super::geometry::{self, Locators};
use super::{Painted, Parts};

/// What decides which definitions a main texture's preview shows.
pub struct Scope<'a> {
    pub parts: &'a Parts,
    /// The main texture the pieces not on an accessory texture are painted by.
    pub main: &'a str,
    /// The cabin of a truck's main texture; `None` for a trailer, whose chassis is the root.
    pub cabin: Option<&'a str>,
    /// The part id of the accessory texture of each definition that has one.
    pub textures: &'a BTreeMap<String, String>,
    /// Source files of the definitions the game fits on a new vehicle.
    pub defaults: &'a BTreeSet<String>,
    /// Limits the definitions of a trailer to those it can be bought with.
    pub allowed: &'a dyn Fn(&Painted) -> bool,
}

/// One option while definitions are being merged.
struct Merged<'a> {
    pieces: Vec<PreviewPiece>,
    locators: BTreeMap<String, Vec<PreviewLocator>>,
    members: Vec<&'a Painted>,
}

/// The definitions of `scope.parts` that can be fitted to the cabin of the scope, by slot.
fn candidates<'a>(scope: &Scope<'a>) -> BTreeMap<&'a str, Vec<&'a Painted>> {
    let mut slots = BTreeMap::<&str, Vec<&Painted>>::new();
    for p in scope
        .parts
        .painted
        .values()
        .chain(scope.parts.bare.values())
    {
        let fits = match scope.cabin {
            Some(cabin) => {
                (p.cabins.is_empty() || p.cabins.contains(cabin))
                    && fits(scope.parts, &p.id, cabin, 0)
            }
            None => (scope.allowed)(p),
        };
        if fits && p.group != "cabin" {
            slots.entry(&p.group).or_default().push(p);
        }
    }
    slots
}

/// Whether the definition `id` can be fitted to `cabin`, following what it is suitable for
/// (cabins, interiors, chassis and other accessories).
fn fits(parts: &Parts, id: &str, cabin: &str, depth: usize) -> bool {
    let Some(unit) = parts.units.get(id) else {
        return false;
    };
    unit.suitable.is_empty()
        || unit
            .suitable
            .iter()
            .any(|target| match target.split_once('.') {
                Some(("cabin", name)) => name == cabin,
                _ => depth < 4 && fits(parts, target, cabin, depth + 1),
            })
}

fn merge<'a>(
    definitions: &[&'a Painted],
    scope: &Scope,
    chassis: bool,
    wanted: &BTreeSet<String>,
) -> Vec<Merged<'a>> {
    let mut merged: Vec<Merged> = Vec::new();
    for p in definitions {
        let eu = if p.eu == 0 { p.mesh.len() } else { p.eu };
        let texture = scope.textures.get(&p.id).map_or(scope.main, String::as_str);
        let pieces = geometry::preview_pieces(&p.mesh[..eu], texture);
        // A chassis option is told apart by where it holds its accessories as well.
        let locators = if chassis {
            p.locators
                .iter()
                .filter(|(name, _)| wanted.contains(*name))
                .map(|(name, list)| {
                    (
                        name.clone(),
                        list.iter().map(geometry::preview_locator).collect(),
                    )
                })
                .collect()
        } else {
            BTreeMap::new()
        };
        match merged
            .iter_mut()
            .find(|m| m.pieces == pieces && m.locators == locators)
        {
            Some(same) => same.members.push(p),
            None => merged.push(Merged {
                pieces,
                locators,
                members: vec![p],
            }),
        }
    }
    merged
}

fn options(
    merged: Vec<Merged>,
    defaults: &BTreeSet<String>,
    names: &Parts,
) -> (Vec<PreviewOption>, Vec<Value>) {
    let mut options = Vec::new();
    let mut report = Vec::new();
    let mut has_default = false;
    for m in merged {
        let reached = m.members.iter().position(|p| defaults.contains(&p.source));
        let default = reached.is_some() && !has_default;
        has_default |= default;
        // A merged option is named after the default definition when there is one.
        let shown = m.members[reached.unwrap_or(0)];
        let name = names
            .units
            .get(&shown.id)
            .map_or(shown.name.clone(), |u| u.name.clone());
        report.push(json!({"id": shown.name, "name": name, "default": default, "definitions": m.members.iter().map(|p| &p.id).collect::<Vec<_>>()}));
        options.push(PreviewOption {
            id: shown.name.clone(),
            name,
            default,
            pieces: m.pieces,
            locators: m.locators,
        });
    }
    (options, report)
}

/// The preview mesh of the scope and its report entry; `None` when there is nothing to draw.
pub fn build(scope: &Scope) -> Result<Option<(PreviewMesh, Value)>, String> {
    let by_slot = candidates(scope);
    let mut mesh = PreviewMesh {
        format: tp_vehicles::PREVIEW_FORMAT,
        ..PreviewMesh::default()
    };
    let mut left_out = Vec::new();
    let mut cabin_locators = Locators::new();
    if let Some(cabin) = scope.cabin {
        let p = scope
            .parts
            .painted
            .get(&format!("cabin.{cabin}"))
            .ok_or_else(|| format!("cabin has no painted geometry: {cabin}"))?;
        let eu = if p.eu == 0 { p.mesh.len() } else { p.eu };
        mesh.pieces = geometry::preview_pieces(&p.mesh[..eu], scope.main);
        cabin_locators.clone_from(&p.locators);
    }
    // The chassis: the slot every other one may hang on.
    let chassis_defs = by_slot.get(CHASSIS_SLOT).cloned().unwrap_or_default();
    let others = by_slot
        .iter()
        .filter(|(slot, _)| **slot != CHASSIS_SLOT)
        .collect::<Vec<_>>();
    // A slot hangs on the cabin when the cabin has a locator of its name, otherwise on the chassis.
    let on_cabin = |slot: &str| scope.cabin.is_some() && cabin_locators.contains_key(slot);
    let on_chassis = |slot: &str| chassis_defs.iter().any(|c| c.locators.contains_key(slot));
    let chassis_slots = others
        .iter()
        .map(|(slot, _)| **slot)
        .filter(|slot| !on_cabin(slot) && on_chassis(slot))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let mut slots = Vec::new();
    let mut report = Vec::new();
    if !chassis_defs.is_empty() {
        let (list, listed) = options(
            merge(&chassis_defs, scope, true, &chassis_slots),
            scope.defaults,
            scope.parts,
        );
        report.push(json!({"id": CHASSIS_SLOT, "options": listed}));
        slots.push(PreviewSlot {
            id: CHASSIS_SLOT.to_owned(),
            parent: None,
            options: list,
        });
    }
    for (slot, definitions) in others {
        let parent = if on_cabin(slot) {
            SlotParent::Cabin
        } else if chassis_slots.contains(*slot) {
            SlotParent::Chassis
        } else {
            left_out.push(json!({"slot": slot, "reason": "no locator of this name on the cabin or the suitable chassis", "definitions": definitions.iter().map(|p| &p.id).collect::<Vec<_>>()}));
            continue;
        };
        if parent == SlotParent::Cabin {
            let list = cabin_locators[*slot].iter().map(geometry::preview_locator);
            mesh.locators.insert((*slot).to_owned(), list.collect());
        }
        let (list, listed) = options(
            merge(definitions, scope, false, &BTreeSet::new()),
            scope.defaults,
            scope.parts,
        );
        report.push(json!({"id": slot, "parent": parent, "options": listed}));
        slots.push(PreviewSlot {
            id: (*slot).to_owned(),
            parent: Some(parent),
            options: list,
        });
    }
    // Unpainted accessories are not read as models, but the report names them by slot.
    let unpainted = scope
        .parts
        .units
        .iter()
        .filter(|(id, _)| {
            let group = id.split_once('.').map_or("", |(group, _)| group);
            !matches!(group, "cabin" | "chassis" | "interior" | "body")
                && !scope.parts.painted.contains_key(*id)
                && scope
                    .cabin
                    .is_none_or(|cabin| fits(scope.parts, id, cabin, 0))
        })
        .fold(
            BTreeMap::<&str, Vec<&String>>::new(),
            |mut slots, (id, _)| {
                slots
                    .entry(id.split_once('.').map_or("", |(g, _)| g))
                    .or_default()
                    .push(id);
                slots
            },
        );
    for (slot, definitions) in unpainted {
        left_out
            .push(json!({"slot": slot, "reason": "no painted piece", "definitions": definitions}));
    }
    mesh.slots = slots;
    if mesh.pieces.is_empty() && mesh.slots.is_empty() {
        return Ok(None);
    }
    let entry = json!({"configuration": scope.cabin.map(|c| format!("cabin.{c}")), "pieces": mesh.pieces.len(), "slots": report, "left_out": left_out});
    Ok(Some((mesh, entry)))
}
