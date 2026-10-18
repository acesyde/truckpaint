## Why

TruckPaint will no longer share vehicle packages built from the games. The templates belong to SCS Software and the DLC content must not be passed around. Each player will build the packages from their own game (see `native-game-reader`, `in-app-vehicle-generation` and `first-run`), so the Marketplace has no use.

## What Changes

- **BREAKING:** the **Marketplace** tab of the Vehicle Library is removed, with its install and update flow, cache and legal notice. The Vehicle Library keeps **Installed** and **My vehicles**.
- The **Catalog address** setting and the `TRUCKPAINT_CATALOG_URL` variable are removed. The app makes no network request at all.
- `tpv index` and `tpv index --check` are removed, with the catalog file format. Nothing builds a `catalog.json` any more.
- Packages made by `tpv extract` keep their license and authors but no longer point to the marketplace repository as their homepage.
- The documentation stops mentioning the Marketplace. Pages that sent players there now say to generate the package (`in-app-vehicle-generation` will later point them to the app).
- No migration: the app is not released, so no catalog cache or `catalog_url` preference needs cleaning.

Non-goals:
- the replacement (building packages in the app, game paths, first launch): see `native-game-reader`, `in-app-vehicle-generation`, `first-run`;
- changing how `.tpv` files are installed by hand.

## Capabilities

### Modified Capabilities
- `marketplace`: retired, every requirement removed.
- `marketplace-catalog`: retired, every requirement removed.
- `app-preferences`: the Marketplace address requirement is removed.
- `vehicle-authoring`: the catalog building requirement (`tpv index`) is removed.

## Impact

- **tp-app:** `marketplace.rs`, `ui/marketplace_tab.rs`, the Marketplace tab and its state, `AppDirs::catalog`, `Prefs.catalog_url`, the Preferences section, tests and doc media for it.
- **tp-vehicles:** the `catalog` module. **tp-pack:** `tpv index`, its tests and usage text; the extract homepage.
- **tp-ui:** icons used only by the Marketplace, if still unused.
- **Dependencies:** `ureq` and `sha2` leave the workspace.
- **tp-i18n:** `market-*`, `vehicles-tab-marketplace*`, `prefs-marketplace`, `prefs-catalog-url*` in en/fr/es/de.
- **Documentation:** library, official vehicles, first project and package format pages in four languages (package format: English only), the Marketplace screenshots, `CONTRIBUTING.md`.
- **Outside this repository (owner task):** archive `acesyde/truckpaint-marketplace` and its Pages site `truckpaint-marketplace.ace-dev.me`.
