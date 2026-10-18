## ADDED Requirements

### Requirement: Native archive reading
The tool SHALL read the installed game's `.scs` archives itself, without any external program, and SHALL NOT modify them. It SHALL read HashFS version 2 archives, find a file by its game path, list a folder (one level and recursively), and return the file's bytes, whether the entry is stored or compressed. Archives SHALL be mounted in the order the game mounts them (base archives, then sorted DLC archives); when several archives hold the same path, the last mounted SHALL win, as the extractor's recorded archive precedence states. Reading SHALL be safe on damaged input: an invalid header, entry or offset, a size that does not match the decompressed data, or an entry that is encrypted or uses a compression the tool does not support SHALL fail with a message naming the archive, the game path when there is one, and the cause.

#### Scenario: Last mounted archive wins
- **WHEN** a base archive and a DLC archive both hold `/def/vehicle/truck/x/accessory/cabin.sii`
- **THEN** the content of the DLC archive is read and the report lists both archives in mount order

#### Scenario: Compressed and stored entries
- **WHEN** an archive holds one stored file and one zlib-compressed file
- **THEN** both are returned with their original bytes

#### Scenario: Recursive listing
- **WHEN** a folder is listed recursively
- **THEN** every file below it in that archive is returned with its full game path, and an archive whose root is not listable does not stop the listing of other archives

#### Scenario: Damaged archive
- **WHEN** an entry points outside its archive
- **THEN** reading fails with an error naming the archive and the path, and no panic or partial file is produced

### Requirement: Native model reading
The tool SHALL read the geometry (`.pmg`) and model description (`.pmd`) files of the vehicles it extracts itself, and from them obtain what painted-surface extraction needs: the looks and variants with their materials, the effect of each material, the parts with their visible state per variant, the pieces with their material, texture-coordinate streams (with the `_TEXCOORDn` aliases each stream carries) and triangles. Coordinates SHALL keep the exact 32-bit values stored in the game files. The data SHALL be identical to what the previous external converter produced for the same files, so that selection, UV and shader rules do not change. Files of a version the tool does not support, truncated files and offsets outside a file SHALL fail the affected vehicle with a message naming the file and the version found.

#### Scenario: Same geometry as before
- **WHEN** a model is read
- **THEN** its painted pieces have the same triangles, UV coordinates, material effects, aliases and visible parts as the converted model of the previous extractor

#### Scenario: Aliased stream
- **WHEN** a piece stores its second UV set with texture-coordinate mask nibble 1 mapped to `_TEXCOORD1`
- **THEN** the stream carrying the alias `_TEXCOORD1` is the one reported, whatever its position in the piece

#### Scenario: Truncated model
- **WHEN** a geometry file is shorter than its header declares
- **THEN** only the vehicle that needs it is refused, with the file named

### Requirement: Supported game format versions
Reading SHALL accept only the archive and model format versions it implements and has verified on a real installation. For any other version of an archive, geometry file or model description, the failure message SHALL name the file (archive file name, or game path inside it), the format and the version found, and SHALL say that this game version is not supported yet. An unsupported archive SHALL stop the batch before any vehicle is processed; an unsupported model SHALL refuse only the vehicles that use it. No file SHALL be skipped silently.

#### Scenario: Newer archive version
- **WHEN** an installed archive declares HashFS version 3
- **THEN** the batch stops before processing and the message names the archive and "HashFS version 3"

#### Scenario: Unknown geometry version
- **WHEN** a model's geometry file declares a version the tool does not implement
- **THEN** vehicles using it are refused with the game path and the version, and other vehicles are still built

#### Scenario: Unsupported compression
- **WHEN** an entry we need uses a compression method the tool does not implement
- **THEN** reading it fails naming the archive, the path and the method

## MODIFIED Requirements

### Requirement: Extraction command and source boundary
The tool SHALL expose `tpv extract <installation> [--vehicle <identifier>] [--output <directory>] [--previous <batch directory>] --package-version <version>`. A valid SemVer package version SHALL be required. The command SHALL read the game itself and SHALL NOT need an external converter; `--converter` is not an option. Without a vehicle filter it SHALL discover official trucks (`def/vehicle/truck/`) and official owned trailers (`def/vehicle/trailer_owned/`) from the installed ETS2 archives, including installed vehicle, trailer and tuning DLC. It SHALL exclude user mods and ATS vehicles, and SHALL NOT modify the installation. A game id defined both as a truck and as an owned trailer SHALL refuse that id rather than pick one. An unknown or excluded requested vehicle, invalid installation or invalid arguments SHALL return failure rather than an empty successful batch.

#### Scenario: Installed DLC trucks
- **WHEN** extraction runs without a vehicle filter on ETS2 with vehicle and tuning DLC installed
- **THEN** its truck and trailer discovery and painted option coverage include those DLC and exclude user mods

#### Scenario: No converter needed
- **WHEN** extraction runs on a machine with only the game installed
- **THEN** it reads the archives and models itself and succeeds without any other program

#### Scenario: Explicit version
- **WHEN** the package version is missing or is not SemVer
- **THEN** extraction fails before producing packages

#### Scenario: Filtered vehicle
- **WHEN** `--vehicle daf.2021` names a discovered truck
- **THEN** only that vehicle is processed, with its available cabins and accessories

#### Scenario: Filtered trailer
- **WHEN** `--vehicle krone.coolliner` names a discovered owned trailer
- **THEN** only that trailer is processed, with its available bodies, chassis and accessories

#### Scenario: Unsupported installation or vehicle
- **WHEN** the installation is ATS or the requested vehicle is missing
- **THEN** the command fails and produces no package for that request

### Requirement: Complete autonomous vehicle package
Each successful vehicle SHALL produce one `.tpv` containing its SVG templates and manifest, with id `official.ets2.<vehicle>` and the supplied package version. It SHALL record the game path, supported game version range and paint/export metadata. Every cabin SHALL be covered exactly once by a main texture, and every supported painted accessory assignment SHALL be covered by its independent texture or the appropriate main texture. Main templates SHALL superpose compatible painted options. Cabin textures SHALL be 4096 × 4096. Accessory dimensions SHALL come from game paint job definitions using the validated crop-name method. When the game declares several crop sizes for one accessory, the largest (by area) SHALL be used, since paint UVs are normalized. A declared size with a side below the package minimum (64) SHALL be scaled up by a whole factor that keeps its aspect ratio. An accessory whose paint overrides only use masks without crop dimensions (shared fill masks) SHALL get no independent texture and SHALL contribute to the main texture, as in game paint jobs that do not override it; no template size SHALL be inferred from a fill mask. Both cases SHALL appear in `dimension_diagnostics` with a `resolved_largest`, `raised_to_minimum` or `main_texture` disposition, the selected size, candidate sizes and mask references tied to their source definitions/archives. Empty templates, invalid geometry and incomplete coverage SHALL refuse the vehicle. Successful packages SHALL pass the same validation as manual application installation and SHALL require neither the game nor any converter for installation and use.

#### Scenario: Complete MAN or DAF
- **WHEN** extraction succeeds for MAN TGX or DAF NGD
- **THEN** its package contains every discovered cabin and painted accessory, valid export identifiers and all referenced templates

#### Scenario: Conflicting accessory dimensions
- **WHEN** an accessory has two declared crop dimensions, such as 1024×512 and 2048×512
- **THEN** its independent texture uses the larger size and `dimension_diagnostics` records both candidates with a `resolved_largest` disposition

#### Scenario: Accessory with only shared fill masks
- **WHEN** every paint override of an accessory uses a mask without crop dimensions
- **THEN** the accessory has no independent texture, contributes to the compatible main textures and is recorded with a `main_texture` disposition

#### Scenario: Missing painted coverage
- **WHEN** a cabin or painted accessory cannot be assigned a valid nonempty template
- **THEN** the entire vehicle is refused rather than producing a reduced package

### Requirement: Isolated batch outputs
Each invocation that starts a batch SHALL create a unique batch subdirectory under the output parent, defaulting to `dist/`. Intermediates (package staging) SHALL remain under `target/`, separate from published packages, in a work folder unique to the batch. When the batch succeeds (every requested or discovered vehicle built or skipped as not paintable), its work folder SHALL be deleted after the report is written. When any vehicle fails or the report cannot be written, the work folder SHALL be kept, so the intermediates of the failed vehicles remain available. Work folders of other batches SHALL NOT be touched. Existing batch contents SHALL NOT be overwritten. Vehicles of a batch MAY be built concurrently; each concurrent build SHALL keep its intermediates in its own subfolder of the batch work folder. The published packages, the report content and the order of report entries SHALL be the same as when the vehicles are built one at a time, in trucks-then-trailers id order. Vehicle failures SHALL NOT stop other vehicles. Only fully built and validated packages SHALL be published; failed vehicles and interrupted package writes SHALL leave no partial `.tpv`. The command SHALL return zero only when every requested/discovered vehicle succeeds or is skipped as not paintable and the final report is written; any incomplete batch or output failure SHALL return nonzero.

#### Scenario: Converter failure for one truck
- **WHEN** a model needed by one vehicle cannot be read and every other vehicle succeeds
- **THEN** the successful package is published, the failed vehicle has no package, both results are reported and the command returns nonzero

#### Scenario: Repeated invocation
- **WHEN** two batches use the same output parent
- **THEN** they use distinct directories and neither overwrites the other's outputs

#### Scenario: Package write failure
- **WHEN** publishing a package fails partway through writing
- **THEN** no partial `.tpv` is left as a published package and the command returns nonzero

#### Scenario: Successful batch cleans its work folder
- **WHEN** every vehicle of a batch is built or skipped as not paintable
- **THEN** the packages and report are published and the batch's work folder under `target/` no longer exists

#### Scenario: Failed batch keeps its work folder
- **WHEN** one vehicle of a batch fails
- **THEN** the batch's work folder is kept and the report names it

#### Scenario: Concurrent builds give the same batch
- **WHEN** a batch builds several vehicles concurrently
- **THEN** each package is byte-identical to the package from a batch that builds only that vehicle, and the report lists results in trucks-then-trailers id order

### Requirement: Traceable technical report
Every completed or recoverably failed batch SHALL publish `report.json` beside its successful packages. The report SHALL identify the selected game installation, mounted archive sources and precedence, game version, the reader version (the version of the native game reader), tool and package versions, requested and discovered vehicles, successes, failures and limits, and the batch work folder with whether it was kept. Per-vehicle evidence SHALL identify source definitions and model/look/variant/UV selections contributing to each texture, its dimensions and coverage, or the stage and reason for refusal. Unavailable source version information SHALL be marked explicitly. Native package validation SHALL be described as technical validation, never as proof of in-game correctness. Newly generated vehicles SHALL be identified as needing in-game checks unless explicit matching in-game evidence is available.

#### Scenario: Technical success without game evidence
- **WHEN** a new truck passes extraction and native package validation
- **THEN** the report records technical success and separately states that in-game verification remains required

#### Scenario: Reader version
- **WHEN** a batch completes
- **THEN** `report.json` records the reader version in the place of the former converter identity, and has no converter or model cache entry

#### Scenario: Explain a failed vehicle
- **WHEN** a vehicle fails, for example because a required model file is missing
- **THEN** its report identifies the concerned definitions or files and the failure stage

#### Scenario: Missing reference with unknown DLC ownership
- **WHEN** a required game file is missing
- **THEN** the vehicle is refused and its `missing_dependencies` entries identify the missing reference, its kind and a `blocking` disposition
- **AND** `expected_dlc` is null and `dlc_origin` is `unknown` when the installed definitions do not establish which DLC supplies the missing data; the archive containing a reference is not presented as the missing DLC

#### Scenario: Orphan paint override does not prevent extraction
- **WHEN** a paint override references accessory ids with no corresponding installed definition and all defined painted content can be covered
- **THEN** the vehicle package is generated with the same templates as without those references
- **AND** `missing_dependencies` records each excluded id, referencing definitions and archives, an `ignored_reference` disposition and unknown DLC ownership rather than an asserted missing DLC

#### Scenario: Defined accessory is still required
- **WHEN** a paint override references an installed accessory definition whose painted geometry cannot be extracted
- **THEN** extraction refuses the vehicle rather than treating the definition as an orphan reference

#### Scenario: Accessory without livery material
- **WHEN** a paint override references an installed accessory whose selected model has no truckpaint material
- **THEN** the vehicle package is generated without a template for it, and `missing_dependencies` records it as `surface_without_livery_material` with an `excluded` disposition

## REMOVED Requirements

### Requirement: Model cache between batches
**Reason**: Models are read directly from the game archives in memory; there is no conversion step whose result needs keeping, so the 11 GB cache, its key and the converted/reused counters disappear.
**Migration**: None. The app is not released. Delete any leftover `target/game-extraction/cache` folder.
