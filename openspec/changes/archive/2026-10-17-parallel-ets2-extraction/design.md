## Context

After `speed-up-ets2-extraction`, measured on ETS2 1.61.1.9 (115 archives, 84 vehicles, 16 cores):

| Batch | Converter calls | Duration |
|---|---|---|
| Empty model cache | 28,484 (9,978 conversions) | 55 min |
| Full model cache | 18,506 (0 conversions) | 4 min |

- A conversion with the 115 archives mounted takes about 310 ms. A plain listing with the same mounts takes 244 ms, so most of the cost is mounting, paid by every converter process. ConverterPIX converts one model per call.
- `extract::run` builds vehicles one after another. Each `Build` borrows `&mut Sources` and uses only its file map, folder listings, work folder, converter command and model cache counters. Names, previous packages and the vehicle list are read-only.
- `--list-directory-recursive /` lists a whole archive: 306 ms and 181,576 entries for `base.scs`, 12 ms for a DLC. `base_cfg.scs`, `core.scs` and `locale.scs` return nothing for `/` (`readDir returned null`) although they hold folders. Their sub-folders list normally.
- The remaining calls of a cached batch are per-folder listings (memoized and pruned) and single-file extractions.

## Goals / Non-Goals

**Goals:**
- Use the idle cores for the first batch after an update. Target: under 10 minutes, with the longest single vehicle (Scania R 2016, about 3.5 minutes) as the floor.
- Replace per-folder listings with one recursive listing per archive where the archive allows it.
- Packages, report content and report order identical to the sequential batch.
- Keep the desktop usable: a GNOME session was killed by `systemd-oomd` (user memory at 24.4 of 30 GB) during a full-core `cargo` CI run. Measured peaks: a conversion takes 32 MB, the recursive listing of `base.scs` 89 MB, and `tpv` for one vehicle with the full file map 237 MB.

**Non-Goals:**
- A `--jobs` option.
- Fewer mounts per conversion: the `.pit` effects come from `.mat` files that may live in any archive, and a missing mount would change a shader silently.
- Parallel conversions inside one vehicle.

## Decisions

1. **One recursive listing per archive.** `inventory` runs `--list-directory-recursive /` once per archive and adds every `[F]` entry to the file map (archive indices kept in mount order) and every `[D]`/`[F]` entry to the listing memo under its parent folder. An archive whose recursive root listing is empty is marked "unlisted" and keeps the per-folder path of `speed-up-ets2-extraction` (memo plus direct listing of children). The two existing `--list-directory` + `--extract-directory` inventory calls per folder become `--extract-directory` only, for archives that hold the folder.

   Alternatives rejected:
   - Recursive listings of the vehicle folders only: the models, materials and locale files live outside them.
   - Running without the fallback: it loses the localization files.

2. **Workers over a shared vehicle queue.** After discovery, `run` starts `min(max(available_parallelism / 2, 1), vehicles)` workers (8 on the reference machine). Half the cores keeps most of the gain, since the longest vehicle is the floor, and leaves headroom for the desktop with `std::thread::scope`. Each worker takes the next vehicle position from an `AtomicUsize` and builds and publishes that vehicle. It stores `(position, result entry, failed)` in a `Mutex<Vec<…>>`. After the scope, results are sorted by position, so the report order is the sequential order. The `eprintln!("extracting …")` progress lines may interleave. No new dependency.

   Alternative rejected: a channel-fed pool or rayon, which adds machinery or a dependency for a queue of 84 items.

3. **A `Sources` per worker over a shared file map.** The inventoried file map (hundreds of thousands of paths) and the listing memo are moved into an `Arc` and read by every worker. Each worker's `Sources` keeps its own additions (files found later in archives without a whole listing), its own `work = <batch work>/worker-<n>` and the shared model cache path. Memory then stays near one map plus 8 converters (about 0.5 GB), instead of one map per worker. Extractions, export folders and converter logs are then never written by two processes at once. A clone repeats the extraction of shared definitions it needs (a few ms each); this is accepted. The model cache counters are summed after the scope.

   Two workers may convert the same model at once. Each converts into its own folder and renames into the cache; `rename` replaces atomically and the content is identical. A reader requires both `.pim` and `.pit`, and the `.pim` arrives last, as today.

4. **Proof of equivalence.**
   - **Fake converter:** a batch of several vehicles gives packages byte-identical to single-vehicle batches, and reports in the sequential order.
   - **Fake converter:** an archive marked as unlisted keeps per-folder listings; a listable one gets exactly one recursive listing and no per-folder listing.
   - **Local game:** a full batch with an empty cache, then one with a full cache. All 69 packages byte-identical to the last batch, the `sources` and `name_sources` evidence equal, and durations and call counts recorded.

## Risks / Trade-offs

- [Recursive listings differ from per-folder ones in some archive] → The full-batch evidence comparison catches it (same provenance for every file used).
- [8 converters at once write a lot of `.pim` text (11 GB per empty-cache batch)] → Accepted, measured on the full batch.
- [Memory] → One shared file map and 8 converters, measured on the full batch.
- [Interleaved progress lines are harder to read] → The report stays ordered. Progress only names the vehicle being started.

## Migration Plan

No migration. Report fields and package formats are unchanged.

## Open Questions

None.
