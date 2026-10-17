# custom-vehicles Specification

## Purpose

Lets a player create a vehicle package inside TruckPaint from the game's template files, when no package exists for their vehicle, and make new versions of it after a game update.

## Requirements

### Requirement: Custom Vehicle dialog
The **Custom Vehicle** dialog SHALL be reached from:
- **Custom Vehicle…** in the vehicle list of New Project (see the start-screen capability);
- **Custom Vehicle…** in Add Vehicle… (see the vehicle-projects capability);
- **Custom Vehicle…** in the Vehicle Library (see the vehicle-packages capability).

It SHALL ask for:
- **Name** and **Brand**, both required;
- **Kind:** truck or trailer;
- **Game:** ETS2 or ATS. Opened from Add Vehicle…, the game is the project's game and can't be changed;
- **Game path:** the vehicle's path in the game's definitions (`scania.r_2016`), required;
- **Game versions:** optional. Empty means any game version; otherwise a version range (`>=1.53`);
- **Alternate UV set** and **Color picker:** unchecked by default;
- the **templates**, as described by the Templates requirement.

The dialog SHALL remind the player that templates from the base games belong to SCS Software and are for their own use.

Cancel or Escape SHALL close the dialog without installing anything and return to the dialog it was opened from, unchanged. The dialog SHALL be operable by keyboard.

#### Scenario: Opened from New Project
- **WHEN** the player clicks Custom Vehicle… in the vehicle list of New Project
- **THEN** the Custom Vehicle dialog opens with an empty form, Kind set to Truck and Game set to ETS2

#### Scenario: Opened from Add Vehicle
- **WHEN** the player clicks Custom Vehicle… in Add Vehicle… of an ATS project
- **THEN** the dialog's game is ATS and can't be changed

#### Scenario: Cancelling
- **WHEN** the player fills in part of the form and presses Escape
- **THEN** the dialog closes, nothing is installed, and the dialog it was opened from is shown as it was

### Requirement: Templates
The player SHALL add templates by dropping files on the dialog or with **Add Templates…**, one or more at a time. Accepted files:
- PNG;
- SVG;
- DDS, in the formats `tpv` accepts: BC1, BC2 or BC3 (also with a DX10 header), and uncompressed 24 or 32-bit RGB(A).

Each file SHALL become one **texture row**, in the order added, with:
- **Name:** the file name without its extension, with `_` and `-` shown as spaces (`highline_8x4.dds` gives "highline 8x4"); editable;
- **Role:** Main texture or Accessory. A row is a main texture when the vehicle has no main texture yet, an accessory otherwise; the player can change it;
- **Game ids:** empty; editable as a list separated by commas or spaces;
- **Size:** a width and a height, each a power of two from 64 to 8192. They default to the image's width and height, each rounded to the nearest allowed side when it isn't one. The player can change either.

A row SHALL show its file name and the image's dimensions. It SHALL warn when the image's size differs from the texture's size, since the template is then scaled to the texture.

Each row SHALL offer:
- **Replace…**, which picks another file for that row; dropping a single file on a row does the same;
- **Remove**.

A file that can't be read SHALL not become a row. The dialog SHALL name the file and the reason, for example a DDS compression that isn't supported. Other files of the same drop SHALL still be added.

#### Scenario: Drop the game's templates
- **WHEN** the player drops `cabin.dds` (BC3, 4096×4096) and `mirrors.dds` (BC1, 1024×1024) on an empty dialog
- **THEN** the dialog lists "cabin" as a main texture of size 4096² and "mirrors" as an accessory of size 1024²

#### Scenario: Unsupported DDS
- **WHEN** the player drops `cabin.png` and `glass.dds`, a BC7 DDS file
- **THEN** "cabin" is added, and the dialog says that `glass.dds` can't be read because its compression is not supported

#### Scenario: Replace a template
- **WHEN** the player clicks Replace… on the "cabin" row and picks `cabin_v2.png`
- **THEN** the row keeps its name, role and game ids, and shows `cabin_v2.png`

#### Scenario: Not square
- **WHEN** the player adds a 4096×2048 PNG
- **THEN** its row has the size 4096 × 2048 and shows no warning: the texture keeps the image's shape

#### Scenario: Size that isn't a power of two
- **WHEN** the player adds a 4000×1000 PNG
- **THEN** its row has the size 4096 × 1024 and warns that the template will be scaled to that size

### Requirement: Describing the paint job
The dialog SHALL check the form as the player types and show each problem next to its field. **Create** SHALL be enabled only when there is none:
- the name or brand is empty;
- the game path is not words of `a`–`z`, `0`–`9` and `_` separated by dots;
- the game versions are not a valid version range;
- there is no main texture;
- a trailer has more than one main texture;
- there are several main textures and one of them has no game id;
- an accessory has no game id;
- a game id is not words of `a`–`z`, `0`–`9` and `_` separated by dots;
- a game id appears twice among the main textures, or twice among the accessories;
- a texture's name is empty.

A truck with a single main texture without game ids SHALL be accepted: that texture is painted on every cabin.

The game ids field SHALL say what to enter:
- for a main texture, the internal names of the cabins sharing its layout;
- for an accessory, the accessory ids it covers.

#### Scenario: One cabin layout
- **WHEN** a truck has one main texture without game ids and one accessory "Mirrors" with the game id `mirror.painted`, and the other fields are valid
- **THEN** Create is enabled

#### Scenario: Two cabin layouts need cabin names
- **WHEN** a truck has the main textures "Highline" with the game ids `highline` and "Topline" without game ids
- **THEN** Create is disabled, and the "Topline" row asks for the internal names of its cabins

#### Scenario: Accessory without game id
- **WHEN** an accessory "Side skirts" has no game id
- **THEN** Create is disabled, and the "Side skirts" row asks for the accessory ids it covers

#### Scenario: Trailer with two main textures
- **WHEN** the kind is Trailer and two rows are main textures
- **THEN** Create is disabled, and the dialog says that a trailer has one main texture

#### Scenario: Invalid game path
- **WHEN** the player types `Scania R` as the game path
- **THEN** Create is disabled, and the game path field shows the expected form, with an example

### Requirement: Creating a custom vehicle
**Create** SHALL build a vehicle package as `tpv pack` does (see the vehicle-authoring capability):
- **id:** `custom.<brand>.<name>`. Each word is the brand or the name, lowercased, with characters outside `a`–`z`, `0`–`9`, `_` and `-` replaced by `_`, repeated `_` collapsed, and `_` trimmed at both ends. A word left empty becomes `vehicle`;
- **version:** 1.0.0;
- **every texture** at layout version 1. It gets a texture id made from its name the same way, unique within the vehicle, and its template;
- **DDS templates** converted to PNG of the same pixels.

The package SHALL pass the same validation as installing a package. It SHALL then be installed in the library, like an installed `.tpv`. While the package is built, the dialog SHALL show that it is working and SHALL stay responsive.

On success, the dialog SHALL close, and the dialog it was opened from SHALL show the new vehicle:
- in New Project and Add Vehicle…, it is selected, with its first main texture and every accessory checked;
- in the Vehicle Library, it is listed.

A confirmation SHALL name the vehicle and its version.

When a vehicle with the same id is already installed, Create SHALL be refused. The message SHALL say so and point to New Version… in the Vehicle Library. When building or validation fails, the dialog SHALL stay open with the reason. Nothing SHALL be installed then.

#### Scenario: Create from New Project
- **WHEN** the player creates the custom truck "R 2024" of brand "Scania", with one main texture "cabin" and one accessory, from the vehicle list of New Project
- **THEN** the package `custom.scania.r_2024` 1.0.0 is installed, and the vehicle list of New Project shows "R 2024" selected with "cabin" and the accessory checked

#### Scenario: DDS converted
- **WHEN** a custom vehicle is created from a BC3 DDS template of 4096×4096
- **THEN** the installed package holds a 4096×4096 PNG template of the same pixels

#### Scenario: Same id already installed
- **WHEN** the player creates the custom truck "R 2024" of brand "Scania" while `custom.scania.r_2024` is already installed
- **THEN** Create is refused with a message pointing to New Version…, and the library is unchanged

#### Scenario: Shared like any package
- **WHEN** a custom vehicle is exported from the Vehicle Library and installed on another computer
- **THEN** it is accepted and listed like any other vehicle

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
