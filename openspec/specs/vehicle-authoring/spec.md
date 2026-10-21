# vehicle-authoring Specification

## Purpose

Defines how vehicle packages are authored: a command-line packer that builds and checks `.tpv` files, including from the game's DDS templates, and the sample truck and trailer shipped with TruckPaint as reference examples and for end-to-end testing.

## Requirements

### Requirement: Packing a folder
The `tpv pack <folder>` command SHALL build a vehicle package from a folder holding a `vehicle.json` manifest and the files it references. It writes the result to the path given with `-o`, or, by default, to `<id>-<version>.tpv` in the current directory.

The package SHALL contain:
- the manifest;
- every template it references;
- its preview, if any.

Other files in the folder SHALL be left out, and the packer SHALL list them as ignored.

Before writing anything, the packer SHALL validate the result exactly as the application does when installing. On failure, it SHALL print the reason and the file concerned, write no file, and exit with a non-zero status.

Packing the same folder twice SHALL produce identical bytes.

#### Scenario: Pack a valid folder
- **WHEN** the author runs `tpv pack my-truck/` on a folder with a valid manifest and its templates
- **THEN** `community.jdoe.my_truck-1.0.0.tpv` is written, and the application installs it without error

#### Scenario: Missing template
- **WHEN** the manifest references `templates/cabin.png` and the folder has no such file
- **THEN** the packer reports that the Cabin template is missing, writes no file and exits with a non-zero status

#### Scenario: Unrelated files
- **WHEN** the folder also holds `notes.txt` and `cabin.psd`
- **THEN** the package does not contain them and the packer lists them as ignored

### Requirement: DDS templates
The packer SHALL accept templates in DDS format, as SCS distributes them: uncompressed, or block-compressed BC1, BC2 or BC3. It SHALL convert each one to a PNG of the same pixels and update the template's path in the packaged manifest to that PNG. The source folder SHALL NOT be modified.

A DDS file the packer cannot decode SHALL fail packing, naming the texture concerned.

#### Scenario: Pack SCS templates
- **WHEN** the manifest references `templates/cabin.dds`, a BC3 DDS file of 4096×4096
- **THEN** the package holds `templates/cabin.png`, a 4096×4096 PNG, and its manifest references that file

#### Scenario: Unsupported DDS
- **WHEN** a referenced DDS file uses a compression the packer cannot decode
- **THEN** packing fails with a message naming the texture, and no file is written

### Requirement: Checking a package
The `tpv check <file.tpv>` command SHALL validate a package as the application does when installing. On success it SHALL print a summary:
- the id, version, name, kind, game, game path and supported game versions;
- the paint job: each main texture, then each accessory, with its game ids and its texture size.

On failure it SHALL print the reason. The exit status SHALL be zero only for a valid package.

#### Scenario: Check a downloaded package
- **WHEN** the author runs `tpv check volvo_fh16-1.3.0.tpv` on a valid package
- **THEN** its id, version, game path, main textures, accessories and texture sizes are printed and the command exits with status 0

### Requirement: Sample vehicle
TruckPaint SHALL ship a sample truck, "TruckPaint Sample Truck", made only of original artwork and no game asset. Its id is `community.truckpaint.sample_truck`, it targets ETS2, and its game path is fictional.

Its templates SHALL be vector drawings that look like a truck's UV layout:
- labelled panels for the cab sides, front, rear and roof;
- the doors, the chassis rails, the fuel tanks and the accessories.

Two versions SHALL be provided:
- **1.0.0:** the main textures **Standard cab** and **High roof** (4096 each, one per cabin layout), and the accessories **Chassis** (2048) and **Cab accessories** (1024), all at layout version 1.
- **1.1.0:** a newer supported game range. The Standard cab's layout changes (layout version 2), the Chassis becomes 4096, and a **Side skirts** accessory (1024) is added.

Each version's sources SHALL be kept in the repository next to the package built from them. A test SHALL fail when a committed package does not match what packing its sources produces.

#### Scenario: Try Update Template end to end
- **WHEN** a painter creates a project from the Standard cab of sample truck 1.0.0 with its accessories, paints on it, installs sample truck 1.1.0 and runs Update Template, keeping Side skirts checked
- **THEN** the Standard cab texture is flagged "Layout changed", the Chassis artwork is scaled to 4096, and a new Side skirts texture appears

#### Scenario: Stale sample package
- **WHEN** a template source of a sample is edited and its package is not rebuilt
- **THEN** the test suite fails and names the out-of-date package

### Requirement: Sample trailer
TruckPaint SHALL ship a sample owned trailer, "TruckPaint Sample Trailer", made only of original artwork and no game asset. Its id is `community.truckpaint.sample_trailer`, it targets ETS2, and its game path is fictional.

Its version 1.0.0 SHALL have:
- the main texture **Base** (2048): the chassis frame, the landing gear and the front wall;
- the accessories **Curtain body 13.6 m** (4096), **Curtain body 10.5 m** (4096) and **Mudflaps** (512), all at layout version 1.

Its sources and built package SHALL be kept and checked like the sample truck's.

#### Scenario: A trailer project needs no choice
- **WHEN** a painter creates a project from the sample trailer
- **THEN** the project has the Base texture, always included, and the three accessories

### Requirement: Maintained game extraction command
The Rust `tpv` command SHALL provide the ETS2 extraction interface and behavior defined by the ets2-game-extraction capability alongside its existing pack and check commands. The old `tpv catalog check` and `tpv catalog sync` commands SHALL be removed from the accepted interface and help.

#### Scenario: Authoring command help
- **WHEN** an author asks for `tpv --help`
- **THEN** it documents extract, pack and check, with no Carsmaniac catalog commands

#### Scenario: Removed command
- **WHEN** an author invokes `tpv catalog sync`
- **THEN** the tool rejects the command with a nonzero exit status

### Requirement: Changelog in packages
`tpv pack` SHALL write the manifest's optional `changelog` into the package unchanged, and SHALL validate it as the application does when installing. `tpv check` SHALL print the changelog in its summary when the package has one, and SHALL fail, with the reason, when it is longer than 1000 characters. A manifest without `changelog` SHALL pack and check exactly as before.

#### Scenario: Pack with a changelog
- **WHEN** the author runs `tpv pack my-truck/` on a manifest with `"changelog": "Door layout redrawn."`
- **THEN** the package is written, and `tpv check` on it prints "Door layout redrawn."

#### Scenario: Changelog too long
- **WHEN** the manifest's `changelog` has 1001 characters
- **THEN** the packer reports the changelog is too long, writes no file and exits with a non-zero status

#### Scenario: No changelog
- **WHEN** the manifest has no `changelog`
- **THEN** packing and checking succeed and print no changelog

### Requirement: Sample preview meshes
Every main texture of the sample truck (both versions) and of the sample trailer SHALL have a hand-made preview mesh (see the vehicle-packages capability), made of a few boxes whose faces map the labelled panels of its template: for the cab, the sides, front, rear and roof; for the trailer's Base, the chassis frame and the front wall. The meshes SHALL be original work like the templates, kept as sources next to them and packed into the committed packages. A test SHALL fail when a face of a sample mesh does not lie inside a labelled panel of its template, so a template change cannot leave its mesh behind unnoticed.

#### Scenario: Sample truck in the Side View
- **WHEN** a painter opens the Standard cab of the sample truck and shows the Side View from the left
- **THEN** a cab shape is drawn with the artwork of the cab's left side panel on its left face

#### Scenario: Template panel moved
- **WHEN** a panel of the Standard cab template is moved and the preview mesh is not updated
- **THEN** the test suite fails and names the sample and the texture
