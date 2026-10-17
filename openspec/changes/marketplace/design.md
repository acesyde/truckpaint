## Context

Packages are installed from bytes (`install_bytes` in `tp-app/src/vehicles.rs`), which validates fully and refuses on error; every install path goes through it. `tp-vehicles` already owns the manifest and `diff`; `tp-pack` owns the `tpv` command. The workspace has no HTTP client and no SHA-256 crate. The app is not released, so there is nothing earlier to stay compatible with. See proposal.md for motivation.

## Goals / Non-Goals

**Goals:**
- One static file and plain release assets: no server code to run.
- The same validation for a downloaded package as for a dropped file.
- The catalog format shared by the app and `tpv index`, so they can't drift.

**Non-Goals:**
- Package signing, accounts, upload from the app, delta downloads, mirrors.

## Decisions

**Hosting: GitHub Release assets for `.tpv`, `catalog.json` on GitHub Pages.** Not Git LFS: raw URLs return the pointer file, and the free bandwidth quota runs out. Not `.tpv` in git: the history grows with every version. Not a submodule: the app reads the catalog at run time, not at build time. Pages gives a stable address, a CDN and CORS. The address is a setting, so moving it later costs no release. The default address is one constant in the app.

**One catalog, all versions, immutable.** A flat `packages` list, one entry per `(id, version)`. Immutability is checked by `tpv index --check` against the committed catalog (an existing version's SHA-256 must not change). The app picks the newest version with `min_app <= app version`; older ones stay available in the detail pane, which serves projects pinned to an earlier layout (`template-update-impact`).

**Catalog types live in `tp-vehicles`,** next to `Manifest`: `Catalog`, `Entry`, parsing, and `newest_compatible`. `tp-pack` builds it, `tp-app` reads it. Pure code, unit-tested, no I/O.

**`tpv index` reuses `tpv check`'s validation** and fills the entry from the manifest, not from file names. Output is sorted and has no timestamp except `generated`, which `--check` ignores, so the check is stable.

**Dependencies: `ureq` (blocking, rustls) and `sha2`.** The app already runs work off the UI thread; a blocking client in a worker keeps that pattern and avoids pulling an async runtime into an egui app. Alternative `reqwest`: heavier, async, no benefit here. Both are added to the workspace and checked for license and build time on all three desktop targets before merging.

**Download rules.** HTTPS only (or `file://` for the override, to test a catalog locally). The body is read with a size cap (the entry's `size`, and a hard maximum), a connect and a total timeout. The SHA-256 is computed over the bytes and compared to the entry before `install_bytes` is called. A mismatch or a failed install writes nothing.

**Cache.** `catalog.json` is saved in the app's cache folder after it parses and has a known `schema`, via a temporary file and rename, so a bad response or a crash never leaves a broken cache. The tab shows the cache's date when offline. No conditional requests at first; the file is small.

**Address precedence:** `TRUCKPAINT_CATALOG_URL`, then the preference, then the built-in address.

**No telemetry.** Requests carry a plain `User-Agent` with the app version only.

**UI.** The tab is a third tab of the existing Vehicle Library dialog and reuses its list, filters and detail pane. Install state is derived by comparing catalog entries with installed manifests, so it can't disagree with the Installed tab. `first-run` later reuses the same catalog client and list model, without the dialog.

## Risks / Trade-offs

- **Redistribution of SCS assets** → catalog entries need `license`, and the repository only accepts packages whose author states the right to share them. This is a legal question the repository owner decides; the design does not solve it.
- **Checksum is not authenticity.** Anyone who can change the repository can change the catalog and the hash together → the repository is protected (reviewed pull requests); signing is a later change if contributions open up.
- **GitHub availability and rate limits on assets** → the cache keeps browsing working offline; installed packages never depend on the network.
- **A bad catalog address makes the tab useless** → Clearing the setting restores the official one; the environment variable is documented for testing only.
- **New dependencies add build time and binary size** → measured at the start (task 1) before the rest is built.
- **A pinned older version removed from the catalog by mistake** → `tpv index --check` fails when an entry disappears from the existing catalog.

## Migration Plan

1. Create the catalog repository's workflows and an empty `catalog.json`; publish the sample packages first as a real test.
2. Ship the app change with the built-in address pointing there.
3. Rollback: the setting can point elsewhere; the tab can be hidden by an empty catalog without breaking installed vehicles.

## Open Questions

- Where `generated` and the changelog text are shown in the detail pane (layout only).
