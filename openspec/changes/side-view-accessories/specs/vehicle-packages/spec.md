## MODIFIED Requirements

### Requirement: Preview mesh
A main texture MAY name, in an optional `preview_mesh` field, a JSON file inside the package that holds a simple model of the vehicle's painted surfaces for that texture, used only by the Side View (see the side-view capability). The file SHALL contain:

- `format`: the preview mesh format version (1);
- `pieces`: one or more pieces, each with:
  - `texture`: the id of the package part whose texture paints the piece;
  - `positions`: one `[x, y, z]` per vertex, in meters, with x towards the vehicle's right, y up and z towards its back, as in the game;
  - `uv`: one `[u, v]` per vertex in the texture's coordinates, `[0, 0]` at the top left corner and `[1, 1]` at the bottom right, as the template is drawn;
  - `triangles`: `[a, b, c]` vertex indices;
- optionally `locators`: the attachment points of the cabin (or of a trailer's root), by name, each a list of `{position, rotation, scale}` with `position` in the same axes, `rotation` a unit quaternion `[w, x, y, z]` and `scale` a number;
- optionally `slots`: the accessory slots, in display order, each with:
  - `id`: the slot's game name (`cab_door`, `r_deflector`, `chassis`, `body`…);
  - `parent`: `cabin` or `chassis`, the model whose locator of the same name as the slot places it; the `chassis` slot itself has no parent;
  - `options`: one or more options, each with an `id`, a `name`, `default` (true for the option the game fits on a new vehicle, at most one per slot), `pieces` (as above, in the option's own model space) and, for the chassis slot's options, the chassis's `locators`.

A piece of an option SHALL be drawn at every locator of the slot's name of its parent: its position is the locator's position plus the locator's rotation and scale applied to the piece's position. A `pieces` entry outside the slots is always drawn.

Validation SHALL refuse the package, naming the file, when a named preview mesh is missing or unreadable, when a piece names a texture the package does not have, when `positions` and `uv` have different lengths, when an index is out of range, when a number is not finite, when a slot or an option id is repeated, when a slot has more than one default, or when a slot's parent is `chassis` and the mesh has no `chassis` slot. A package without any preview mesh SHALL stay valid. Pieces MAY name any texture of the package, main or accessory. Preview meshes count towards the package's size limit. A project SHALL NOT embed a preview mesh: it is read from the installed package.

#### Scenario: Package with a preview mesh
- **WHEN** a truck package's main texture "Standard cab" names `preview/standard_cab.json`, holding one piece painted by "Standard cab"
- **THEN** the package is accepted and the Side View can draw that texture

#### Scenario: Package without a preview mesh
- **WHEN** a package's main textures name no preview mesh
- **THEN** it is accepted as before, and the Side View says it is not available for that vehicle

#### Scenario: Broken preview mesh
- **WHEN** a preview mesh has a triangle with the index 120 and only 100 vertices
- **THEN** the package is refused and the message names the preview mesh file

#### Scenario: Project saved and shared
- **WHEN** a project whose vehicle has preview meshes is saved
- **THEN** the `.truckpaint` file contains the templates it embeds and no preview mesh

#### Scenario: Door placed at its locator
- **WHEN** a preview mesh's slot `cab_door` (parent `cabin`) has an option whose pieces span x from −2.32 to 0.17, and the cabin's locator `cab_door` is at x = 1.075 with no rotation
- **THEN** the door is drawn from x = −1.245 to 1.245

#### Scenario: Two defaults
- **WHEN** a slot has two options with `default: true`
- **THEN** the package is refused and the message names the preview mesh file and the slot
