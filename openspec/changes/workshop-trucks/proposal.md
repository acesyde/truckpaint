## Why

My Game turns the player's official trucks into packages, so they can paint them. A truck from the Steam Workshop (for example the Mercedes-Benz MP6 mod) is not in the game's archives, so the player cannot paint it: the only way today is Custom Vehicle, where they must supply the templates themselves.

A Workshop truck is a mod the game already knows how to read. TruckPaint can read it the same way it reads the game, and make the same package.

## What Changes

- **Workshop trucks appear in My Game.** TruckPaint finds the Workshop folder of the game by itself: it is in the same Steam library as the game folder (`steamapps/workshop/content/227300`, next to `steamapps/common`). It lists the trucks of the subscribed mods, with their names, and makes a package for each one the player checks, exactly like an official truck.
- **A mod is picked in three small steps, without opening big files.** For each mod folder it reads `versions.sii` and takes the package that fits the game's version, reads that package's `manifest.sii`, and only goes on when `category[]` has `truck`. Then it lists `def/vehicle/truck/<id>/data.sii` in the package. A truck whose game id the base game already defines (a retexture, an interior mod) is not a new truck and is not listed.
- **The three kinds of mod package are read.** A ZIP (the usual case, the MP6), a folder, and an `.scs` archive (HashFS, already read for the game). The package's files sit at its root, or under `base/` with conditional `dlc_*/` folders next to it (the MP6); a `dlc_*` folder is used only when the player owns that DLC.
- **The mod is read on top of the game.** A mod's definitions can point to models of the base game, so extraction mounts the game first and the mod after it: the last mounted wins, as in the game.
- **The package says where it comes from.** Its id is `mod.<game>.<workshop id>.<truck id>`, so two mods that define the same truck id do not collide, and it records the mod's name, author and version. The license is the author's: it says the content belongs to the mod's author and is for the player's own use.
- **One truck at a time.** Mod trucks are made strictly one after another, because reading a ZIP model unpacks it whole in memory (the MP6's main model is 132 MB). `tpv extract` keeps its parallel workers for the official vehicles only.
- **A mod whose trucks cannot be painted says why**, with the same refusals as an official truck (unsupported shader, unreadable model), naming the mod.

Non-goals:
- trailers: only `truck` mods are listed. Trailer mods are a later change;
- the game's `mod` folder (hand-placed `.scs` and `.zip` files, and the Proton path on Linux) and a Workshop folder setting in Preferences: the Workshop folder is only found from the game folder for now, so a Workshop in another library is not read;
- ATS: like My Game, only ETS2 for now;
- dependencies between mods, or a mod that needs another mod;
- changing how a package is edited or exported: a Workshop truck is an ordinary package once made.

## Capabilities

### New Capabilities

- `workshop-vehicles`: finding the trucks of Steam Workshop mods (the mod folders, the package that fits the game, the category and truck filter, trucks already in the base game left out), the package id and the license of a mod truck, and the My Game section that lists and makes them.

### Modified Capabilities

- `ets2-game-extraction`: archives are also read from ZIP files and folders, and from a package with a `base/` root and conditional `dlc_*` roots; extraction can mount a mod over the game; mod trucks are extracted one at a time.
- `vehicle-generation`: the Workshop trucks are shown in the My Game tab and made like the official ones, and a mod truck's package carries the mod's license instead of SCS's.

## Impact

- **Code:** `crates/tp-scs` (a ZIP reader and a folder reader behind `Mount`, root and DLC handling), `crates/tp-pack/src/extract` (a source made of the game and one mod, truck discovery in a mod, the package id, the single worker for mods), `crates/tp-app` (the My Game list, the license text).
- **Dependencies:** none new: the `zip` crate, with deflate, is already in the workspace.
- **Strings:** new Fluent messages in all four languages for the My Game section, the mod trucks' license and the refusal reasons.
- **Documentation:** the My Game page, in the four languages, and their media.
- **Checked on the author's machine:** the MP6 (`3815870196`) has 926 models, all `.pmd` version 4 and `.pmg` version 0x15, and its paint materials use the `truckpaint` families already supported; nothing in the geometry or shader readers needs to change.
- **Assumption, to check at implementation:** the rule for the `base/` root and the conditional `dlc_*` folders comes from how the game's mod format works, not from a spec read in this repository.
