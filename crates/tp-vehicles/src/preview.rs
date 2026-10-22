//! The preview mesh of a main texture: a simple model of the painted surfaces, for the Side View.

use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};

/// Highest preview mesh format this version understands.
pub const PREVIEW_FORMAT: u32 = 1;

/// Id of the slot holding the chassis options.
pub const CHASSIS_SLOT: &str = "chassis";

/// A simple model of a vehicle's painted surfaces, used by the Side View.
/// Positions are in meters (x right, y up, z towards the back), UVs in
/// texture coordinates with `[0, 0]` at the top left.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct PreviewMesh {
    pub format: u32,
    /// Always drawn, as they are.
    pub pieces: Vec<PreviewPiece>,
    /// The attachment points of the cabin (or of a trailer's root), by name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub locators: BTreeMap<String, Vec<PreviewLocator>>,
    /// The accessory slots, in display order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slots: Vec<PreviewSlot>,
}

/// Triangles painted by one texture of the package.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct PreviewPiece {
    /// Id of the package part whose texture paints the piece.
    pub texture: String,
    pub positions: Vec<[f32; 3]>,
    pub uv: Vec<[f32; 2]>,
    pub triangles: Vec<[u32; 3]>,
}

/// An attachment point: where an accessory named like it is mounted.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct PreviewLocator {
    pub position: [f32; 3],
    /// A unit quaternion `[w, x, y, z]`, as stored in the game's models.
    pub rotation: [f32; 4],
    pub scale: f32,
}

/// The model an accessory is mounted on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SlotParent {
    Cabin,
    Chassis,
}

/// One place of the vehicle where a single option is fitted.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct PreviewSlot {
    /// The game's name of the slot (`cab_door`, `r_deflector`, `chassis`, `body`...).
    pub id: String,
    /// Absent for the chassis slot, which is drawn where it is modeled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<SlotParent>,
    pub options: Vec<PreviewOption>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct PreviewOption {
    pub id: String,
    pub name: String,
    /// Whether the game fits it on a new vehicle.
    #[serde(default)]
    pub default: bool,
    /// In the option's own model space.
    pub pieces: Vec<PreviewPiece>,
    /// The attachment points of a chassis option.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub locators: BTreeMap<String, Vec<PreviewLocator>>,
}

impl PreviewLocator {
    pub const IDENTITY: Self = Self {
        position: [0.0; 3],
        rotation: [1.0, 0.0, 0.0, 0.0],
        scale: 1.0,
    };

    /// Where the point `p`, in the model space of an accessory, lies once the accessory is
    /// mounted here. The game stores the inverse of the usual rotation, so the conjugate of the
    /// quaternion is applied: a tool box on the left chassis locator (stored as a 90° yaw) then
    /// sticks out to the left.
    pub fn place(&self, p: [f32; 3]) -> [f32; 3] {
        let [w, x, y, z] = self.rotation;
        let u = [-x, -y, -z];
        let cross = |a: [f32; 3], b: [f32; 3]| {
            [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ]
        };
        let v = p.map(|c| c * self.scale);
        let t = cross(u, v);
        let tt = cross(u, t);
        std::array::from_fn(|i| self.position[i] + v[i] + 2.0 * (w * t[i] + tt[i]))
    }
}

impl PreviewMesh {
    pub fn slot(&self, id: &str) -> Option<&PreviewSlot> {
        self.slots.iter().find(|s| s.id == id)
    }

    /// The locators a slot's option is drawn at: the slot's name in its parent's locators (the
    /// cabin's, or those of `chassis`, the chosen chassis option). The chassis slot itself is
    /// drawn once, as modeled.
    pub fn mounts(
        &self,
        slot: &PreviewSlot,
        chassis: Option<&PreviewOption>,
    ) -> Vec<PreviewLocator> {
        let found = match slot.parent {
            None => return vec![PreviewLocator::IDENTITY],
            Some(SlotParent::Cabin) => self.locators.get(&slot.id),
            Some(SlotParent::Chassis) => chassis.and_then(|c| c.locators.get(&slot.id)),
        };
        found.cloned().unwrap_or_default()
    }

    /// Checks the mesh; `known` tells whether a texture id exists in the package.
    pub(crate) fn validate(&self, known: impl Fn(&str) -> bool) -> Result<(), String> {
        for piece in self.pieces.iter().chain(
            self.slots
                .iter()
                .flat_map(|s| &s.options)
                .flat_map(|o| &o.pieces),
        ) {
            piece.validate(&known)?;
        }
        let locators = self.locators.values().flatten().chain(
            self.slots
                .iter()
                .flat_map(|s| &s.options)
                .flat_map(|o| o.locators.values().flatten()),
        );
        for l in locators {
            let numbers = l.position.iter().chain(&l.rotation).chain([&l.scale]);
            if numbers.into_iter().any(|n| !n.is_finite()) {
                return Err("a number is not finite".into());
            }
        }
        let mut slots = HashSet::new();
        for slot in &self.slots {
            if !slots.insert(&slot.id) {
                return Err(format!("slot \"{}\" is repeated", slot.id));
            }
            let mut options = HashSet::new();
            if let Some(repeated) = slot.options.iter().find(|o| !options.insert(&o.id)) {
                return Err(format!(
                    "option \"{}\" is repeated in slot \"{}\"",
                    repeated.id, slot.id
                ));
            }
            if slot.options.iter().filter(|o| o.default).count() > 1 {
                return Err(format!("slot \"{}\" has more than one default", slot.id));
            }
            let mounted = match slot.parent {
                None => true,
                Some(SlotParent::Cabin) => self.locators.contains_key(&slot.id),
                Some(SlotParent::Chassis) => {
                    let Some(chassis) = self.slot(CHASSIS_SLOT) else {
                        return Err(format!(
                            "slot \"{}\" hangs on the chassis, which the mesh has no slot for",
                            slot.id
                        ));
                    };
                    chassis
                        .options
                        .iter()
                        .any(|o| o.locators.contains_key(&slot.id))
                }
            };
            if !mounted {
                return Err(format!("slot \"{}\" has no locator on its parent", slot.id));
            }
        }
        Ok(())
    }
}

impl PreviewPiece {
    fn validate(&self, known: &impl Fn(&str) -> bool) -> Result<(), String> {
        if !known(&self.texture) {
            return Err(format!("unknown texture \"{}\"", self.texture));
        }
        let vertices = self.positions.len();
        if self.uv.len() != vertices {
            return Err("positions and uv have different lengths".into());
        }
        if self
            .triangles
            .iter()
            .flatten()
            .any(|i| *i as usize >= vertices)
        {
            return Err(format!(
                "a triangle uses a vertex beyond the {vertices} of its piece"
            ));
        }
        let numbers = self
            .positions
            .iter()
            .flatten()
            .chain(self.uv.iter().flatten());
        if numbers.into_iter().any(|n| !n.is_finite()) {
            return Err("a number is not finite".into());
        }
        Ok(())
    }
}
