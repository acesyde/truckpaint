## ADDED Requirements

### Requirement: Mod package reading
The tool SHALL also read the files of a Steam Workshop mod package, without any external program and without modifying it, whether the package is a ZIP archive, a HashFS archive or a folder. It SHALL find a file by its game path, list a folder (one level and recursively) and return the file's bytes, whether an entry of a ZIP is stored or deflated. The entries of an archive SHALL be read only when asked for: opening a ZIP SHALL read its list of files, not their content. Reading SHALL be safe on damaged input: an entry name that is absolute or goes above the root, an offset or size outside the archive, a size that does not match the unpacked data, an encrypted entry, or a compression method the tool does not support SHALL fail with a message naming the package, the path when there is one, and the cause. A folder SHALL NOT be followed through a symbolic link that leaves it. No entry SHALL be skipped silently.

#### Scenario: ZIP entry
- **WHEN** a ZIP package holds a deflated `manifest.sii` and a stored `base/def/vehicle/truck/x/data.sii`
- **THEN** both are returned with their original bytes, and no other entry is read

#### Scenario: Entry above the root
- **WHEN** a ZIP entry is named `../../x.sii`
- **THEN** it is refused with a message naming the package and the entry, and nothing is written outside

#### Scenario: Damaged ZIP
- **WHEN** the list of files of a ZIP is cut short
- **THEN** reading it fails with a message naming the package, and the other mods are still examined

#### Scenario: Unsupported compression
- **WHEN** an entry we need uses a method other than stored or deflate
- **THEN** reading it fails naming the package, the path and the method

#### Scenario: Folder package
- **WHEN** a package is a folder
- **THEN** its files are read by the same game paths as in an archive

### Requirement: Extracting from a mod over the game
The extractor SHALL be able to build a truck from the game's archives with the files of one mod package mounted after them, under the order rule of Native archive reading (the last mounted wins). The truck's definition, models and materials SHALL be resolved through that mount, with the provenance recorded as for any vehicle, naming the mod's package for the files it holds. Selection, shader, UV and coverage rules SHALL be those of an official truck. The mod's files SHALL NOT change what another vehicle reads: each truck is built from the game and its own mod only.

#### Scenario: Model from the mod
- **WHEN** a mod truck's cabin model is in the mod package
- **THEN** the template is built from it and the report names the package as the source

#### Scenario: Model from the game
- **WHEN** a mod truck's definition points to a model that only the game holds
- **THEN** the template is built from the game's model

#### Scenario: Two mods do not mix
- **WHEN** two mods each override the same game file
- **THEN** a truck of the first mod is built with the first mod's file, and a truck of the second with the second's

### Requirement: One mod truck at a time
Mod trucks SHALL be extracted strictly one after another, whatever the number of processor cores, because reading a ZIP entry unpacks it whole in memory and a single model can be over 100 MB. Official vehicles extracted in the same batch SHALL keep the parallelism of the batch. A batch that holds only official vehicles SHALL behave as before.

#### Scenario: Two trucks of one mod
- **WHEN** a mod with two trucks is extracted on a machine with many cores
- **THEN** the first truck is finished before the second starts

#### Scenario: Official vehicles unchanged
- **WHEN** a batch of official vehicles is extracted
- **THEN** it uses the same number of workers as before this change
