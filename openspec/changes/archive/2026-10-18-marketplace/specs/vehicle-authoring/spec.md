## ADDED Requirements

### Requirement: Building the catalog
`tpv index <folder> --base-url <url>` SHALL read every `.tpv` in the folder, validate it like `tpv check`, and write `catalog.json` (see the catalog format) to the path given with `-o`, or into the folder. Each entry's `url` is `<base-url>/<file name>`. Entries SHALL be sorted by id then version, so the same input always gives a byte-identical file.

`--default-license <text>` and `--default-notice <text>` SHALL fill `license` and `notice` for packages that don't set them (game-derived packages carry no license of their own).

`--previous <catalog.json>` names the published catalog to compare with. The command SHALL fail, and write nothing, when a package is invalid, when an id and version appear twice, when an entry has no `license`, or, with `--previous`, when a version of the previous catalog now has another SHA-256 or is gone.

`--check` SHALL run the same checks and write nothing.

#### Scenario: Index a folder
- **WHEN** the folder holds two valid packages and `tpv index` runs
- **THEN** `catalog.json` lists both with their sizes and SHA-256

#### Scenario: Same input, same output
- **WHEN** `tpv index` runs twice on the same folder
- **THEN** both files are byte-identical

#### Scenario: Default license
- **WHEN** a package has no `license` and `--default-license "Personal modding only"` is given
- **THEN** its entry's license is "Personal modding only"

#### Scenario: Missing license
- **WHEN** a package has no `license` and no default is given
- **THEN** the command fails, names the package, and writes nothing

#### Scenario: Republished version
- **WHEN** a package has the version of an entry in `--previous` but a different SHA-256
- **THEN** the command fails, names the package and version, and writes nothing

#### Scenario: Removed version
- **WHEN** an entry of `--previous` has no package in the folder
- **THEN** the command fails and names it
