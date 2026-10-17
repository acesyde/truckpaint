## MODIFIED Requirements

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
