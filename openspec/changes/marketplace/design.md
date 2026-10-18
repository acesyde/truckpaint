## Context

Packages are installed from bytes (`install_bytes` in `tp-app/src/vehicles.rs`), which validates fully and refuses on error; every install path goes through it. `tp-vehicles` already owns the manifest and `diff`; `tp-pack` owns the `tpv` command. The workspace has no HTTP client and no SHA-256 crate. The app is not released, so there is nothing earlier to stay compatible with. See proposal.md for motivation.

## Goals / Non-Goals

**Goals:**
- One static file and plain git files: no server code to run.
- The same validation for a downloaded package as for a dropped file.
- The catalog format shared by the app and `tpv index`, so they can't drift.

**Non-Goals:**
- Package signing, accounts, upload from the app, delta downloads, mirrors.
- Generating packages in CI. Game-derived `.tpv` files are built on the maintainer's computer with `tpv extract` (the game and ConverterPIX are needed) and committed; CI only validates them and builds the catalog.

## Decisions

**Hosting: `.tpv` files committed in the catalog repository under `packages/`; `catalog.json` generated and published on GitHub Pages.** Git history versions every package, and there is one place to review. Not Git LFS: raw URLs return the pointer file, and the free bandwidth quota runs out. Not release assets: a second place to keep in step with the pull request. Not a submodule: the app reads the catalog at run time, not at build time. Each entry's `url` is a `raw.githubusercontent.com` address pinned to the commit that published the catalog (given to `tpv index` as `--base-url`), never to `main`; versions are immutable, so the bytes behind a SHA-256 stay the same. Pages gives a stable address, a CDN and CORS. The address is a setting, so moving it later costs no release. The default address is one constant in the app.

**Catalog is built, not committed.** A workflow on every merge to `main` runs `tpv index` and deploys the file to Pages. No bot commits, so no loops and no conflicts between pull requests, and the catalog can't lag behind the packages. Pull requests run `tpv index --check` only.

**One catalog, all versions, immutable.** A flat `packages` list, one entry per `(id, version)`. Immutability is checked by `tpv index --check` against the committed catalog (an existing version's SHA-256 must not change; it compares with the catalog currently published on Pages). The app picks the newest version with `min_app <= app version`; older ones stay available in the detail pane, which serves projects pinned to an earlier layout (`template-update-impact`).

**Catalog types live in `tp-vehicles`,** next to `Manifest`: `Catalog`, `Entry`, parsing, and `newest_compatible`. `tp-pack` builds it, `tp-app` reads it. Pure code, unit-tested, no I/O.

**`tpv index` reuses `tpv check`'s validation** and fills the entry from the manifest, not from file names. Output is sorted and has no timestamp, so it is byte-identical for the same input. `min_app` comes from the package format (format 1 → 0.1.0), not from the `tpv` that indexed it, so a new `tpv` doesn't hide old entries from old apps. Game-derived packages have no `license`; the publish workflow passes `--default-license` and `--default-notice`. Immutability is checked with `--previous`, the catalog downloaded from Pages.

**Dependencies: `ureq` (blocking, rustls) and `sha2`.** The app already runs work off the UI thread; a blocking client in a worker keeps that pattern and avoids pulling an async runtime into an egui app. Alternative `reqwest`: heavier, async, no benefit here. Both are added to the workspace and checked for license and build time on all three desktop targets before merging.

**Download rules.** HTTPS only (or `file://` for the override, to test a catalog locally). The body is read with a size cap (the entry's `size`, and a hard maximum), a connect and a total timeout. The SHA-256 is computed over the bytes and compared to the entry before `install_bytes` is called. A mismatch or a failed install writes nothing.

**Cache.** `catalog.json` is saved in the app's cache folder after it parses and has a known `schema`, via a temporary file and rename, so a bad response or a crash never leaves a broken cache. The tab shows the cache's date when offline. No conditional requests at first; the file is small.

**Address precedence:** `TRUCKPAINT_CATALOG_URL`, then the preference, then the built-in address.

**No telemetry.** Requests carry a plain `User-Agent` with the app version only.

**UI.** The tab is a third tab of the existing Vehicle Library dialog and reuses its list, filters and detail pane. Install state is derived by comparing catalog entries with installed manifests, so it can't disagree with the Installed tab. `first-run` later reuses the same catalog client and list model, without the dialog.

## Risks / Trade-offs

- **Redistribution of SCS assets** → catalog entries need `license`, and the repository only accepts packages whose author states the right to share them. This is a legal question the repository owner decides; the design does not solve it.
- **Checksum is not authenticity.** Anyone who can change the repository can change the catalog and the hash together → the repository is protected (reviewed pull requests); signing is a later change if contributions open up.
- **Repository grows with every version** → measured at about 76 MB for 69 packages; if it nears 1 GB, move files to release assets (only `url` changes).
- **GitHub availability and rate limits on raw files** → the cache keeps browsing working offline; installed packages never depend on the network.
- **A bad catalog address makes the tab useless** → Clearing the setting restores the official one; the environment variable is documented for testing only.
- **New dependencies add build time and binary size** → measured at the start (task 1) before the rest is built.
- **A pinned older version removed from the catalog by mistake** → `tpv index --check` fails when an entry disappears from the existing catalog.

## Migration Plan

1. Create the catalog repository's workflows; commit the sample packages first and let the workflow publish the first catalog as a real test.
2. Ship the app change with the built-in address pointing there.
3. Rollback: the setting can point elsewhere; the tab can be hidden by an empty catalog without breaking installed vehicles.

## Open Questions

- Where `generated` and the changelog text are shown in the detail pane (layout only).
