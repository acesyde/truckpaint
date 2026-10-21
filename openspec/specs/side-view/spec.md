# side-view Specification

## Purpose

The Side View shows, while a painter designs, the vehicle of the open texture drawn with the livery being edited, seen from fixed directions, so they can tell where each part of the flat texture lands on the vehicle. It is a guide for designing; the check in the game stays the final test.

## Requirements

### Requirement: Side View panel
In the Workshop, **View › Show Side View** SHALL show or hide a Side View panel under the canvas, across the width of the canvas area. The menu item SHALL show its state with a check mark and be enabled only in the Workshop. The panel's height SHALL be resizable by dragging its top edge, clamped between a minimum and a maximum height, and the canvas view SHALL follow the resizing rule of the Canvas area requirement (see the workspace-layout capability). Hide Panels SHALL NOT hide the Side View. Whether the panel is shown, its direction and its height SHALL be remembered across sessions (see the app-preferences capability). The panel SHALL be hidden on first launch.

#### Scenario: Showing the Side View
- **WHEN** the Workshop is shown and the user chooses View › Show Side View
- **THEN** a panel appears under the canvas across its width, the menu item is checked, and the zoom and the document point at the center of the canvas stay the same

#### Scenario: Resizing the panel
- **WHEN** the user drags the top edge of the Side View upwards
- **THEN** the panel grows live, clamped to its maximum height, and the canvas area shrinks by as much

#### Scenario: Hide Panels keeps the Side View
- **WHEN** the Side View is shown and the user presses Tab
- **THEN** the left panel and the inspector are hidden and the Side View stays

### Requirement: Fixed directions
The Side View SHALL draw the vehicle from one of five directions, chosen in a segmented control in the panel's header: **Left**, **Right**, **Front**, **Back** and **Top**. The drawing SHALL be a parallel projection (no perspective). For Left, Right, Front and Back the vehicle's up SHALL point up; for Top the vehicle is seen from above with its front towards the left of the panel, as in Left, so its right side is at the top. The drawing SHALL be fitted to the panel, centered, keeping its proportions. It SHALL NOT offer orbiting, panning or zooming of the model. Each face SHALL be shaded from its orientation to the viewer only enough to tell the shape apart; nearer faces SHALL hide farther ones.

#### Scenario: Left side
- **WHEN** the Side View shows the sample truck's Standard cab and the user picks Left
- **THEN** the cab is drawn seen from its left, front towards the left of the panel, filling the panel's height or width with its proportions kept

#### Scenario: Top
- **WHEN** a logo is placed on the roof panel of a cab and the user picks Top
- **THEN** the cab is drawn seen from above, front towards the left of the panel and its right side at the top, and the logo is drawn on the roof

#### Scenario: Front
- **WHEN** the user picks Front
- **THEN** the cab is drawn seen from the front, and its back faces are hidden by its front faces

### Requirement: What the Side View shows
The Side View SHALL draw the preview mesh of the open texture (see the vehicle-packages capability), painted with that texture's artwork as the canvas shows it without the template and guides. Only main textures SHALL be drawn: for a truck, the cabin of the open main texture; for a trailer, its main texture. Accessory textures SHALL NOT be drawn, and when an accessory texture is open the Side View SHALL show the main texture the painter last opened for the same vehicle, or the vehicle's first main texture. Switching to a texture of another vehicle or another main texture SHALL switch the drawing.

#### Scenario: Logo across a door
- **WHEN** a logo is placed on the left side panel of the sample truck's Standard cab and the Side View shows Left
- **THEN** the logo is drawn on the left of the cab, where the panel lies on the vehicle

#### Scenario: Switching cabins
- **WHEN** the Side View shows the Standard cab and the user opens the High roof texture
- **THEN** the Side View draws the High roof cab with its artwork

#### Scenario: Accessory texture open
- **WHEN** the user opens the Chassis texture of the sample truck after working on the Standard cab
- **THEN** the Side View keeps drawing the Standard cab

### Requirement: Live update
The Side View SHALL show each edit of the drawn texture's artwork: within a moment after the edit, and for an edit made by dragging, at the latest a moment after the drag ends. Drawing the Side View SHALL never make the canvas wait.

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
- for a main texture with no preview mesh because its template is blank (a trailer whose parts all have their own texture): that this texture only carries the base color and its parts are painted on accessory textures, which the Side View does not show yet;
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
