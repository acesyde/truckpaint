# ets2-game-extraction Specification

## Purpose

Generate autonomous official ETS2 truck packages from an installed game's definitions and models, with explicit coverage checks and traceable batch results.

## Requirements

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

### Requirement: Game names
Truck, trailer, brand and cabin names SHALL come from the installed game's English localization (`/locale/common.sui` for brands and series, `/locale/en_us/accessories.sui` for cabins), looked up from the `@@key@@` values of the truck, trailer and cabin definitions. A trailer whose brand is empty (SCS's own trailers) SHALL be named "SCS" followed by its game series id in readable form, words capitalized and underscores as spaces (`scs.box` → "SCS Box", `scs.fueltank` → "SCS Fueltank"), since the game's series strings for them are abbreviations. A truck's optional short series (fifth `info[]` value) SHALL be appended when the series does not already contain it, so trucks sharing a series stay distinct. Comments after a translated value SHALL be ignored. Cabins of one truck sharing a translated name (such as 8x4 versions) SHALL have their game cabin id appended in parentheses. Plain values are used as written; a key without a translation SHALL fall back to a readable form of the key. The report SHALL identify the localization files and their archives.

#### Scenario: Trucks sharing a series
- **WHEN** `iveco.hiway` declares the Stralis series with the Hi-Way short series
- **THEN** its package is named "Iveco Stralis Hi-Way", distinct from `iveco.stralis` ("Iveco Stralis")

#### Scenario: SCS trailer without a brand
- **WHEN** `scs.box` declares an empty brand and a series translated as "STD"
- **THEN** its package is named "SCS Box"

#### Scenario: Branded trailer
- **WHEN** `krone.coolliner` declares the Krone brand and the Cool Liner series
- **THEN** its package is named "Krone Cool Liner"

### Requirement: Definition-driven selection
The extractor SHALL derive vehicle metadata, cabins, accessory game ids, model references and compatibility from game definitions rather than per-vehicle profiles. It SHALL resolve referenced inclusions and archive overrides with recorded provenance. It SHALL select each model's requested look and variant, retain only visible painted faces, and choose the UV stream by its semantic alias. Missing required files, inclusion cycles, conflicting unresolved definitions, unsupported paint shaders or UV mappings, and ambiguous looks, variants or UV streams SHALL refuse the affected vehicle. A look or variant repeated under the same name with identical content (a game data duplicate) is not ambiguous: it SHALL be used once. Unsupported data SHALL NOT be silently omitted. Paint overrides referencing accessory ids with no definition in the installed source view SHALL be excluded from texture assignments and reported as ignored references, without refusing the vehicle. A defined accessory whose painted geometry cannot be extracted (missing files, invalid geometry, unsupported paint shaders) SHALL still refuse the vehicle. A defined accessory whose definition has no exterior model field at all (for example a container body whose model is commented out) has nothing to paint: it SHALL be excluded from all templates and reported as `accessory_without_model` with an `excluded` disposition, without refusing the vehicle. A defined accessory whose selected looks/variants contain no truckpaint material (for example a base-colour-only `paint` flavor) cannot show a livery: it SHALL be excluded from all templates and reported with an `excluded` disposition, without refusing the vehicle.

#### Scenario: Included shared model
- **WHEN** an accessory definition includes a file that references a shared model outside the truck's model directory
- **THEN** the referenced definition and model contribute to the vehicle and their source is recorded

#### Scenario: Visible variant and semantic UV alias
- **WHEN** a look contains painted materials and a variant hides one model part, with the paint UV alias on a stream whose numeric tag differs
- **THEN** the template contains only visible painted geometry using the aliased stream, without an additional vertical flip

#### Scenario: Ambiguous or unsupported selection
- **WHEN** a definition has an unresolved include or its painted model has ambiguous look, variant or UV data, or unsupported UV data
- **THEN** the vehicle fails with the concerned definition or model identified

#### Scenario: Look repeated with identical content
- **WHEN** a painted model defines the requested look twice with identical content
- **THEN** the extractor uses that look once and does not refuse the vehicle
#### Scenario: Body without any model
- **WHEN** a paint override targets a trailer body whose definition has no model field
- **THEN** the trailer package is generated without it and `missing_dependencies` records it as `accessory_without_model` with an `excluded` disposition

### Requirement: Shared shader and UV rules
The extractor SHALL recognize the `eut2.truckpaint`, `eut2.truckpaint.tsnmap` and `eut2.truckpaint.tsnmapuv` families with optional `airbrush`, `flipflake` and `altuv` flavors and optional `.a` or `.asafew.a` alpha-test suffixes, without vehicle-specific profiles. Alpha-tested material geometry SHALL contribute its UV wireframe without attempting pixel-level opacity-mask clipping. Other flavors and conflicting normal-map flavors SHALL remain unsupported. Every painted material, cabins and accessories alike and including `altuv` flavors, SHALL use the ordinary paint UVs (`_TEXCOORD1`), the layout of the game's colour paint jobs and of the community templates. Packages SHALL record `game.alt_uv: false`, so mod export never sets `alternate_uvset`. Missing or ambiguous `_TEXCOORD1` aliases and invalid selected geometry SHALL refuse the vehicle. Reports SHALL identify selected shaders, UV aliases, coordinate bounds and out-of-bounds vertex counts. Finite coordinates outside [0, 1] SHALL be preserved without wrapping, coordinate clamping or automatic UV-channel changes; the SVG viewport SHALL bound the visible wireframe. Non-finite coordinates and coordinates that cannot be represented by SVG edge quantization SHALL refuse the vehicle. Definition-driven exterior model resolution SHALL include `detail_model`, including UK variants reusing that model.

#### Scenario: Ordinary family without normal maps
- **WHEN** a visible painted face uses `eut2.truckpaint`
- **THEN** extraction selects `_TEXCOORD1` by alias and writes `game.alt_uv: false`

#### Scenario: Alternate-UV materials use the ordinary layout
- **WHEN** cabin paint materials use an `altuv` flavor whose `_TEXCOORD2` differs from `_TEXCOORD1`
- **THEN** all painted contributions use `_TEXCOORD1` and the package records `game.alt_uv: false`

#### Scenario: Missing selected mapping
- **WHEN** a painted material has no `_TEXCOORD1` alias or several streams carry it
- **THEN** extraction refuses the vehicle rather than falling back to another UV stream

#### Scenario: Metallic paint flavor
- **WHEN** a selected painted material uses `flipflake`, optionally with airbrush, a normal map and alternate UVs
- **THEN** its geometry contributes using the same semantic UV rules, while duplicate or unknown flavors remain invalid

#### Scenario: Finite out-of-bounds coordinates
- **WHEN** selected painted geometry has finite representable UV coordinates outside [0, 1]
- **THEN** its original coordinates and selected alias are retained, its bounds and overflow count are reported, and no wrapping or alternate-channel fallback is applied

#### Scenario: Detailed cabin model
- **WHEN** a cabin references `detail_model` and a UK variant reuses it
- **THEN** both selected variants contribute to its template with recorded provenance

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

### Requirement: Layout versions across batches
Without `--previous`, every texture SHALL have layout version 1. With `--previous <batch directory>`, each texture SHALL keep the layout version of the same texture in the newest package of the same id found in that directory when its template is byte-identical, and SHALL get that version plus one when its template differs; a texture absent from the previous package SHALL get layout version 1. An unreadable or invalid previous package SHALL stop the batch before any vehicle is processed. The report SHALL record the previous batch directory.

#### Scenario: A game update moves a texture's UVs
- **WHEN** a batch runs with `--previous` pointing at a batch whose package has the same cabin at layout version 1 with a different template
- **THEN** the new package records layout version 2 for that cabin, and unchanged textures keep their version

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

### Requirement: Trailer packages
Each successful owned trailer SHALL produce one `.tpv` with id `official.ets2.<trailer>`, `kind: "trailer"`, its game path, and the same validation, size rules, UV rules, exclusions, layout versions and report entries as trucks. A trailer package SHALL have exactly one main texture with no cabin ids. When at least one painted body, chassis or accessory has no independent texture, the main texture SHALL be 4096 × 4096 and its template SHALL superpose all those painted options. When every painted part has its own texture, the main texture SHALL be 64 × 64 with a blank template: it only carries the trailer's base colour, as the game's fill mask does. Accessory textures SHALL be shared by the whole trailer; trailers SHALL have no variants. Accessory ids that use the same sized mask in every game paint job that overrides any of them SHALL form one accessory texture listing all those game ids, whose size follows the size rules applied to that mask and whose template superposes their layouts; ids whose masks differ in at least one paint job SHALL keep separate textures. A trailer whose paint jobs define no main mask and no override (only plain, possibly locked, colours) cannot carry a livery: it SHALL be reported as `skipped` with a `not_paintable` reason, SHALL produce no package and SHALL NOT count as a failure. The report SHALL record each vehicle's kind and, for every grouped texture, its game ids and the shared mask references.

#### Scenario: Bodies sharing one livery region
- **WHEN** the dry van, insulated and reefer 13.6 m bodies of `scs.box` use the same 4096×2048 mask in every paint job that overrides them
- **THEN** the package has one accessory texture of 4096 × 2048 listing the three body ids, whose template shows the three layouts superposed

#### Scenario: Ids that differ in one paint job
- **WHEN** two body ids share a mask in most paint jobs but one paint job gives them different masks
- **THEN** they keep separate accessory textures

#### Scenario: Trailer main texture
- **WHEN** a trailer's chassis has no paint override with crop dimensions
- **THEN** the chassis contributes to the single 4096 × 4096 main texture, which has no cabin ids

#### Scenario: Every part has its own region
- **WHEN** every painted body, chassis and accessory of a trailer has an independent texture
- **THEN** the package has a 64 × 64 main texture with a blank template and no cabin ids

#### Scenario: Colour-only trailer
- **WHEN** a trailer's paint jobs only set base colours, with no mask and no override
- **THEN** it is reported as skipped and not paintable, no package is written, and the batch can still succeed

#### Scenario: Install and export an extracted trailer
- **WHEN** the player installs `official.ets2.krone.coolliner` and exports a mod painting its main texture and body
- **THEN** the mod writes the trailer paint job under `trailer_owned` with one override for the body texture listing its game ids

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
