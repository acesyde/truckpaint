## Context

See proposal.md for why. What exists today:

- `tp-scs` reads HashFS v2 archives (`Archive`) and mounts them in order (`Mount`: the last archive holding a path wins). Nothing else is readable.
- `tp-pack::extract::Sources::new` mounts a fixed list of the game's archives (base, `def`, `dlc_*`...). `scan` lists the trucks and owned trailers in `/def/vehicle/...`; `generate` builds one package; ids are `official.<game>.<vehicle>` (hard-coded in the id and in the lookup of the previous package).
- The app (`tp-app::game`) finds the game folder in Steam libraries, scans it off the UI thread, and generates the checked vehicles strictly one after another.
- `tp-vehicles::is_valid_id` accepts lowercase words separated by dots, digits included, so `mod.ets2.3815870196.actros.mp6` is a valid id.

Checked on a real Workshop (16 mods, the author's machine):

- A mod folder holds `versions.sii` and one or more packages, each a ZIP, an `.scs` (HashFS) or a folder, named by `package_name` (`content_161`, `latest`, `universal`...). Informational packages carry no content.
- The MP6 (`3815870196`) is one ZIP of 11 783 files under `base/`, with `dlc_phys_flags/`, `dlc_toys/`, `dlc_krone/` and `dlc_actros_tuning/` beside it. All its 926 `.pmd` are version 4 and its `.pmg` version 0x15. Its paint materials are `truckpaint` families the extractor supports, in both `.mat` syntaxes (`material :` and `effect :`).
- A trailer mod (`3806633402`) ships only definitions and points to models of the base game. A truck mod can do the same, so a mod is always read over the game.
- Two mods redefine official truck ids (`daf.2021`, `man.tgx`...); they are not new vehicles.

## Goals / Non-Goals

**Goals:**
- Reading a Workshop truck is the same extraction as an official truck; only the sources differ.
- Scanning stays cheap: manifests and file tables only, never a model.
- A mod's problems (damaged, missing DLC, unsupported shader) name the mod and never stop the others.

**Non-Goals:**
- Trailers, ATS, the game's `mod` folder, a Workshop folder setting, mods that need other mods (see proposal.md).
- Changing the readers of models, materials or shaders (checked: nothing to change for the MP6).
- A "Mod updated" status. Regenerating a truck picks up a new mod version; a status that notices it can come later.
- Mod support in the `tpv extract` command. It keeps excluding user mods.

## Decisions

### 1. One mount of "layers", each with its own kind of reader

`Mount` today holds `Vec<Archive>`. It becomes a list of layers, each a reader plus a root prefix: `HashFs`, `Zip` (the `zip` crate, already in the workspace) or `Dir`. Every layer answers the same four questions as `Archive` (contains, read, walk, name). A layer's root prefix (`""` or `base/` or `dlc_x/`) is added when it looks a path up and removed from the paths it lists, so everything above `Mount` sees game paths.

Alternatives considered: a trait object per reader (more flexible, but there are exactly three readers and no plugin); extracting a ZIP to a cache folder and reading it as a folder (no new reader, but a large cache for a 132 MB model, and a cleanup to get wrong). The enum keeps the change inside `tp-scs`.

The ZIP reader reads the central directory when opened, and an entry only when `read` is called, so opening a package is cheap. Names are normalized and checked on open (absolute, `..`, backslash, NUL are refused); an entry is unpacked whole into a `Vec` with its declared size checked, as `Archive::read` does.

### 2. Mod roots are layers, not renames

A package gets one layer for its main root and one layer per `dlc_<name>/` folder it holds and the player owns. The main root is `""` when `def/` is at the top of the package, else `base/`. Layers are mounted after the game, DLC layers after the main one. A game path is therefore looked up in the mod's layers first and in the game second, which is the order the game uses. This is the one rule taken from how the game reads mods rather than from a file in this repository; it is checked against the MP6 and the trailer mod, and a package with neither root has nothing to mount, so it adds no truck.

### 3. Discovery is a separate small module, in `tp-pack`

`workshop` (in `tp-pack::extract`) does what the proposal lists: list the folders of the Workshop, read `versions.sii` with the existing SII parser (`syntax.rs`), pick the package, read `manifest.sii`, filter by category, list `def/vehicle/truck/*/data.sii` through the layers, remove ids the base game defines. The version match is a plain pattern with `*` as the only wildcard (the manifests use `1.61.*`); it is a few lines and gets its own tests, because a wrong match silently drops a mod.

The base game's ids come from the `Scan` the app already makes, so the Workshop scan does not read the game twice.

The result is a list of mod trucks: mod id, mod name and package version, author, package location, truck id, name, brand. It is built without reading any model.

### 4. Building a mod truck reuses the build, with a mod-aware source

`Sources` gets a constructor that takes the game folder and an optional mod package, and mounts the layers after the game's. `Build` is unchanged except for the places that hard-code the official id: the package id and the lookup of the previous package are given by the caller (`official.<game>.<id>` or `mod.<game>.<workshop id>.<id>`). The provenance already names the archive a file came from; for a layer it names the package.

Names: the localization is read through the same mount, so the mod's own `locale` files, if it has some, are found without a special case.

The package version follows the game exactly like an official truck (first three numbers of the game's version), because the existing rule already does what a mod needs: the same version with changed templates takes the next patch, and unchanged templates keep their layout version. The mod's own `package_version` is not a SemVer (the MP6 says `2.2.00`), so it is only recorded in the description.

### 5. One mod truck at a time

The application generates one vehicle at a time already, so nothing changes there. The cap matters for the library function that builds a batch (`run`, used by `tpv extract`): a batch that includes mod trucks must give them one worker. `tpv extract` does not extract mods in this change, so this is a guard in the batch code, with a test, and not something a player sees. The cost of the choice is time on a mod with several trucks; the MP6 has two.

### 6. The Workshop folder comes from the game folder

`<library>/steamapps/common/<game>` gives `<library>/steamapps/workshop/content/227300`. It needs no new setting and no change to Preferences. A game folder outside a `steamapps/common` folder, or a library without that Workshop folder, shows "could not be found" in the tab. A setting in Preferences is the obvious next step and does not change this design.

### 7. The tab: a Workshop section in My Game

The scan job already runs off the UI thread. It runs the Workshop scan after the game scan and sends the list as another event, so the official vehicles appear first and the tab does not wait for the mods. Rows, checkboxes, filters, statuses and Generate reuse the official ones; the row carries the mod's name, and the status is looked up with the mod truck's package id.

## Risks / Trade-offs

- [Memory: a ZIP entry is unpacked whole, and the MP6's main model is 132 MB] → one mod truck at a time (decision 5), and the entry is dropped as soon as the model is parsed. The validation task measures the peak on the MP6; if it is too high, reading a model in chunks is a later change.
- [The category is only a hint, so a truck mod with a wrong category is missed, and a non-truck mod with `truck` is examined for nothing] → the truck definitions decide what is listed; a mod with the wrong category is not offered, which the player can only fix by asking the author.
- [The `versions.sii` rule for the mod's "latest" package (no `compatible_versions`) is inferred from the files seen, not from a specification] → it is the last choice, after a real match, and it is covered by a test on the sample files; a wrong guess shows as an unsupported-file refusal, never as a wrong package silently.
- [A mod that overrides a game file used by an official truck] → each truck is built from the game plus its own mod only (spec: Two mods do not mix), so no other vehicle changes.
- [A mod truck needs a DLC the player lacks] → the truck is listed; building it fails with the DLC named. Listing it keeps the list the same on every machine, and the refusal tells the player what to do.
- [Workshop in a library other than the game's] → not read; named as a limit in the tab. A Preferences setting fixes it later.
- [The truck's `data.sii` names parts that exist only in a mod the player has not subscribed to] → "missing game file" refusal naming the file, as for an official vehicle.

## Open Questions

- Should the tab say when Steam updated a mod after its truck was made (a "Mod updated" status)? It does not change what is built, only what is shown.
