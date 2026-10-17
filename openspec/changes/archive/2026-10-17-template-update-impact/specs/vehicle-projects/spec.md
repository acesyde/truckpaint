## MODIFIED Requirements

### Requirement: Update Template
Update Template SHALL work on one vehicle of the project. It is reached from that vehicle's update button or ⋯ menu in the Textures tab, from its vehicle card in the Project space (see the project-screen capability), or from Vehicle › Update Template…, which applies to the vehicle of the active surface. It SHALL be enabled when a newer version of that vehicle's package is installed.

It SHALL show what will change for every texture of the vehicle in the project. It SHALL also list the textures that are new in that version: new accessories as checked checkboxes and new main textures as unchecked ones. When the new version has a single main texture and the vehicle doesn't paint it yet, that texture is always added. On confirmation, it switches the vehicle to that version as one undo step:

- **Same texture id:** the template is replaced, the artwork is kept, and the size is kept.
  - If the texture's size changed, the artwork is scaled to the new size: horizontally by the ratio of the widths and vertically by the ratio of the heights.
  - If its layout version changed, the surface is flagged "Layout changed".
- **Checked new texture, or an always-added single main texture:** a new empty surface is added with its template, in package order among the vehicle's surfaces.
- **Texture of the project missing from the new version:** the surface and its artwork are kept, marked "Not in this version" and drawn without a template.
- The vehicle's recorded package version becomes the new one. The project's other vehicles are unchanged.

The dialog SHALL also show the new version's changelog when it has one and, for each texture flagged "Layout changed", the number of objects on it that will have to be repositioned (see the template-update-impact capability). On confirmation, those objects SHALL be flagged (see the layers-panel capability) and remembered with the project.

Undo SHALL restore the previous templates, surfaces and recorded version. Undo SHALL also remove the flags the update added.

#### Scenario: Patch that changed the cabin layout
- **WHEN** a vehicle in a project uses version 1.2.0 (Standard cab layout 1, Chassis layout 1) and the user updates it to 1.3.0 (Standard cab layout 2, Chassis layout 1)
- **THEN** both templates are replaced, the artwork is unchanged, Standard cab is flagged "Layout changed", Chassis is not, and the vehicle records version 1.3.0

#### Scenario: New accessory in the new version
- **WHEN** the user updates a project's sample truck from 1.0.0 to 1.1.0
- **THEN** Update Template lists "Side skirts" as a new texture, checked, and confirming adds a Side skirts surface after the vehicle's other accessories

#### Scenario: Undo an update
- **WHEN** the user presses Undo right after updating a vehicle to 1.3.0
- **THEN** the vehicle's templates and recorded version are those of 1.2.0 again

#### Scenario: Only one vehicle is updated
- **WHEN** a project holds the sample truck 1.0.0 and the sample trailer, and the user updates the sample truck to 1.1.0
- **THEN** the truck's surfaces are updated and the trailer's surfaces and recorded version are unchanged

#### Scenario: Update from the Project space
- **WHEN** the Project space is shown and the user starts Update Template from the sample truck's vehicle card
- **THEN** the same Update Template dialog opens for the sample truck

#### Scenario: Objects to reposition in the dialog
- **WHEN** the Standard cab texture holds a logo, a text and a group of two rectangles, and the user opens Update Template to a version that changed its layout
- **THEN** the dialog lists Standard cab as "Layout changed" with 4 objects to reposition, and shows the version's changelog

#### Scenario: Update without a changelog
- **WHEN** the new version has no changelog
- **THEN** the dialog shows no changelog section
