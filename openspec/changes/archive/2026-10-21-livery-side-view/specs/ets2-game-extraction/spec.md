## MODIFIED Requirements

### Requirement: Native model reading
The tool SHALL read the geometry (`.pmg`) and model description (`.pmd`) files of the vehicles it extracts itself, and from them obtain what painted-surface extraction needs: the looks and variants with their materials, the effect of each material, the parts with their visible state per variant, the pieces with their material, vertex positions, texture-coordinate streams (with the `_TEXCOORDn` aliases each stream carries) and triangles. Coordinates SHALL keep the exact 32-bit values stored in the game files. The data SHALL be identical to what the previous external converter produced for the same files, so that selection, UV and shader rules do not change. Files of a version the tool does not support, truncated files and offsets outside a file SHALL fail the affected vehicle with a message naming the file and the version found.

#### Scenario: Same geometry as before
- **WHEN** a model is read
- **THEN** its painted pieces have the same triangles, UV coordinates, material effects, aliases and visible parts as the converted model of the previous extractor, and one position per vertex

#### Scenario: Aliased stream
- **WHEN** a piece stores its second UV set with texture-coordinate mask nibble 1 mapped to `_TEXCOORD1`
- **THEN** the stream carrying the alias `_TEXCOORD1` is the one reported, whatever its position in the piece

#### Scenario: Truncated model
- **WHEN** a geometry file is shorter than its header declares
- **THEN** only the vehicle that needs it is refused, with the file named

## ADDED Requirements

### Requirement: Preview meshes in official packages
Each successful vehicle package SHALL carry a preview mesh (see the vehicle-packages capability) for every main texture whose template is not blank. The mesh SHALL be made of the painted pieces of one model of that texture: for a truck, the cabin model of the texture's first game id (the first cabin of the vehicle when the texture has none); for a trailer, the first painted definition the extractor reads. Pieces of other cabins or configurations SHALL NOT be added, so that the shapes of several cabins are not drawn on top of each other. Accessories painted with the main texture (doors, bumpers, fenders, deflectors, mirrors…) SHALL NOT be added either: the game models each of them around its own origin and mounts it at an attachment point the extractor does not read yet, so they would be drawn in the wrong place. Each piece SHALL keep the positions and the paint UVs read from the game, the UVs being the same as those that draw the template, and the triangles; normals, materials and any other part SHALL be left out. A main texture with a blank template (a trailer whose parts all have their own texture) SHALL get no preview mesh. The report SHALL record, for each main texture, the configuration its preview mesh comes from or why it has none.

#### Scenario: Truck with two cabin layouts
- **WHEN** a truck with the cabins `standard`, `standard_8x4` and `high_roof` is extracted, the first two sharing one main texture
- **THEN** its package has two preview meshes: one built from the `standard` cabin and one from `high_roof`, and neither contains the other cabin's pieces

#### Scenario: Door mounted as an accessory
- **WHEN** a truck's main texture template includes the cabin and the `cab_door` accessory painted with it, as on the DAF XD
- **THEN** its preview mesh holds the cabin's pieces only, and the template still holds both

#### Scenario: Trailer with every part on its own texture
- **WHEN** a trailer's main texture is the 64 × 64 blank one
- **THEN** that texture has no preview mesh and the report says why

#### Scenario: Same UVs as the template
- **WHEN** a preview mesh is drawn with the texture's template
- **THEN** every triangle edge of the mesh lies on a line of the template
