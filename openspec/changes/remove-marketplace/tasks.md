## 1. App

- [ ] 1.1 Delete `crates/tp-app/src/marketplace.rs` and `ui/marketplace_tab.rs`; remove `pub mod` lines in `lib.rs` and `ui/mod.rs`, `AppState.marketplace`, `AppDirs::catalog`, and the cache line in `main.rs`; `cargo check -p tp-app` passes
- [ ] 1.2 Vehicle Library: remove `LibraryTab::Marketplace`, `market_checked`, the `market` clicks, the catalog address, the install polling and cancel in `ui/vehicle_dialogs.rs`; the dialog shows Installed and My vehicles only
- [ ] 1.3 Remove `Prefs.catalog_url` and `catalog_section` (Preferences "Marketplace" section) in `prefs.rs` and `ui/dialogs.rs`; Preferences opens with no Marketplace section
- [ ] 1.4 Remove `marketplace_*` tests, `market_catalog`, `settle_market`, `open_marketplace` in `tests/vehicles.rs` and the `vehicles/marketplace` shot in `tests/doc_media.rs`; `mise run test` passes

## 2. Crates and dependencies

- [ ] 2.1 Delete `crates/tp-vehicles/src/catalog.rs` (and its `pub mod`), `crates/tp-pack/src/index.rs`, the `index` command and usage text in `bin/tpv.rs`, and the index test in `tests/cli.rs`
- [ ] 2.2 Remove the `homepage` field from the manifest written by `tpv extract` (`extract/mod.rs`); the extract test still passes
- [ ] 2.3 Remove `ureq` and `sha2` from `Cargo.toml` and the crate manifests; remove `semver` from a crate that no longer uses it; `cargo build --workspace` and `Cargo.lock` have no `ureq`
- [ ] 2.4 Remove `DOWNLOAD`, `OFFLINE`, `INFO` from `tp-ui/src/icons.rs` if `grep` shows no other use

## 3. Strings

- [ ] 3.1 Remove `market-*`, `vehicles-tab-marketplace*`, `prefs-marketplace`, `prefs-catalog-url*` from `crates/tp-i18n/locales/{en,fr,es,de}/dialogs.ftl`; the localization parity test and `grep -ri market crates/tp-i18n` pass

## 4. Documentation

- [ ] 4.1 `vehicles/library.mdx` (en, fr, es, de): remove the Marketplace section, the intro bullet, the Preferences paragraph and the `vehicles/marketplace` shot; "three tabs" becomes two
- [ ] 4.2 `vehicles/official-vehicles.mdx` and `getting-started/first-project.mdx` (four languages): point to generating the package, not the Marketplace; fix the page description
- [ ] 4.3 `en/vehicles/package-format.mdx`: remove "Publishing in the Marketplace"
- [ ] 4.4 Delete `website/public/media/*/vehicles/marketplace.webp` and its entries in `media/manifest.json`; the site build passes (`mise run doc:build`)
- [ ] 4.5 `CONTRIBUTING.md`: remove the Marketplace paragraph and the "builds the Marketplace catalog" words in the `tp-pack` row
- [ ] 4.6 `grep -ri "marketplace\|catalog" ` over `crates website CONTRIBUTING.md` finds only the unrelated menu command catalog and the archived history

## 5. Check

- [ ] 5.1 `mise run ci` passes; `openspec validate remove-marketplace` passes
- [ ] 5.2 Owner task, outside the repo: archive GitHub repository `acesyde/truckpaint-marketplace` and take down `truckpaint-marketplace.ace-dev.me`
