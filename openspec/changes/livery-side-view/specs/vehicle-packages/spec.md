## ADDED Requirements

### Requirement: Preview mesh
A main texture MAY name, in an optional `preview_mesh` field, a JSON file inside the package that holds a simple model of the vehicle's painted surfaces for that texture, used only by the Side View (see the side-view capability). The file SHALL contain:

- `format`: the preview mesh format version (1);
- `pieces`: one or more pieces, each with:
  - `texture`: the id of the package part whose texture paints the piece;
  - `positions`: one `[x, y, z]` per vertex, in meters, with x towards the vehicle's right, y up and z towards its back, as in the game;
  - `uv`: one `[u, v]` per vertex in the texture's coordinates, `[0, 0]` at the top left corner and `[1, 1]` at the bottom right, as the template is drawn;
  - `triangles`: `[a, b, c]` vertex indices.

Validation SHALL refuse the package, naming the file, when a named preview mesh is missing or unreadable, when a piece names a texture the package does not have, when `positions` and `uv` have different lengths, when an index is out of range, or when a number is not finite. A package without any preview mesh SHALL stay valid. The application SHALL draw only the pieces whose texture it shows and ignore the others, so later versions can add pieces for accessory textures. Preview meshes count towards the package's size limit. A project SHALL NOT embed a preview mesh: it is read from the installed package.

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
