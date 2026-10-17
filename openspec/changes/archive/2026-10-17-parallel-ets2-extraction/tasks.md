## 1. One recursive listing per archive

- [x] 1.1 Make the fake converter answer `--list-directory-recursive` (and the `.noroot` case), and list each archive once recursively at inventory, filling the file map and listing memo; archives with an empty recursive root listing keep per-folder listings; verify with the call log that a listable archive gets one recursive listing and no `--list-directory`, that the `.noroot` archive keeps per-folder listings, and that all extraction tests pass unchanged.

## 2. Concurrent vehicle builds

- [x] 2.1 Split `Sources` into the shared inventoried file map and listing memo (`Arc`, read-only) and per-worker state (own additions, work subfolder, shared model cache path); verify the provenance unit test passes through a worker view and that two workers never write the same extraction or log path.
- [x] 2.2 Build vehicles on `available_parallelism / 2` (at least 1) scoped workers pulling from a shared position counter, collect results by position and sum failures and model cache counters; verify a fixture batch of several trucks and trailers gives packages byte-identical to single-vehicle batches, the report in trucks-then-trailers id order, unchanged failure isolation and exit codes, and that all extraction tests pass.

## 3. Validate on the game

- [x] 3.1 Run a full batch on the local ETS2 installation with an empty model cache, then a second one, both with `--previous` on the last full batch; verify all packages are byte-identical to it, the `sources` and `name_sources` evidence are equal, and record converter calls, durations and the peak memory of both in `docs/game-extraction-plan.md`.
- [x] 3.2 Run `mise run ci` and verify format, lint, tests and build pass; report any GPU or game-only checks that could not run.
