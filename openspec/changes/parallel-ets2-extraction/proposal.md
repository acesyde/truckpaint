## Why

The first `tpv extract` batch after a game or DLC update still takes 55 minutes, because the batch builds one vehicle after another and each of its 9,978 model conversions starts a converter process that mounts the 115 game archives (about 0.3 s, mostly mounting). The machine has many idle cores: 16 on the reference install. Even a batch that reuses every cached model takes 4 minutes, most of it spent on 18,506 per-folder listings and file extractions.

## What Changes

- A batch builds several vehicles at the same time, on half of the available processor cores (8 on a 16-core machine), so the desktop keeps memory and CPU headroom. Each vehicle still converts its own models in order. Packages, the report's content and the order of its entries are identical to a batch built one vehicle at a time.
- Each parallel build uses its own work subfolder for extracted files, converted models and converter logs. Converted models still go to the shared model cache.
- At the start of a batch, every archive is listed once, recursively, to know which files it holds. The archives that cannot list their root (`base_cfg.scs`, `core.scs`, `locale.scs` on ETS2 1.61) keep today's per-folder listing. File provenance and archive precedence are unchanged.

Non-goals:
- A `--jobs` option. Use `--vehicle` to rebuild one vehicle alone.
- Mounting fewer archives per conversion.
- Parallel conversions within one vehicle.
- ATS.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `ets2-game-extraction`: "Isolated batch outputs" allows vehicles of a batch to be built concurrently, without changing packages, report content or order, and keeps each concurrent build's intermediates apart.

## Impact

- `tp-pack` batch run (`extract::run`): a worker per two available cores, built on the standard library's scoped threads (no new dependency). Results are collected by vehicle position.
- `tp-pack` sources: a recursive listing per archive at inventory time, the per-folder listing kept for archives without a listable root, and a per-worker `Sources` that shares the inventoried file map (read-only) and has its own work subfolder.
- Fake-converter tests: a multi-vehicle batch is identical to its single-vehicle builds, and the recursive listing is used only where the root lists.
- `docs/game-extraction-plan.md`: call counts and durations of a first and a second full batch.
- Player documentation: none expected. The official vehicles page does not describe batch timing or listing.
