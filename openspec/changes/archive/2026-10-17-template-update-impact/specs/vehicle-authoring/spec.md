## ADDED Requirements

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
