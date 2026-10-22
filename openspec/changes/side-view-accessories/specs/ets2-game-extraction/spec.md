## MODIFIED Requirements

### Requirement: Native model reading
The tool SHALL read the geometry (`.pmg`) and model description (`.pmd`) files of the vehicles it extracts itself, and from them obtain what painted-surface extraction needs: the looks and variants with their materials, the effect of each material, the parts with their visible state per variant, the locators of each part (name, position, rotation and scale), the pieces with their material, vertex positions, texture-coordinate streams (with the `_TEXCOORDn` aliases each stream carries) and triangles. Coordinates SHALL keep the exact 32-bit values stored in the game files. The data SHALL be identical to what the previous external converter produced for the same files, so that selection, UV and shader rules do not change. Files of a version the tool does not support, truncated files and offsets outside a file SHALL fail the affected vehicle with a message naming the file and the version found.

#### Scenario: Same geometry as before
- **WHEN** a model is read
- **THEN** its painted pieces have the same triangles, UV coordinates, material effects, aliases and visible parts as the converted model of the previous extractor, and one position per vertex

#### Scenario: Aliased stream
- **WHEN** a piece stores its second UV set with texture-coordinate mask nibble 1 mapped to `_TEXCOORD1`
- **THEN** the stream carrying the alias `_TEXCOORD1` is the one reported, whatever its position in the piece

#### Scenario: Truncated model
- **WHEN** a geometry file is shorter than its header declares
- **THEN** only the vehicle that needs it is refused, with the file named

#### Scenario: Locators of a cabin
- **WHEN** the DAF XD `cab_sl` model is read
- **THEN** its locators include `cab_door` and `r_deflector`, with the part each belongs to (`eu`, `uk` or the default part), their positions and rotations

### Requirement: Preview meshes in official packages
Each successful vehicle package SHALL carry a preview mesh (see the vehicle-packages capability) for every main texture of a truck, and for a trailer's main texture. For a truck, the mesh SHALL hold the painted pieces of the cabin model of the texture's first game id (the first cabin of the vehicle when the texture has none), and that cabin's locators of the EU variant; pieces of other cabins SHALL NOT be added. It SHALL then hold one slot per accessory slot that has at least one painted option suitable for that cabin, and a `chassis` slot with the chassis suitable for it: each option is an accessory definition, its pieces are the painted pieces of its model in the model's own space, with the texture that paints them (the main texture or an accessory texture), and its parent is the model that has a locator of the slot's name (the cabin first, then the chassis). For a trailer, the `chassis` slot holds its chassis and the other slots (the body among them) hang on the chassis; the trailer's main texture gets a preview mesh even when its template is blank. Options of one slot whose painted pieces are identical SHALL be merged into one, named after the default option when one of them is, otherwise after the first. Accessories with no painted piece, and unpainted pieces of painted accessories, SHALL be left out. The `default` option of each slot SHALL follow the game's `defaults` of the cabin and of the chassis they fit, recursively; when the default of a slot is not painted, no option of that slot is the default. Each piece SHALL keep the positions and the paint UVs read from the game, the UVs being the same as those that draw the template, and the triangles; normals, materials and any other part SHALL be left out. An accessory whose slot has no locator in the cabin nor in any suitable chassis SHALL be left out and recorded in the report. The report SHALL record, for each main texture, the cabin its preview mesh comes from, its slots with their options and default, and every accessory left out with the reason.

#### Scenario: Truck with two cabin layouts
- **WHEN** a truck with the cabins `standard`, `standard_8x4` and `high_roof` is extracted, the first two sharing one main texture
- **THEN** its package has two preview meshes: one built from the `standard` cabin and one from `high_roof`, and neither contains the other cabin's pieces

#### Scenario: Door mounted as an accessory
- **WHEN** the DAF XD `sl` cabin texture is extracted
- **THEN** its preview mesh has a `cab_door` slot with parent `cabin` and two options (the three standard doors merged, the default, and the three safety-window doors merged, whose painted shape differs) and an `f_bumper` slot with parent `chassis`

#### Scenario: Unpainted accessory
- **WHEN** a slot's options are all unpainted (mirrors with no paint material, for example)
- **THEN** the preview mesh has no slot for it

#### Scenario: Trailer with every part on its own texture
- **WHEN** a trailer's main texture is the 64 × 64 blank one
- **THEN** its preview mesh has a `chassis` slot and a `body` slot whose options are painted by the body accessory textures

#### Scenario: Same UVs as the template
- **WHEN** a preview mesh is drawn with the texture's template
- **THEN** every triangle edge of the mesh lies on a line of the template
