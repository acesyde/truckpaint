## ADDED Requirements

### Requirement: Building the catalog
`tpv index <folder> --base-url <url>` SHALL read every `.tpv` in the folder, validate it like `tpv check`, and write `catalog.json` (see the catalog format) next to them, or to the path given with `-o`. Each entry's `url` is `<base-url>/<file name>`. The output SHALL be the same for the same input, entries sorted by id then version.

`tpv index --check` SHALL write nothing, and fail when the existing `catalog.json` differs from what would be written, when a package is invalid, when an id and version appear twice, when `license` is missing, or when a version already in the existing catalog now has another SHA-256.

#### Scenario: Index a folder
- **WHEN** the folder holds two valid packages and `tpv index` runs
- **THEN** `catalog.json` lists both with their sizes and SHA-256

#### Scenario: Same input, same output
- **WHEN** `tpv index` runs twice on the same folder
- **THEN** both files are byte-identical

#### Scenario: Republished version
- **WHEN** a package in the folder has the version of an existing catalog entry but a different SHA-256, and `tpv index --check` runs
- **THEN** it fails, names the package and version, and changes nothing
