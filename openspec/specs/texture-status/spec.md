# texture-status Specification

## Purpose
Tells the player, for each texture of the fleet, what is left to do before exporting the mod: whether it is still empty, painted, or flagged by a template update, how many of its colors are off the brand palette, and what the export will warn about.

## Requirements

### Requirement: Texture states
Each texture of a project SHALL have one of three states, computed from the project whenever it is shown and never stored or typed by the player:
- **To check:** an update flagged the texture, "Layout changed" or "Not in this version" (see the vehicle-projects capability). To check SHALL win over the other two states;
- **Empty:** otherwise, when nothing of the texture is drawn: it holds no object, or every object is hidden. An object is drawn when it is visible and, for a group or a symbol instance, when at least one of its children is drawn. An Empty texture is exported transparent, so the game's base color shows;
- **Modified:** otherwise: at least one object is drawn.

The state SHALL follow every change at once (drawing, deleting, hiding or showing an object, undo and redo, Update Template, Mark as Checked). Opacity, locking and the template's visibility SHALL NOT change the state. The To check flag is the one Update Template already records; the objects flagged with it are stored beside it and never change a texture's state (see the template-update-impact capability).

A state SHALL never be shown by color alone: wherever it is shown, it also has its label, an icon or a distinct shape, and its label in the name given to assistive technologies.

#### Scenario: New project
- **WHEN** the user creates a project from the sample truck with both main textures and every accessory
- **THEN** its five textures are Empty

#### Scenario: Drawing makes it Modified
- **WHEN** the user draws a rectangle on the Empty Chassis texture
- **THEN** Chassis is Modified, and Undo makes it Empty again

#### Scenario: Every object hidden
- **WHEN** the Side skirts texture holds a rectangle and a group whose children are all hidden, and the user hides the rectangle
- **THEN** Side skirts is Empty

#### Scenario: Flagged by an update
- **WHEN** Update Template flags the Standard cab texture "Layout changed", and Standard cab holds a logo
- **THEN** Standard cab is To check, not Modified

#### Scenario: Not in this version
- **WHEN** an update marks the Curtain body 10.5 m texture "Not in this version", and it holds no object
- **THEN** Curtain body 10.5 m is To check, not Empty

### Requirement: Mark as Checked
A texture flagged "Layout changed" SHALL be cleared by **Mark as Checked**, offered with the flag in the inspector when nothing is selected (see the properties-panel capability). Mark as Checked SHALL clear the flag as one undo step, "Mark as Checked"; the texture then takes the state its objects give it. Undo SHALL flag it again.

Opening, viewing or editing the texture SHALL NOT clear the flag.

A texture marked "Not in this version" SHALL NOT be offered Mark as Checked: it stays To check until it is removed from the project (it is left out of the mod, see the mod-export capability).

#### Scenario: Marking a texture as checked
- **WHEN** the Standard cab texture, flagged "Layout changed" and holding a logo, is active with nothing selected, and the user clicks Mark as Checked in the inspector
- **THEN** Standard cab is Modified, the undo history's last step is "Mark as Checked", and Undo makes it To check again

#### Scenario: Opening doesn't clear it
- **WHEN** the user clicks the To check texture Curtain body 13.6 m in the Project space, draws on it and shows the Project space again
- **THEN** Curtain body 13.6 m is still To check

#### Scenario: Not in this version stays
- **WHEN** the active texture is marked "Not in this version" and nothing is selected
- **THEN** the inspector offers no Mark as Checked

### Requirement: Off-palette colors
The **off-palette colors** of a texture SHALL be the distinct colors (compared by their exact RGBA value) used by its own objects that aren't linked to a palette swatch:
- the solid fill of a shape or a text whose fill isn't linked to a swatch;
- the solid stroke of a shape or a text whose stroke isn't linked to a swatch;
- each stop of a gradient fill or stroke that isn't linked to a swatch.

Only drawn objects count (see Texture states), inside groups too. Symbol instances SHALL NOT be looked into: their colors belong to the symbol. Images and groups have no colors of their own. A fully transparent color SHALL NOT count. A color equal to a swatch's value but not linked to it SHALL count.

The **objects using them** SHALL be the drawn objects, inside groups too, holding at least one off-palette color.

#### Scenario: Linked colors don't count
- **WHEN** a texture holds a rectangle filled with the swatch "Company red" (linked) and stroked with an unlinked black, and a text filled with an unlinked white
- **THEN** it has 2 off-palette colors, used by the rectangle and the text

#### Scenario: Same value, not linked
- **WHEN** the palette has "Company red" `#C23B2A`, and a texture holds an ellipse filled with `#C23B2A` picked in the color picker, not from the swatch
- **THEN** the texture has 1 off-palette color

#### Scenario: Gradient stops
- **WHEN** a texture holds one rectangle with a linear gradient whose first stop is linked to "Company red" and whose second stop is an unlinked `#151515`
- **THEN** the texture has 1 off-palette color

#### Scenario: Distinct colors
- **WHEN** a texture holds three rectangles filled with the same unlinked `#151515`
- **THEN** it has 1 off-palette color, used by the three rectangles

#### Scenario: Instances are left out
- **WHEN** a texture holds only an instance of a symbol "Logo" whose content is filled with unlinked colors
- **THEN** the texture has no off-palette color

#### Scenario: Hidden objects are left out
- **WHEN** a texture's only unlinked color is on a hidden rectangle
- **THEN** the texture has no off-palette color

### Requirement: Before exporting warnings
The project SHALL have a list of **warnings** about its textures, shown under "Before exporting" in the Project space (see the project-screen capability) and among the checks of Export Mod… (see the mod-export capability), in this order:
- one line per To check texture, in project order: "<texture>: layout changed", or "<texture>: not in this version, left out of the mod";
- one line for the Empty textures, when there are any: "<texture> is empty, exported with the game's color" for one, "N textures empty, exported with the game's color" for several.

A texture is named by its name, preceded by its vehicle ("<vehicle> › <texture>") when another texture of the project has the same name.

Warnings SHALL NOT block the export. They SHALL be listed apart from the problems that block it, with their own icon and wording, so they can't be mistaken for them.

When the project has no warning, the list SHALL say "Nothing to check".

#### Scenario: Fleet with work left
- **WHEN** a project holds the sample truck with Standard cab and High roof Empty and its accessories Modified, and the sample trailer with Base and Curtain body 10.5 m Empty, Curtain body 13.6 m flagged "Layout changed" and Mudflaps Modified
- **THEN** the warnings read "Curtain body 13.6 m: layout changed", then "4 textures empty, exported with the game's color"

#### Scenario: One empty texture
- **WHEN** every texture of a project is Modified except High roof, which is Empty
- **THEN** the warnings read "High roof is empty, exported with the game's color"

#### Scenario: Same name in two vehicles
- **WHEN** a project holds two trailers each painting a texture named "Base", and the first one's Base is Empty
- **THEN** the Empty line names it "<first trailer's name> › Base"

#### Scenario: Nothing left
- **WHEN** every texture of a project is Modified
- **THEN** the list says "Nothing to check"
