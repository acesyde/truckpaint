## Why

The documentation screenshots are shown at the width of the page column, which is narrower than the app window they come from. Toolbars, panels and labels are small and often hard to read, and the reader has no way to see the image larger.

## What Changes

- **A screenshot opens full screen when clicked** (or when Enter or Space is pressed on it): the image is shown as large as the window allows, on a dimmed background, in the page's language.
- **It closes** by clicking anywhere, pressing Escape, or using a close button; the page is back where it was, and keyboard focus returns to the screenshot.
- Screenshots show a zoom-in cursor so readers can tell they are clickable.
- The close button is labeled in the page's language (English, French, Spanish, German).
- Scrolling the page behind the preview is blocked while it is open.
- Animations are unchanged: they already play inline, and a click on them is not changed.

Non-goals:
- zooming or panning inside the preview (it shows the image fitted to the window);
- a gallery to move between screenshots;
- new screenshots, or any change to the media command.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `user-documentation`: screenshots can be opened full screen on click.

## Impact

- **Site:** the `Shot` component and the site's stylesheet (`website/src/components/Shot.astro`, `website/src/styles/theme.css`), plus a small helper for the close-button label per language (`website/src/lib/`).
- **Pages and media:** none; every existing `<Shot />` gets the behavior without edits, and no screenshot is rendered again.
- **App, crates, strings:** none (the close label belongs to the site, not to the app's Fluent files).
