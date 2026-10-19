# Contributing to TruckPaint

Thank you for your interest! This guide explains how to build TruckPaint, how the
repository works, and how a change goes from an idea to `main`.

## Before you start

- TruckPaint is a livery editor for ETS2 and ATS, not a general vector editor. A few
  rules shape every feature:
  - every project has a vehicle, and you always paint on the vehicle's template;
  - a project is a fleet of one game (ETS2 or ATS), exported as one mod;
  - texture layouts differ between vehicles, so the fleet shares elements (palette,
    styles, symbols), never a position copied from one vehicle to another.
- For a bug or a small fix, open an issue or a pull request.
- For a new feature, open an issue first, so we can agree on it before you write code.

## Set up

Tools and commands are managed by [mise](https://mise.jdx.dev). `mise.toml` pins Rust
(with rustfmt and clippy), Node, pnpm and OpenSpec, and defines all the project tasks.

```sh
git clone https://github.com/acesyde/truckpaint.git
cd truckpaint
mise install       # install the pinned tools
mise run run       # run the app (debug build)
mise run build     # release build: target/release/truckpaint
mise tasks         # list all tasks
```

### What your system needs

**Linux** (Debian/Ubuntu package names; X11 and Wayland both work):

```sh
sudo apt-get install libxkbcommon-dev libwayland-dev libx11-dev libxcursor-dev \
  libxrandr-dev libxi-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libgl1-mesa-dev
```

**Windows**: the MSVC toolchain (Visual Studio Build Tools with the C++ workload).

**macOS**: Xcode Command Line Tools (`xcode-select --install`).

### If the app doesn't start

TruckPaint draws with Vulkan, DirectX 12 or Metal. If it fails with a graphics adapter
error (old drivers, virtual machines), choose another backend:

```sh
WGPU_BACKEND=gl truckpaint      # OpenGL (Linux, Windows)
WGPU_BACKEND=vulkan truckpaint  # Vulkan
```

Logs go to stderr and to a daily file in the application data folder (`logs/`). Set
`RUST_LOG=debug` for more detail.

## Check your work

```sh
mise run fmt       # format the code
mise run lint      # clippy, warnings are errors
mise run test      # unit tests and headless UI tests
mise run ci        # everything CI runs, in order
```

UI behavior is tested headlessly with `egui_kittest`. To look at the main screens as
images (needs a GPU):

```sh
mise run screenshots   # images in target/screenshots/
```

### Continuous integration

CI runs the same mise tasks on Linux, Windows and macOS.

- A change that only touches documentation (`*.md` files, `openspec/`, `.claude/`,
  `website/`) skips format, lint, tests and builds. Any other file runs everything.
  Check a range of commits locally with `mise run ci:changes <base> <head>`.
- Branch protection requires the **CI result** check. It passes when the Rust jobs pass,
  or when they were skipped for a documentation-only change.
- The documentation site has its own workflow (see below).

## How a change is made

Features and larger changes are planned with [OpenSpec](https://github.com/Fission-AI/OpenSpec)
in `openspec/`:

- `openspec/specs/` holds the requirements of what is already shipped.
- `openspec/changes/` holds the changes in progress: each has a proposal, a design,
  spec changes and a task list.
- When a change is merged, it is archived in `openspec/changes/archive/` and its specs
  are merged into `openspec/specs/`, in a separate pull request.

Small fixes don't need an OpenSpec change.

### Commits and pull requests

- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org):
  `feat: …`, `fix: …`, `docs: …`, `chore: …`. Put the OpenSpec change name in
  parentheses at the end when there is one, for example
  `feat: textures of any width and height (rectangular-textures)`.
- Fill in the pull request template. Write it for players, in basic English: say what
  they can now do, see or avoid, not which files changed.
- Before you open the pull request, `mise run ci` must pass.

## Releasing

A release is started from GitHub, with a button. Two workflows do it
(`.github/workflows/release.yml` and `prerelease.yml`, sharing `release-build.yml`).
Pushing a tag starts nothing: the tag is created by GitHub when the release is published.

1. Open **Actions**, choose **Release** or **Pre-release**, and click **Run workflow**.
2. Choose the branch in **Use workflow from**, type the `tag`, and keep **publish** ticked
   (or untick it, see below).
3. About ten minutes later the release is published; the run summary has its link.

The same, from a terminal:

```sh
gh workflow run release.yml --ref main -f tag=v0.2.0
gh workflow run prerelease.yml --ref feat/gradients -f tag=v0.2.0-beta.1
```

- **Release** only runs from `main`, with a tag like `v0.2.0` (no suffix). It is the
  Latest release. **Pre-release** runs from any branch, with a suffix like
  `v0.2.0-beta.1`, and is never the Latest release. Choosing the workflow chooses the
  kind; a tag with the wrong shape is refused.
- A wrong request stops in seconds, before any build: a tag that is not a version, the
  wrong kind of tag for the workflow, Release started from another branch than `main`, or
  a tag that already exists.
- The build is the head of the chosen branch at the moment you click. The release is made
  for that exact commit, and GitHub creates the tag on it at publication.
- The **Run workflow** button only appears when the workflows are on `main`, and the branch
  you choose must contain the workflow files: update an older feature branch from `main`
  first.
- Each release holds three downloads (Linux x86_64, Windows x86_64, macOS Apple silicon)
  and a `SHA256SUMS` file, and each of them has an attestation. The release stays a draft
  until all of them are built, attached and attested. If a build fails, run the same
  workflow again with the same tag: the draft is reused.
- **Untick `publish` to review first.** Everything is built and attached, but the release
  stays a draft and the tag does not exist yet. Look at it, then publish it in GitHub, or
  run the workflow again with `publish` ticked.
- **Releases are immutable.** Once published, the tag and the files never change, and a
  tag name is never reused, even if you delete the release. A mistake is fixed with the
  next version. The title and the notes can still be edited (`gh release edit`, or the
  pencil on the release page). Turn immutable releases on in the repository settings (the
  section about releases) before the first release.
- **Rehearse before the first real release.** Run Pre-release from a branch with `publish`
  unticked and look at the draft; run it again with `publish` ticked for a version nobody
  needs (`v0.1.0-beta.2`); then run Release from `main`, first with `publish` unticked.
- **The app's version is the tag's, not the one in `Cargo.toml`.** The workflow gives it to
  the build in `TRUCKPAINT_VERSION`, and About, the log and the MCP server show it. A
  build you make yourself shows `dev`. Don't change `version` in `Cargo.toml` to release.
- Check a download with its attestation (it needs the GitHub CLI):

  ```sh
  gh attestation verify truckpaint-v0.2.0-macos-arm64.zip -R acesyde/truckpaint
  ```

  It proves which workflow and commit built the file. It is not code signing: the programs
  are not signed, so Windows SmartScreen and macOS Gatekeeper still ask players to
  confirm. The documentation says how.
- Make the download of your own platform with `mise run package` (it writes it to `dist/`;
  set `TRUCKPAINT_VERSION` to name a version). The release uses the same task.

## Translations

The app is in English, French, Spanish and German. Every string is in
`crates/tp-i18n/locales/<language>/*.ftl` ([Fluent](https://projectfluent.org/) format).

- A new string goes into all four languages, with the same message id.
- **English casing:** commands, menu items, buttons and dialog titles use Title Case,
  with small words (a, an, the, and, or, of, to, from, in, on, for, by, with, as, at)
  in lowercase unless they come first: "Custom Vehicle…", "Copy from Cabin…",
  "Import from Library…". Labels, hints, messages and tooltips use sentence case:
  "Color picker". The undo history names an action like its command, without "…".
- **Spelling:** American English ("color", not "colour").
- **Other languages** follow their own usage (sentence case in French, Spanish and
  German, except German nouns), use the game's own terms when it has them, and keep
  the names of two commands clearly different.
- German texts are the longest: check that the screens still fit, for example with
  `mise run screenshots`.

## Documentation site

The player documentation is in [website/](website/). It is an
[Astro Starlight](https://starlight.astro.build) site in the same four languages,
published at <https://truckpaint.ace-dev.me/> by the Documentation workflow when `main`
changes it.

```sh
mise run doc:dev               # preview the site, with live reload
mise run doc:build             # build it as CI does
mise run doc:media [section]   # render the screenshots and animations (needs a GPU)
mise run doc:tokens            # update the site's colors from the app
```

- Pages are in `website/src/content/docs/<language>/`. Write the English page first,
  then the three translations.
- Quote an interface label with `<Ui id="message-id" />`: it shows the app's own text
  in the page's language. Show a screenshot with `<Shot name="…" />` and an animation
  with `<Clip name="…" />`. The build fails if a label or a medium is missing.
- Screenshots and animations are rendered from the app, in every language, by
  `crates/tp-app/tests/doc_media.rs`, then committed. When your change modifies a
  documented screen, render its section again, for example
  `mise run doc:media workshop`. `doc:media` installs ffmpeg for you.
- The site's colors come from the app's design tokens. A test fails if they are out of
  date: run `mise run doc:tokens`.

## Vehicle packages

Vehicles come as packages (`.tpv`), described in
[the package format page](https://truckpaint.ace-dev.me/en/vehicles/package-format/) of the documentation (`website/src/content/docs/en/vehicles/package-format.mdx`). The repository ships
sample vehicles in [examples/vehicles/](examples/vehicles/): a truck (versions 1.0.0 and
1.1.0, to try Update Template) and a trailer.

```sh
mise run pack <folder> [-o file.tpv]   # pack a folder (PNG, SVG or DDS templates)
mise run sample-vehicles               # rebuild the sample packages
```

### Game extraction (maintainers)

Players make the official trucks and trailers inside the app (Vehicle Library, My Game tab).
`tpv extract` is the same code (`tp_pack::extract`: `inspect`, `scan`, `generate`) as a developer tool
for whole-game batches, checking a game update and comparing batches:

```sh
mise exec -- cargo run --release -p tp-pack --bin tpv -- extract "/path/to/ETS2" \
  [--vehicle man.tgx_2020] [--output <directory>] [--previous <folder of packages>]
```

- Package versions come from the game (game 1.61.1.9 gives 1.61.1). With `--previous`, an unchanged
  template keeps its layout version, a changed one gets the next, and a vehicle that did not change is
  reported as skipped (`up_to_date`) and not written.
- Each run writes a unique subdirectory of `dist/` (or `--output`) with the packages and `report.json`
  (sources, texture coverage, successes, failures, `missing_dependencies`, `dimension_diagnostics`).
  Intermediates stay under `target/` only when a vehicle fails. Vehicles run in parallel (half the cores).
- Compare two batches by unpacking the `.tpv` files: templates must be byte-identical unless the game changed them.
- Technical validation does not prove correctness in game: check new vehicles in the game.

See `docs/game-extraction-plan.md` for the design and the validated baselines.

## Code layout

| Crate | Role |
|---|---|
| `tp-core` | Domain types: projects, objects, paths, colors. No UI, no files, no threads. |
| `tp-text` | Text layout, glyph outlines and text editing. |
| `tp-render` | Full-size texture rendering and export (PNG, DDS). |
| `tp-file` | The project file format (`.truckpaint`). |
| `tp-i18n` | Interface languages: Fluent messages, current language, local number format. |
| `tp-ui` | Design system: tokens, theme, fonts, icons and widgets. |
| `tp-vehicles` | Vehicle packages (`.tpv`): manifest, checks and templates. |
| `tp-pack` | The `tpv` command: builds and checks vehicle packages. |
| `tp-scs` | Reads the game's `.scs` archives and vehicle models, in memory. |
| `tp-mcp` | The local MCP server: protocol, tools and their arguments (no UI). |
| `tp-app` | The desktop app: screens, commands, layout and preferences. |

## License

TruckPaint is under the MIT license (see [LICENSE](LICENSE)). By contributing, you agree that your work is
published under the same license.
