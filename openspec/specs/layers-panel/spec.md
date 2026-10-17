# layers-panel Specification

## Purpose

Shows the object tree of the active surface in the Workshop's Layers tab and lets users organize it like in Photoshop or Illustrator: select, rename, hide, lock, reorder, group and ungroup.

## Requirements

### Requirement: Tree view
The Layers panel SHALL be the **Layers** tab of the Workshop's left panel (key 2, see the workspace-layout capability). The tab SHALL have a header reading "<texture> · N layers", where N is the number of top-level objects of the active surface, with an **add** button that runs New Layer. Below it, the tab SHALL list the active surface's objects as a tree, topmost first, with groups expandable and collapsible and their children indented. Each row SHALL show its kind icon on a tile, the name, its kind in small type (left out when the name is the kind's), a visibility toggle and a lock toggle. A name is the kind's when it is the kind's default name in any of the supported languages ("Text", "Texte", "Texto"), so that a document made in one language reads the same in another. A text layer whose name is the kind's SHALL show the first non-empty line of its text instead, with its kind in small type. A name too long for its row SHALL end with "…" and show in full in a tooltip, never be cut mid-letter; names are in the secondary text color and in ink when selected; hidden and locked states SHALL be shown by icon shape and by dimming the name, not only by color. An empty surface SHALL show an empty state in the list, under the header.

#### Scenario: Group shown as a tree
- **WHEN** the surface contains a group "Branding" with "Logo" and "Company name", above a rectangle "Background"
- **THEN** the tab shows "Branding" first with its two children indented below it, then "Background"

#### Scenario: Tab header
- **WHEN** the active texture "Side skirts" has five top-level objects and the user opens the Layers tab
- **THEN** the header reads "Side skirts · 5 layers", and clicking its add button adds an empty group "Layer N" at the top of the tree

#### Scenario: Text layers named by their text
- **WHEN** the surface holds two texts named "Text", reading "ACE LOGISTICS" and "Since 1984"
- **THEN** their rows read "ACE LOGISTICS" and "Since 1984", each with the kind "Text" in small type

#### Scenario: Default name from another language
- **WHEN** a rectangle was named "Rectangle" in English and the language is now German
- **THEN** its row reads "Rectangle" once, without the kind "Rechteck" next to it

#### Scenario: Long layer name
- **WHEN** a layer is named "Grey stripe along the lower edge of the cabin" and the panel is at its default width
- **THEN** its row shows the start of the name ending with "…", and resting the pointer on it shows the whole name

### Requirement: Selection sync
Clicking a row SHALL select that object (including objects inside groups); Cmd/Ctrl+click SHALL toggle it in the selection; Shift+click SHALL select the range of visible rows from the last clicked row. Selected rows SHALL be highlighted with a fill and an accent bar. Selecting on the canvas SHALL highlight the corresponding rows and expand collapsed ancestors so they are visible.

#### Scenario: Row selects canvas object
- **WHEN** the user clicks the "Logo" row inside the "Branding" group
- **THEN** only the Logo object is selected on the canvas with its handles

### Requirement: Rename
Double-clicking a name or choosing Rename from the context menu SHALL edit the name inline; Enter commits, Escape cancels; empty names revert to the default name for the kind ("Rectangle", "Ellipse", "Group").

#### Scenario: Rename a layer
- **WHEN** the user double-clicks "Rectangle", types "Burgundy base" and presses Enter
- **THEN** the row and the inspector header show "Burgundy base"

### Requirement: Visibility and lock
The eye toggle SHALL hide or show the object (and all its children for a group); hidden objects are not drawn, not hit on the canvas and not exported later, but stay in the tree. The lock toggle SHALL prevent the object (and its children) from being selected or modified on the canvas; locked objects remain selectable from the panel only for unlocking. Hiding or locking a selected object SHALL remove it from the selection.

#### Scenario: Hide an object
- **WHEN** the user clicks the eye of "White stripe"
- **THEN** the stripe disappears from the canvas, its row is dimmed with a closed-eye icon, and clicking where it was does not select it

#### Scenario: Locked object ignores canvas clicks
- **WHEN** "Background" is locked and the user clicks on it on the canvas
- **THEN** it is not selected

### Requirement: Drag and drop
Dragging rows SHALL reorder objects: a drop indicator line shows the target position between rows, and dropping onto the middle of a group row (highlighted) SHALL move the dragged objects into that group at its top. Dragging several selected rows SHALL move them together, keeping their relative order. A group SHALL NOT be dropped into itself or its descendants.

#### Scenario: Move an object into a group
- **WHEN** the user drags "Truck number" onto the middle of the "Branding" group row
- **THEN** "Truck number" becomes the topmost child of "Branding"

#### Scenario: Reorder between rows
- **WHEN** the user drags "Grey stripe" above "White stripe" inside "Graphics"
- **THEN** "Grey stripe" is drawn above "White stripe"

### Requirement: Group and ungroup
Group (Cmd/Ctrl+G) SHALL wrap the selected objects in a new group named "Group" placed at the position of the topmost selected object, keeping their stacking order, and select the group. Ungroup (Cmd/Ctrl+Shift+G) SHALL replace each selected group by its children at the same position and select those children. New Layer (Cmd/Ctrl+Shift+N) SHALL add an empty top-level group named "Layer N" at the top and select it; Duplicate Layer and Delete Layer act like Duplicate and Delete on the selected rows. These commands SHALL also be available from the Layers tab's context menu and footer buttons, and New Layer from the add button of its header.

#### Scenario: Group then ungroup
- **WHEN** two rectangles are selected and the user presses Cmd/Ctrl+G and then Cmd/Ctrl+Shift+G
- **THEN** after grouping, one group containing both is selected; after ungrouping, the two rectangles are back at the same stacking position and selected

#### Scenario: New layer
- **WHEN** the user presses Cmd/Ctrl+Shift+N in a project without groups
- **THEN** an empty group "Layer 1" appears at the top of the tree and is selected

### Requirement: Objects to reposition
In the Layers tab, an object that a template update flagged (see the template-update-impact capability) SHALL be marked in its row with an icon and the label "To reposition" in its name for assistive technologies, never by color alone; resting the pointer on the mark SHALL say that the texture's layout changed in the version. A group SHALL show the mark when any object inside it is flagged. The header of the tab SHALL NOT change. A mark SHALL disappear when its object is moved or transformed, or when the texture is marked as checked, and SHALL come back with Undo.

#### Scenario: Flagged object marked
- **WHEN** Update Template flagged "Standard cab" "Layout changed" while it held the logo "Ace logo" and the user opens the Layers tab on it
- **THEN** the row "Ace logo" shows the "To reposition" mark, and so does every other object that was on the texture

#### Scenario: Moving clears one mark
- **WHEN** the user moves "Ace logo" on that texture
- **THEN** its mark disappears and the others stay; Undo brings it back

#### Scenario: Group
- **WHEN** a group holds a flagged logo and the group is collapsed
- **THEN** the group's row shows the mark

#### Scenario: Object drawn after the update
- **WHEN** the user draws a rectangle on the flagged texture after the update
- **THEN** its row has no mark
