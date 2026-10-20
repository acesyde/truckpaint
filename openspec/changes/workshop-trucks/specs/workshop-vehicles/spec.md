## Purpose

Lets a player paint a truck from a Steam Workshop mod the way they paint an official truck: TruckPaint finds the mod's trucks in the Workshop folder of the player's game, and makes a package from them on the player's computer, for their own use.

## ADDED Requirements

### Requirement: Workshop folder
The application SHALL look for the Workshop folder of an ETS2 game folder in the Steam library that holds the game: the folder `steamapps/workshop/content/227300` of the library whose `steamapps/common` holds the game folder. The folder SHALL be used only when it exists. Each folder directly inside it is one mod, named by its Workshop id. The application SHALL NOT write in it. When the game folder is not inside such a library, or the Workshop folder does not exist (no mod subscribed), no Workshop truck SHALL be listed and the My Game tab SHALL say so, without an error.

#### Scenario: Workshop next to the game
- **WHEN** the ETS2 game folder is `/games/SteamLibrary/steamapps/common/Euro Truck Simulator 2` and `/games/SteamLibrary/steamapps/workshop/content/227300` exists
- **THEN** that folder is the Workshop folder

#### Scenario: Game outside a Steam library
- **WHEN** the game folder is not below a `steamapps/common` folder
- **THEN** no Workshop folder is used and the My Game tab says the Workshop trucks cannot be found

#### Scenario: No subscription
- **WHEN** the library has no `workshop/content/227300` folder
- **THEN** no Workshop truck is listed and nothing is reported as failed

### Requirement: Package of a mod that fits the game
For each mod, the application SHALL read its `versions.sii` and use the one package that fits the game's version:
- a package whose `compatible_versions` has a pattern matching the game's version (`1.61.*` matches 1.61.1.9; `*` stands for any text) is a fit; when several fit, the first listed is used;
- when none fits, the package that is not informational and has no `compatible_versions` (the mod's "latest") is used;
- a package marked `informational` SHALL NOT be used;
- when none is usable, the mod SHALL be left out of the list with its folder named in the scan's details.

A package is named by its `package_name`, and is the ZIP archive (`<name>.zip`), the `.scs` archive (`<name>.scs`) or the folder (`<name>`) of that name in the mod folder. A mod without a `versions.sii`, and a package that is missing, SHALL be left out the same way. The application SHALL NOT use a mod's older packages for a newer game.

#### Scenario: Package for the game's version
- **WHEN** a mod lists `content_160` for `1.60.*` and `content_161` for `1.61.*`, and the game is 1.61.1.9
- **THEN** `content_161` is used

#### Scenario: The mod's latest package
- **WHEN** a mod lists `latest` with no compatible versions and `156_content` for `1.56.*`, and the game is 1.61.1.9
- **THEN** `latest` is used

#### Scenario: Informational package only
- **WHEN** the only package of a mod is marked `informational`
- **THEN** the mod is left out of the list

#### Scenario: Three forms of the same package
- **WHEN** a package named `content_161` is a ZIP archive in one mod, an `.scs` archive in a second and a folder in a third
- **THEN** each of them is read

### Requirement: Truck mods and their trucks
A mod SHALL be examined for trucks only when the `manifest.sii` at the root of its package has the category `truck` in `category[]` (any case). Its trucks are the folders `<root>/def/vehicle/truck/<id>/` that have a `data.sii`, whose id is a valid game name. A truck whose id is also defined by the base game (the game's own archives, DLC included) is a change to an existing truck, not a new one, and SHALL NOT be listed. A mod whose category is not `truck`, with no trucks left, SHALL NOT be listed. Because the category is only a hint from the mod's author, the presence of the truck definition decides what is listed. Finding the trucks SHALL read only the manifest and the file table of the package, never the models, so scanning many mods stays quick.

#### Scenario: A truck mod
- **WHEN** a mod's category is `truck` and its package defines `actros.mp6` and `mercedes.mp6`, which the game does not
- **THEN** both are listed, from that mod

#### Scenario: Another category
- **WHEN** a mod's category is `ai_traffic`, `ui` or `tuning_parts`
- **THEN** it is not examined and none of its vehicles is listed

#### Scenario: A mod that changes an official truck
- **WHEN** a mod of category `truck` defines `daf.2021`, which the game defines too
- **THEN** that truck is not listed

#### Scenario: Category without trucks
- **WHEN** a mod's category is `truck` but its package has no truck definition
- **THEN** it is not listed and nothing is reported as failed

#### Scenario: Trailers are not listed
- **WHEN** a mod's category is `trailer`
- **THEN** it is not examined

### Requirement: Files of a mod package
A mod package SHALL be read as the game reads it: its files are at its root, or under a `base/` folder at its root when the package has no `def/` at its root. A folder `dlc_<name>/` at the root of the package SHALL be read only when the player's game folder holds that DLC (`dlc_<name>.scs`), and SHALL be left out otherwise. The package's files are mounted after the game's own archives, so that a path the mod holds wins and a path it does not hold is read from the game. A truck whose files point to a path that neither holds SHALL be refused, with that path and the mod named.

#### Scenario: Files under base
- **WHEN** a package has `base/def/vehicle/truck/actros.mp6/data.sii` and no `def/` at its root
- **THEN** the truck is found as `/def/vehicle/truck/actros.mp6/data.sii`

#### Scenario: Files at the root
- **WHEN** a package has `def/vehicle/truck/x/data.sii` at its root
- **THEN** the truck is found as `/def/vehicle/truck/x/data.sii`

#### Scenario: DLC folder of a DLC the player owns
- **WHEN** a package has a `dlc_toys/` folder and the game folder holds `dlc_toys.scs`
- **THEN** the files of `dlc_toys/` are mounted with the package

#### Scenario: DLC folder of a DLC the player does not own
- **WHEN** a package has a `dlc_krone/` folder and the game folder has no `dlc_krone.scs`
- **THEN** the files of `dlc_krone/` are not used

#### Scenario: Model of the base game
- **WHEN** a mod's definitions use a model that only the base game holds
- **THEN** the model is read from the base game

#### Scenario: Missing file
- **WHEN** a truck of the mod needs a file that neither the mod nor the game holds
- **THEN** that truck is refused, with the file and the mod named, and the mod's other trucks are made

### Requirement: Mod truck packages
A mod truck's package SHALL be made by the same rules as an official truck (cabins, accessories, shaders, UV streams, texture sizes, coverage, refusals) and SHALL pass the same validation. Its id SHALL be `mod.<game>.<workshop id>.<truck id>` (`mod.ets2.3815870196.actros.mp6`), so that two mods with the same truck id do not collide and the package of one mod never replaces another's. Its `game.path` SHALL be the truck's game id, so that Export Mod writes the same truck to the game. Its version and its supported game versions SHALL follow the game as the vehicle-generation capability says for official packages, so a mod update that changes a template gives the next template version like a game update does. Its name and brand SHALL come from the game's English localization, completed by the mod's own when it holds one, and a name without a translation SHALL fall back to a readable form of the key. Its description SHALL name the mod (its Workshop id and the `package_version` of its manifest) and its authors SHALL include the mod's `author`. Its license SHALL be "Personal modding only, not for sale. Content belongs to the mod's author." Mod truck packages SHALL stay on the player's computer, like official ones.

#### Scenario: Id of a mod truck
- **WHEN** truck `actros.mp6` of Workshop mod 3815870196 is made
- **THEN** its package id is `mod.ets2.3815870196.actros.mp6` and its game path is `actros.mp6`

#### Scenario: Two mods, one truck id
- **WHEN** mods 111 and 222 both define a truck `foo.bar` that the game does not
- **THEN** both are listed and made, as `mod.ets2.111.foo.bar` and `mod.ets2.222.foo.bar`

#### Scenario: Mod updated
- **WHEN** a mod truck was made, the mod then changes the template of one texture, and the truck is made again
- **THEN** the package takes the next version, that texture takes the next template version, and the first package stays installed

#### Scenario: License and authors
- **WHEN** the details of a mod truck's package are opened in the Installed tab
- **THEN** the license names the mod's author, not SCS, and the authors include the mod's `author`

#### Scenario: Refused truck
- **WHEN** a mod truck uses a paint shader the extraction does not support
- **THEN** it is refused with the shader and the mod named, and the mod's other trucks are made

### Requirement: Mod truck dependencies
When the manifest of the package names `dlc_dependencies` that the player's game folder does not hold, the mod's trucks SHALL still be listed, and a truck that cannot be made for lack of those files SHALL be refused with a reason that names the missing DLC.

#### Scenario: Missing DLC
- **WHEN** a mod needs `dlc_daf_2021`, the player does not own it, and a truck of the mod uses its files
- **THEN** the truck fails with a message naming `dlc_daf_2021`
