//! What the Vehicle Library needs to know about a project without opening
//! it: its vehicles and, per texture, the template and how many objects
//! an update would ask to reposition.

use tp_core::Project;
use tp_core::kurbo::Size;

use crate::current::{FileKind, FileObject, FileProject};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Summary {
    pub name: String,
    pub vehicles: Vec<SummaryVehicle>,
    pub textures: Vec<SummaryTexture>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SummaryVehicle {
    pub package_id: String,
    /// The package version the project records.
    pub version: String,
}

/// A texture painted from a vehicle package.
#[derive(Clone, Debug, PartialEq)]
pub struct SummaryTexture {
    pub package_id: String,
    pub texture_id: String,
    pub layout_version: u32,
    pub size: Size,
    /// See [`tp_core::objects_on`].
    pub objects: usize,
}

impl Summary {
    /// The summary of a project in memory, unsaved edits included.
    pub fn of(project: &Project) -> Self {
        Self {
            name: project.name.clone(),
            vehicles: project
                .vehicles
                .iter()
                .map(|v| SummaryVehicle {
                    package_id: v.package_id.clone(),
                    version: v.version.clone(),
                })
                .collect(),
            textures: project
                .surfaces
                .iter()
                .filter_map(|s| {
                    let t = s.template.as_ref()?;
                    Some(SummaryTexture {
                        package_id: t.package_id.clone(),
                        texture_id: t.texture_id.clone(),
                        layout_version: t.layout_version,
                        size: s.size,
                        objects: tp_core::objects_on(s).len(),
                    })
                })
                .collect(),
        }
    }

    /// The summary of a project file's document, counting as
    /// [`tp_core::objects_on`] does.
    pub(crate) fn of_file(file: &FileProject) -> Self {
        fn count(objects: &[FileObject]) -> usize {
            objects
                .iter()
                .map(|o| match o.kind {
                    FileKind::Group => count(&o.children),
                    _ => 1,
                })
                .sum()
        }
        Self {
            name: file.name.clone(),
            vehicles: file
                .vehicles
                .iter()
                .map(|v| SummaryVehicle {
                    package_id: v.package_id.clone(),
                    version: v.version.clone(),
                })
                .collect(),
            textures: file
                .surfaces
                .iter()
                .filter_map(|s| {
                    let t = s.template.as_ref()?;
                    Some(SummaryTexture {
                        package_id: t.package_id.clone(),
                        texture_id: t.texture_id.clone(),
                        layout_version: t.layout_version,
                        size: Size::new(s.size[0], s.size[1]),
                        objects: count(&s.objects),
                    })
                })
                .collect(),
        }
    }
}
