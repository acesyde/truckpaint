## Why

Community vehicle packages (`.tpv`) can only be installed by hand today: the player must find a file, download it and drop it in the Vehicle Library. A new player has no way to know what exists, and a published vehicle has no way to reach players or to announce a new version. `first-run` and `template-update-impact` both assume a catalog the app can browse and install from.

## What Changes

- **A public catalog,** kept in its own repository (`acesyde/truckpaint-marketplace`). Each `.tpv` is an asset of a GitHub Release (no Git LFS, no submodule in this repository). A single `catalog.json` lists every published package: id, game, kind, name, brand, version, size, SHA-256, download address, minimum app version, game version, preview.
- **`tpv index`,** a new command that builds `catalog.json` from a folder of `.tpv` files, and `tpv index --check` that fails when a package is invalid, a published version changed, or a version does not go up. The marketplace repository's CI uses it on every pull request and on release.
- **A published version never changes.** A fix is a new version. The catalog keeps older versions listed so a project can still get the one it uses.
- **The app browses the catalog:** Vehicle › Vehicle Library… gets a **Marketplace** tab (search, game and kind filters, size, **Install** / **Update** / **Installed**). The catalog is downloaded when the tab opens, cached on disk, and used from the cache when offline.
- **Install checks before writing:** the download must match the catalog's size and SHA-256, then goes through the same validation as any package. Nothing is installed otherwise.
- **The catalog address can be changed:** in Preferences, or with the environment variable `TRUCKPAINT_CATALOG_URL` (which wins), for testing a catalog before publishing it.
- **A legal notice:** the Marketplace tab, the catalog repository's README and each game-derived package say that the game's content belongs to SCS Software, that TruckPaint and the catalog are unofficial and not endorsed by SCS, and that the content is shared only for personal modding, never for sale.
- **No background traffic:** the app only contacts the catalog when the player opens the Marketplace tab or presses Refresh. It sends no identifier.

Non-goals:
- uploading from the app (publishing is a pull request on the marketplace repository);
- ratings, comments, accounts, payments;
- signing packages (the SHA-256 protects the download, not the publisher; revisit if the catalog becomes open to anyone);
- the first-launch flow (`first-run`) and the update summary (`template-update-impact`), which use this change;
- automatic updates.

Open questions:
- **Redistribution rights.** Packages built from the game contain templates owned by SCS Software, and similar content is already shared on SCS's official forums. The Marketplace therefore shows a legal notice (below), and the catalog keeps `license` and a `notice` on each entry. Confirming that SCS's modding policy allows this sharing stays with the repository owner; the notice does not grant any right by itself.
- Whether GitHub Pages or the repository's raw `catalog.json` is the default address. Proposed: Pages (stable address, CORS, CDN); the address is a setting, so it can change without a release.
- How many versions of a vehicle the catalog keeps. Proposed: all of them.

## Capabilities

### New Capabilities
- `marketplace`: the Marketplace tab, listing, filters, install and update, cache and offline behavior, address setting, what is sent and when.
- `marketplace-catalog`: the format of `catalog.json`, the rules every published package follows (immutable versions, release assets, integrity), and the repository's CI checks.

### Modified Capabilities
- `vehicle-authoring`: `tpv index` and `tpv index --check`.
- `app-preferences`: the catalog address setting.

## Impact

- **New repository content** (outside this repo): `acesyde/truckpaint-marketplace` holds `catalog.json` (generated), a validation workflow and a release workflow. No `.tpv` in git.
- **tp-pack:** `tpv index`; the catalog types are shared with the app (in `tp-vehicles`).
- **tp-vehicles:** catalog types, parsing and the choice of the newest compatible version; unit-tested.
- **tp-app:**
  - a catalog client (download, cache, SHA-256, size limit, timeout) running off the UI thread;
  - the Marketplace tab of the Vehicle Library;
  - `prefs.rs`: the optional catalog address;
  - installation reuses `install_bytes`, so validation is unchanged.
- **tp-i18n:** the tab's strings in en/fr/es/de.
- **Dependencies:** an HTTPS client and a SHA-256 implementation are not in the workspace today; both are added (see design).
- **Documentation:** a page on installing vehicles from the Marketplace and one for authors on publishing, in four languages.
- **No compatibility work:** the app is not released, so there is no earlier catalog format or install path to keep.
