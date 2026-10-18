## Context

Plain deletion. The Marketplace is self-contained: its tab (`ui/marketplace_tab.rs`), client (`marketplace.rs`) and state (`AppState.marketplace`) sit beside the Vehicle Library, which installs through `install_bytes` and keeps working. See `proposal.md`.

## Goals / Non-Goals

**Goals:**
- Leave no dead code, string, setting, dependency or doc page about the Marketplace.
- `ureq` removed: the app makes no network request.

**Non-Goals:**
- Anything about generating packages in the app.

## Decisions

- **Retire both capabilities** (`marketplace`, `marketplace-catalog`) with REMOVED requirements for each; archiving removes their specs.
- **Remove the `tp-vehicles::catalog` module and `tpv index` together:** only they and the app client use the catalog types. `tp-vehicles`/`tp-pack` keep `semver` only if still used elsewhere (check on build).
- **`sha2` leaves the workspace** if nothing else uses it after `tpv index` goes (the Marketplace client and `index.rs` are the only users found); `cargo build` decides.
- **Icons `DOWNLOAD`, `OFFLINE`, `INFO`** (tp-ui, documented as Marketplace icons) are removed if unused after the tab goes; used elsewhere, they stay.
- **`tpv extract` homepage:** drop the `homepage` field (it is optional in the manifest, as `pack`/custom packages show); license and authors stay.
- **Docs:** pages that pointed to the Marketplace say to generate the package with `tpv extract` as before the Marketplace; `in-app-vehicle-generation` rewrites them later. The library page goes from three tabs to two.
- **Old preference key:** `catalog_url` is dropped from `Prefs`; unknown keys in an existing file are ignored (check serde does not use `deny_unknown_fields`).

## Risks / Trade-offs

- Players have no download route until `in-app-vehicle-generation` ships → accepted; the app is not released, and `tpv extract` still works.
- Documentation translations drift → update all four languages in the same change (`docs-drift-checker` before archiving).
