## MODIFIED Requirements

### Requirement: What the Side View shows
The Side View SHALL draw the whole vehicle of the open texture from its preview mesh (see the vehicle-packages capability): the cabin of the open main texture for a truck (when an accessory texture is open, the main texture the painter last opened for the same vehicle, or the vehicle's first main texture), the chosen chassis, and the option chosen in every accessory slot (see the Accessories list requirement). Each piece SHALL be painted with the artwork of its own texture, main or accessory, as the canvas shows it without the template and guides; a piece whose texture is not in the project SHALL be drawn in a plain neutral color. Only painted parts SHALL be drawn: glass, lights, unpainted plastic or chrome and every other part without a paint texture never appear. Switching to a texture of another vehicle or another main texture SHALL switch the drawing; switching to an accessory texture of the same vehicle SHALL NOT.

#### Scenario: Logo across a door
- **WHEN** a logo is placed on the left side panel of the sample truck's Standard cab and the Side View shows Left
- **THEN** the logo is drawn on the left of the cab, where the panel lies on the vehicle

#### Scenario: Door mounted as an accessory
- **WHEN** the Side View shows a DAF XD whose door is the game's default option, from the left
- **THEN** the door is drawn in the cab's door opening, painted with the cab texture's artwork, with no gap

#### Scenario: Accessory with its own texture
- **WHEN** the sample truck's Side skirts texture holds a stripe and the Side View shows the Standard cab from the left with its side skirts
- **THEN** the stripe is drawn on the side skirts, under the cab, while the cab shows its own artwork

#### Scenario: Switching cabins
- **WHEN** the Side View shows the Standard cab and the user opens the High roof texture
- **THEN** the Side View draws the High roof cab with its artwork and the same accessory choices

#### Scenario: Accessory texture open
- **WHEN** the user opens the Side skirts texture of the sample truck after working on the Standard cab
- **THEN** the Side View keeps drawing the Standard cab, and edits on the side skirts show on the vehicle

#### Scenario: Trailer
- **WHEN** the Side View shows the sample trailer with the Curtain body 13.6 m chosen
- **THEN** the chassis and the curtain body are drawn, each with its own texture's artwork

#### Scenario: Texture not in the project
- **WHEN** the chosen roof deflector is painted with an accessory texture the project does not include
- **THEN** the deflector is drawn in a plain neutral color

### Requirement: Live update
The Side View SHALL show each edit of the artwork of every texture it draws: within a moment after the edit, and for an edit made by dragging, at the latest a moment after the drag ends. Drawing the Side View SHALL never make the canvas wait.

#### Scenario: Recoloring
- **WHEN** the Side View is shown and the user changes the fill color of an object on the drawn texture
- **THEN** the Side View shows the new color a moment later, without any other action

#### Scenario: Dragging an object
- **WHEN** the user drags an object across the drawn texture
- **THEN** the canvas follows the pointer as without the Side View, and the Side View shows the object at its new place once the drag ends

### Requirement: When there is nothing to draw
When the open texture's vehicle has no preview mesh to draw, the Side View SHALL stay shown and say why, in place of the drawing:
- for a custom vehicle: that the Side View is not available for custom vehicles;
- for an official vehicle whose installed package has none: that the vehicle must be generated again from My Game to get the Side View;
- for a package with no preview mesh that is neither official nor custom: that the Side View is not available for this vehicle;
- when the vehicle's package is not installed: that the vehicle's package is needed for the Side View.

The messages SHALL be in the interface language, like every other text.

#### Scenario: Custom vehicle
- **WHEN** the open texture belongs to a vehicle made with Custom Vehicle and the Side View is shown
- **THEN** the panel says the Side View is not available for custom vehicles

#### Scenario: Official package generated before the Side View
- **WHEN** the open texture belongs to an official truck whose installed package has no preview mesh
- **THEN** the panel says to generate the vehicle again from My Game

#### Scenario: Package not installed
- **WHEN** a project is opened on a computer where its vehicle's package is not installed
- **THEN** the canvas works as usual and the Side View says the vehicle's package is needed

An official package made before accessories were drawn, whose preview mesh has no accessory slot, SHALL still be drawn (the cabin alone, as before), with an empty Accessories list saying that generating the vehicle again from My Game adds its accessories.

## ADDED Requirements

### Requirement: Accessories list
The Side View panel's header SHALL have an **Accessories** button that opens a list of the vehicle's slots that have at least one painted option, one row per slot, in the order of the preview mesh: the slot's name and a drop-down with **None** followed by its options, by name. The chassis slot SHALL have no None, and a vehicle SHALL always have one chassis. Only one option per slot SHALL be drawn. A slot's name SHALL be shown in the interface language when TruckPaint knows the slot (Door, Front bumper, Roof deflector, Side deflectors, Side skirts, Sun visor, Body, Chassis…), and as its game name otherwise; an option's name SHALL be the game's English name. When the panel shows no vehicle, the button SHALL be disabled.

A vehicle SHALL start with the game's default configuration: in each slot, the option the game fits on a new vehicle when it is painted, and None otherwise. Choosing an option SHALL redraw the view at once.

#### Scenario: Removing the roof deflector
- **WHEN** the Side View shows a DAF XD with its default roof deflector and the user picks None for Roof deflector
- **THEN** the deflector disappears from the view and the roof is drawn

#### Scenario: One option per slot
- **WHEN** the user picks "Long" in Roof deflector after "Stock"
- **THEN** only the long deflector is drawn, never both

#### Scenario: Default configuration
- **WHEN** a DAF XD is added to a project and the Side View is shown for the first time
- **THEN** each slot shows the game's default option when it is painted, and None otherwise

#### Scenario: Unknown slot
- **WHEN** a package has a slot `toolbox_x` TruckPaint does not know
- **THEN** its row is named `toolbox_x` and works like the others
