## MODIFIED Requirements

### Requirement: New version of a custom vehicle
A vehicle whose id starts with `custom.` SHALL offer **New Version…** in the Vehicle Library. It SHALL open the Custom Vehicle dialog filled in from the vehicle's newest installed version:
- its fields;
- each of its textures as a row, with its name, role, game ids, size and template.

In that dialog:
- the id, the kind and the game SHALL be shown and can't be changed;
- a **Version** field SHALL default to the next minor version (1.0.0 gives 1.1.0). It SHALL be refused unless it is a valid version higher than every installed version of the vehicle;
- the other fields and the rows SHALL be edited as when creating a vehicle;
- a **What changed** field, empty by default, SHALL hold the text of at most 1000 characters written as the new version's `changelog` (an empty field writes none). It SHALL be refused when too long, with a counter shown.

Create SHALL build and install the new version, alongside the older ones:
- a texture kept from the previous version keeps its texture id, even when renamed;
- a kept texture whose template the player replaced gets the previous layout version plus one. Otherwise it keeps its layout version;
- a new row gets a new texture id and layout version 1;
- a removed row is absent from the new version.

The checks, the building and the errors SHALL be those of creating a vehicle.

#### Scenario: Game update changed a cabin
- **WHEN** the player opens New Version… on `custom.scania.r_2024` 1.0.0, replaces the template of "cabin", keeps the accessory and clicks Create
- **THEN** version 1.1.0 is installed alongside 1.0.0, "cabin" has layout version 2, and the accessory has layout version 1

#### Scenario: Update Template on a custom vehicle
- **WHEN** a project uses `custom.scania.r_2024` 1.0.0 and the player installs 1.1.0 with New Version…, having replaced the "cabin" template
- **THEN** Update Template is offered for that vehicle, and confirming it flags "cabin" with "Layout changed"

#### Scenario: Version not higher
- **WHEN** the player enters 1.0.0 as the version of a new version of a vehicle whose newest installed version is 1.0.0
- **THEN** Create is disabled, and the Version field says the version must be higher than 1.0.0

#### Scenario: Not offered for other packages
- **WHEN** the Vehicle Library lists the sample truck
- **THEN** it offers no New Version… action

#### Scenario: What changed
- **WHEN** the player enters "Cabin template redrawn after game update 1.57" in What changed and clicks Create
- **THEN** version 1.1.0 is installed with that text as its changelog, and the Vehicle Library shows it for that version

#### Scenario: What changed left empty
- **WHEN** the player leaves What changed empty
- **THEN** the new version has no changelog
