## 1. Reading mod packages (`tp-scs`)

- [x] 1.1 Turn `Mount` into a list of layers (reader plus root prefix), with the existing HashFS archives as the first kind, and verify the existing `tp-scs` tests pass unchanged
- [x] 1.2 Add the folder layer (read, contains, walk by game path; no symbolic link out of the folder) and verify with tests on a temporary folder, including a link that leaves it
- [x] 1.3 Add the ZIP layer on the `zip` crate (list on open, entry read on demand, declared size checked) and verify with tests for stored and deflated entries, and that opening reads no entry
- [x] 1.4 Refuse unsafe ZIP names (absolute, `..`, backslash, NUL), encrypted entries, other compression methods and damaged tables, each with a message naming the package, and verify with a test per case
- [x] 1.5 Add the root prefixes (`""`, `base/`, `dlc_<name>/`) to layers and verify a test mounts a `base/` package so its files read as `/def/...` and a `dlc_` folder is left out when not asked for
- [x] 1.6 Verify "the last layer wins" with the game, the mod's main root and a mod DLC root holding the same path

## 2. Finding Workshop trucks (`tp-pack`)

- [x] 2.1 Derive the Workshop folder from a game folder inside `steamapps/common`, and verify tests for a library, a game outside a library and a library without the Workshop folder
- [x] 2.2 Parse `versions.sii` and choose the package (pattern match with `*`, first listed, the "latest" without versions as last resort, never informational), and verify with tests built from the real files seen: `content_161`, `latest` with older packages, informational only, a missing file
- [x] 2.3 Resolve a package name to a ZIP, an `.scs` or a folder, and verify with a test of the three forms
- [x] 2.4 Read `manifest.sii` (category, name, author, `package_version`, `dlc_dependencies`) and keep the `truck` mods, and verify with tests for `truck`, `trailer`, `ai_traffic`, two categories and no manifest
- [x] 2.5 List `def/vehicle/truck/<id>/data.sii` of a package through the layers (root or `base/`), drop ids the base game defines and invalid names, and verify with a test of a mod that redefines an official truck
- [x] 2.6 Choose the DLC layers from the game folder's `dlc_*.scs` and verify with tests for a DLC the player owns and one they do not
- [x] 2.7 Add the scan of the Workshop (all mods, one reason per unreadable mod, no model read) and verify with a test over several mods where one is damaged

## 3. Building a mod truck (`tp-pack`)

- [x] 3.1 Make `Sources` mount the game and one mod package, and verify with a test where the mod holds the truck and the game holds its model
- [x] 3.2 Give the package id and the previous-package lookup to the build instead of the hard-coded `official.<game>.<id>`, and verify the existing extraction tests pass unchanged
- [x] 3.3 Build the package of a mod truck (`mod.<game>.<workshop id>.<id>`, game path, description with the mod and its version, mod author, mod license) and verify with a synthetic mod that the manifest is valid and installable
- [x] 3.4 Name provenance by package and refuse a missing file or a missing DLC with the mod and the file or DLC named, and verify with two tests
- [x] 3.5 Make a mod update bump the template version like a game update, and verify with a test that builds the same mod truck twice with one changed template
- [x] 3.6 Give mod trucks one worker in the batch code and verify with a test that two mod trucks never run at the same time and that an official-only batch keeps its worker count

## 4. My Game tab (`tp-app`)

- [x] 4.1 Run the Workshop scan after the game scan in the scan job and send it as its own event, and verify with a headless test that the official list appears before the Workshop list
- [x] 4.2 Show the Workshop section (rows, mod name, status from the mod truck's package id, filters, search, Select All and Select None) and verify with `egui_kittest` tests on a list of two mods
- [x] 4.3 Show the empty states (Workshop not found, no truck mod, a mod that cannot be read with its reason) and verify with one test each
- [x] 4.4 Generate checked Workshop trucks in the same run as official ones (progress, cancel, failures) and verify with a headless test of a mixed run
- [x] 4.5 Set the license text and the notice for Workshop trucks and verify the Installed tab shows the mod's license for one
- [x] 4.6 Hide the Workshop section when the Kind filter is Trailers and verify with a test

## 5. Strings and documentation

- [x] 5.1 Add the new messages (section heading, empty states, mod line, notice, refusal reasons) to the four locales with the same ids, following the casing rules, and verify `mise run test` (the locale checks) passes, with German checked for fit
- [x] 5.2 Update the My Game page of the documentation in English, with `<Ui id="…" />` for every label, and verify the site builds (`mise run doc:build`)
- [x] 5.3 Translate the page into French, Spanish and German with the docs-translator agent and verify the files and `sidebar.order` match across languages
- [x] 5.4 Render the media of the My Game section again (`mise run doc:media`, needs a GPU) and verify the Workshop section is shown on a synthetic fixture, not on the author's mods

## 6. Check on a real Workshop

- [x] 6.1 Run the scan on a Workshop that holds the MP6 and verify `actros.mp6` and `mercedes.mp6` are listed from mod 3815870196, and that the mods of other categories are not
- [ ] 6.2 Generate both MP6 trucks and verify the packages install, are listed in New Project, and open with their templates; record the peak memory of the run
- [ ] 6.3 Export a project that uses an MP6 package and verify the mod's `game.path` is the truck's game id
- [x] 6.4 Run `mise run ci` through the rust-ui-tester agent and verify it passes, or state the skipped steps (GPU-only tests are ignored)
