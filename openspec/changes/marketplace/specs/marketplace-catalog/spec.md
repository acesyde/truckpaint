## Purpose
Defines the public catalog of vehicle packages: the `catalog.json` file, the rules every published package follows, and the checks the catalog repository runs. The app and the repository agree on this format and nothing else.

## ADDED Requirements

### Requirement: Catalog file
The catalog SHALL be one JSON file, `catalog.json`, with:
- `schema`: the catalog format version (1);
- `generated`: the time it was built, in UTC;
- `packages`: one entry per published version of a vehicle.

Each entry SHALL have `id`, `version`, `name`, `brand`, `kind` (`truck` or `trailer`), `game` (`ets2` or `ats`), `size` (bytes), `sha256` (lowercase hex of the `.tpv`), `url` (HTTPS address of the `.tpv`), `license`, and `min_app` (the oldest TruckPaint that can install it). It MAY have `game_versions` (the range from the package), `preview` (an HTTPS image address), `authors` and `changelog`.

A reader SHALL ignore unknown fields. A catalog whose `schema` it does not know SHALL be refused with a message saying the app must be updated.

#### Scenario: Entry for a package
- **WHEN** `volvo_fh-1.2.0.tpv` is indexed
- **THEN** its entry has the package's own id, version, name, brand, kind and game, the file's size and SHA-256, and the address it will be downloaded from

#### Scenario: Unknown catalog format
- **WHEN** a catalog declares `schema` 2 and the app knows only 1
- **THEN** the app shows that it must be updated and lists nothing from that catalog

### Requirement: Immutable versions
A published version of a vehicle SHALL never change. A correction SHALL be a new, higher version. Within one id, every version SHALL be unique, and the catalog SHALL list all published versions.

#### Scenario: Changed file under the same version
- **WHEN** a pull request replaces the file of an already published `1.2.0` with different content
- **THEN** the repository's check fails and says the version is published and must be raised

### Requirement: Release assets and integrity
Each `.tpv` SHALL be downloadable from a stable HTTPS address (a GitHub Release asset). The entry's `size` and `sha256` SHALL describe exactly the file at `url`.

#### Scenario: Wrong checksum
- **WHEN** the file at an entry's address does not match its `sha256`
- **THEN** the repository's check fails, and the app, if it ever downloads it, refuses to install it

### Requirement: Repository checks
Every pull request on the catalog repository SHALL be checked: each `.tpv` is a valid package, ids and versions are unique and well formed, versions only go up, `license` is present, and the generated `catalog.json` matches what the packages say. A release SHALL publish the `.tpv` files and the regenerated catalog together.

#### Scenario: Invalid package in a pull request
- **WHEN** a pull request adds a `.tpv` that `tpv check` refuses
- **THEN** the check fails with the same message as `tpv check`
