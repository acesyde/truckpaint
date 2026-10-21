//! The menus' structure as data: the menu bar draws it, and the command
//! palette reads each command's menu path from it, so the two can't
//! disagree.

use tp_core::document::{BooleanOp, DistributeAxis, DistributeMode, Edge, FlipAxis};

use crate::commands::{CommandId, EditContext};
use crate::layout::{LeftTab, Space, WorkspaceLayout};
use crate::prefs::ViewAids;
use crate::tool::Tool;

/// A row of a menu.
#[derive(Clone, Debug, PartialEq)]
pub enum Entry {
    Item(CommandId),
    /// An item with a check mark showing its state ([`is_checked`]).
    Toggle(CommandId),
    Separator,
    Submenu {
        /// Message id of its title.
        title: &'static str,
        min_width: f32,
        entries: Vec<Entry>,
    },
}

/// A menu of the menu bar.
#[derive(Clone, Debug, PartialEq)]
pub struct Menu {
    /// Message id of its title.
    pub title: &'static str,
    pub entries: Vec<Entry>,
}

/// Minimum width of a menu.
pub const MENU_MIN_WIDTH: f32 = 220.0;

/// Fallback groups of the commands no menu shows: their message id and
/// the commands, in order.
fn groups() -> Vec<(&'static str, Vec<CommandId>)> {
    use CommandId::*;
    vec![
        (
            "palette-group-tools",
            Tool::ALL.into_iter().map(SelectTool).collect(),
        ),
        (
            "palette-group-colors",
            vec![SwapColorTarget, SwapFillStroke, DefaultColors],
        ),
        ("palette-group-symbol", vec![FinishSymbol]),
    ]
}

/// The menus drawn in the window, in order (File, Edit, Object, Layer,
/// View, Vehicle, Help). The design system gallery is in debug builds
/// only.
pub fn menus() -> Vec<Menu> {
    use CommandId::*;
    use Entry::{Item, Separator, Toggle};
    let menu = |title, entries| Menu { title, entries };
    let mut view = vec![Item(CommandPalette), Separator];
    view.extend(Space::ALL.map(|s| Toggle(ShowSpace(s))));
    view.push(Separator);
    view.extend(LeftTab::ALL.map(|t| Toggle(ShowLeftTab(t))));
    view.extend([
        Separator,
        Toggle(TogglePanels),
        Separator,
        Toggle(ShowTemplate),
        Toggle(ShowGrid),
        Toggle(ShowGuides),
        Item(ClearGuides),
        Toggle(Snapping),
        Separator,
        Toggle(ShowSideView),
        Separator,
        Item(ZoomIn),
        Item(ZoomOut),
        Item(FitToScreen),
        Item(ActualSize),
        Separator,
        Item(ResetWorkspace),
    ]);
    if cfg!(debug_assertions) {
        view.extend([Separator, Item(DesignGallery)]);
    }

    let mut align: Vec<Entry> = Edge::ALL.into_iter().map(|e| Item(Align(e))).collect();
    align.push(Separator);
    align.extend(
        [
            (DistributeAxis::Horizontal, DistributeMode::Centers),
            (DistributeAxis::Vertical, DistributeMode::Centers),
            (DistributeAxis::Horizontal, DistributeMode::Spacing),
            (DistributeAxis::Vertical, DistributeMode::Spacing),
        ]
        .map(|(axis, mode)| Item(Distribute(axis, mode))),
    );
    let mut object = vec![
        Item(EditText),
        Separator,
        Item(Group),
        Item(Ungroup),
        Separator,
        Item(ConvertToSymbol),
        Item(ImportFromLibrary),
        Item(EditSymbol),
        Item(DetachInstance),
        Separator,
        Item(ConvertToPath),
        Item(CreateOutlines),
        Entry::Submenu {
            title: "menu-combine",
            min_width: MENU_MIN_WIDTH,
            entries: BooleanOp::ALL
                .into_iter()
                .map(|o| Item(Combine(o)))
                .collect(),
        },
        Entry::Submenu {
            title: "menu-align",
            min_width: 260.0,
            entries: align,
        },
        Separator,
    ];
    object.extend(FlipAxis::ALL.map(|a| Item(Flip(a))));
    object.extend([Separator, Item(BringForward), Item(SendBackward)]);

    vec![
        menu(
            "menu-file",
            vec![
                Item(NewProject),
                Item(OpenProject),
                Separator,
                Item(Save),
                Item(SaveAs),
                Separator,
                Item(Place),
                Separator,
                Item(CloseProject),
                Separator,
                Item(ExportTexture),
                Item(ExportMod),
                Separator,
                Item(Quit),
            ],
        ),
        menu(
            "menu-edit",
            vec![
                Item(Undo),
                Item(Redo),
                Separator,
                Item(Cut),
                Item(Copy),
                Item(Paste),
                Item(Duplicate),
                Item(Delete),
                Separator,
                Item(SelectAll),
                Item(Deselect),
                Separator,
                Item(Preferences),
            ],
        ),
        menu("menu-object", object),
        menu(
            "menu-layer",
            vec![Item(NewLayer), Item(DuplicateLayer), Item(DeleteLayer)],
        ),
        menu("menu-view", view),
        menu(
            "menu-vehicle",
            vec![
                Item(VehicleLibrary),
                Item(AddVehicle),
                Separator,
                Item(NextTexture),
                Item(PreviousTexture),
                Item(CopyFromCabin),
                Separator,
                Item(UpdateTemplate),
            ],
        ),
        menu(
            "menu-help",
            vec![
                Item(KeyboardShortcuts),
                Item(RunFirstLaunch),
                Separator,
                Item(About),
            ],
        ),
    ]
}

/// The macOS application menu (titled TruckPaint): About, Preferences and
/// Quit, which [`menus_for`] takes out of Help, Edit and File.
pub fn app_menu() -> Menu {
    use CommandId::*;
    use Entry::{Item, Separator};
    Menu {
        title: "menu-app",
        entries: vec![
            Item(About),
            Separator,
            Item(Preferences),
            Separator,
            Item(Quit),
        ],
    }
}

/// The menus of the platform: [`menus`] elsewhere; on macOS (`mac`) the
/// application menu first, and About, Preferences and Quit only there.
pub fn menus_for(mac: bool) -> Vec<Menu> {
    let mut menus = menus();
    if !mac {
        return menus;
    }
    let app = app_menu();
    let moved = |entry: &Entry| match entry {
        Entry::Item(id) => app.entries.contains(&Entry::Item(*id)),
        _ => false,
    };
    for menu in &mut menus {
        // Each moved item goes with the separator before it.
        while let Some(at) = menu.entries.iter().position(moved) {
            menu.entries.remove(at);
            if at > 0 && menu.entries.get(at - 1) == Some(&Entry::Separator) {
                menu.entries.remove(at - 1);
            }
        }
    }
    menus.insert(0, app);
    menus
}

/// Whether toggle `id` is on: the space shown, the left tab, Hide Panels,
/// the template, the grid, the guides, snapping and the Side View. `layout` is `None` on
/// the home screen, where every toggle is off.
pub fn is_checked(
    id: CommandId,
    edit: &EditContext,
    layout: Option<&WorkspaceLayout>,
    aids: ViewAids,
) -> bool {
    let open = layout.is_some();
    match id {
        CommandId::ShowSpace(s) => edit.space == Some(s),
        CommandId::ShowLeftTab(t) => layout.is_some_and(|l| l.left_tab == t),
        CommandId::TogglePanels => layout.is_some_and(|l| l.panels_hidden),
        CommandId::ShowTemplate => edit.template_visible,
        CommandId::ShowGrid => open && aids.grid,
        CommandId::ShowGuides => open && aids.guides,
        CommandId::Snapping => open && aids.snapping,
        CommandId::ShowSideView => layout.is_some_and(|l| l.side_view),
        _ => false,
    }
}

/// A command the palette lists, with its context.
#[derive(Clone, Debug, PartialEq)]
pub struct CatalogEntry {
    pub id: CommandId,
    /// Message ids of its menu and submenu titles, or of its group.
    pub path: Vec<&'static str>,
    /// It is a toggle of its menu (shows a check mark when on).
    pub toggle: bool,
}

/// Every command the palette lists, in menu order with its menu path (the
/// platform's menus, [`menus_for`]), then the commands outside the menus
/// with their group. A command shown twice keeps its first path.
pub fn catalog() -> Vec<CatalogEntry> {
    catalog_for(cfg!(target_os = "macos"))
}

/// [`catalog`] with the menus of macOS (`mac`) or of the other platforms.
pub fn catalog_for(mac: bool) -> Vec<CatalogEntry> {
    fn walk(entries: &[Entry], path: &mut Vec<&'static str>, out: &mut Vec<CatalogEntry>) {
        for entry in entries {
            match entry {
                Entry::Item(id) | Entry::Toggle(id) => {
                    if id.in_palette() && !out.iter().any(|e| e.id == *id) {
                        out.push(CatalogEntry {
                            id: *id,
                            path: path.clone(),
                            toggle: matches!(entry, Entry::Toggle(_)),
                        });
                    }
                }
                Entry::Separator => {}
                Entry::Submenu { title, entries, .. } => {
                    path.push(title);
                    walk(entries, path, out);
                    path.pop();
                }
            }
        }
    }
    let mut out = Vec::new();
    for menu in menus_for(mac) {
        let mut path = vec![menu.title];
        walk(&menu.entries, &mut path, &mut out);
    }
    for (group, ids) in groups() {
        for id in ids {
            if !out.iter().any(|e| e.id == id) {
                out.push(CatalogEntry {
                    id,
                    path: vec![group],
                    toggle: false,
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn menu_commands(entries: &[Entry], out: &mut Vec<CommandId>) {
        for entry in entries {
            match entry {
                Entry::Item(id) | Entry::Toggle(id) => out.push(*id),
                Entry::Separator => {}
                Entry::Submenu { entries, .. } => menu_commands(entries, out),
            }
        }
    }

    #[test]
    fn every_paletted_command_is_in_the_catalog_once() {
        for mac in [false, true] {
            let catalog = catalog_for(mac);
            for id in CommandId::all() {
                let count = catalog.iter().filter(|e| e.id == id).count();
                if id == CommandId::DesignGallery && !cfg!(debug_assertions) {
                    assert_eq!(count, 0, "{id:?}");
                } else if id.in_palette() {
                    assert_eq!(count, 1, "{id:?} (mac: {mac})");
                } else {
                    assert_eq!(count, 0, "{id:?}");
                }
            }
        }
    }

    fn commands_of(menu: &Menu) -> Vec<CommandId> {
        let mut out = Vec::new();
        menu_commands(&menu.entries, &mut out);
        out
    }

    fn find<'a>(menus: &'a [Menu], title: &str) -> &'a Menu {
        menus.iter().find(|m| m.title == title).unwrap()
    }

    #[test]
    fn export_items_end_the_file_menu() {
        use CommandId::*;
        let menus = menus();
        assert!(menus.iter().all(|m| m.title != "menu-export"));
        let file = &find(&menus, "menu-file").entries;
        assert_eq!(
            file[file.len() - 5..],
            [
                Entry::Separator,
                Entry::Item(ExportTexture),
                Entry::Item(ExportMod),
                Entry::Separator,
                Entry::Item(Quit),
            ]
        );
    }

    #[test]
    fn the_macos_app_menu_holds_about_preferences_and_quit() {
        use CommandId::*;
        let mac = menus_for(true);
        assert_eq!(mac[0], app_menu());
        assert_eq!(commands_of(&mac[0]), [About, Preferences, Quit]);
        assert!(!commands_of(find(&mac, "menu-file")).contains(&Quit));
        assert!(!commands_of(find(&mac, "menu-edit")).contains(&Preferences));
        assert!(!commands_of(find(&mac, "menu-help")).contains(&About));
        // No separator is left at the end of a menu.
        for menu in &mac {
            assert_ne!(
                menu.entries.last(),
                Some(&Entry::Separator),
                "{}",
                menu.title
            );
        }
        assert_eq!(
            mac[1..].iter().map(|m| m.title).collect::<Vec<_>>(),
            crate::ui::menu_bar::MENUS
        );
        assert_eq!(menus_for(false), menus());
        let catalog = catalog_for(true);
        let path = |id| catalog.iter().find(|e| e.id == id).unwrap().path.clone();
        assert_eq!(path(Preferences), ["menu-app"]);
        assert_eq!(path(ExportMod), ["menu-file"]);
        assert_eq!(
            catalog_for(false)
                .iter()
                .find(|e| e.id == Preferences)
                .unwrap()
                .path,
            ["menu-edit"]
        );
    }

    #[test]
    fn every_menu_command_is_registered() {
        let all = CommandId::all();
        let mut listed = Vec::new();
        for menu in menus() {
            menu_commands(&menu.entries, &mut listed);
        }
        for id in listed {
            assert!(all.contains(&id), "{id:?}");
        }
    }

    #[test]
    fn paths_name_the_menu_and_submenu_or_the_group() {
        let catalog = catalog();
        let path = |id| {
            catalog
                .iter()
                .find(|e| e.id == id)
                .map(|e| e.path.clone())
                .unwrap()
        };
        assert_eq!(
            path(CommandId::Align(Edge::Left)),
            ["menu-object", "menu-align"]
        );
        assert_eq!(path(CommandId::Flip(FlipAxis::Horizontal)), ["menu-object"]);
        assert_eq!(
            path(CommandId::SelectTool(Tool::Ellipse)),
            ["palette-group-tools"]
        );
        assert_eq!(path(CommandId::DefaultColors), ["palette-group-colors"]);
        assert_eq!(path(CommandId::FinishSymbol), ["palette-group-symbol"]);
        assert!(
            catalog
                .iter()
                .all(|e| !matches!(e.id, CommandId::Nudge(..) | CommandId::CommandPalette))
        );
        assert_eq!(
            catalog.iter().any(|e| e.id == CommandId::DesignGallery),
            cfg!(debug_assertions)
        );
    }

    #[test]
    fn menu_titles_follow_the_menu_bar() {
        let titles: Vec<&str> = menus().iter().map(|m| m.title).collect();
        assert_eq!(titles, crate::ui::menu_bar::MENUS);
    }

    #[test]
    fn view_starts_with_the_palette() {
        let view = menus()
            .into_iter()
            .find(|m| m.title == "menu-view")
            .unwrap();
        assert_eq!(
            view.entries[..2],
            [Entry::Item(CommandId::CommandPalette), Entry::Separator]
        );
    }

    #[test]
    fn toggles_follow_the_state() {
        let layout = WorkspaceLayout {
            left_tab: LeftTab::Layers,
            ..WorkspaceLayout::default()
        };
        let edit = EditContext {
            has_project: true,
            space: Some(Space::Workshop),
            ..EditContext::default()
        };
        let aids = ViewAids {
            grid: true,
            ..ViewAids::default()
        };
        let on = |id| is_checked(id, &edit, Some(&layout), aids);
        assert!(on(CommandId::ShowSpace(Space::Workshop)));
        assert!(!on(CommandId::ShowSpace(Space::Brand)));
        assert!(on(CommandId::ShowLeftTab(LeftTab::Layers)));
        assert!(on(CommandId::ShowGrid));
        assert!(!on(CommandId::TogglePanels));
        assert!(!is_checked(
            CommandId::ShowGrid,
            &EditContext::default(),
            None,
            aids
        ));
    }
}
