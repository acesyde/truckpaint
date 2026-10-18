## Context

`Shot.astro` renders a plain `<img class="tp-media">` (WebP, width and height from `sharp`, lazy loaded). Every page uses it through `<Shot name alt />`, and `Clip.astro` already has a page-level `<script>` pattern that Astro bundles once however many components are on the page. The site has no client framework and no lightbox dependency; Starlight ships no strings for a custom control.

See proposal.md for motivation.

## Goals / Non-Goals

**Goals:**
- Zero edits to existing pages: the behavior lives in `Shot`.
- Accessible: keyboard open and close, focus returned, labeled close control, modal semantics.
- No new dependency, no JavaScript needed to read the page (the image stays a plain image without it).

**Non-Goals:**
- Zoom, pan, gallery, swipe gestures.
- Preview for `Clip` animations.

## Decisions

**A native `<dialog>` opened with `showModal()`, created once by the page script.** The browser gives the top layer, a dimmed `::backdrop`, Escape to close, focus trapping and focus return to the opener for free. Alternative: a hand-made fixed overlay `div` — needs all of that written and tested by hand. Another: a library (e.g. GLightbox, medium-zoom) — a new dependency for ~40 lines of behavior, and its styling would have to be matched to the app's tokens.

**`Shot` wraps the image in a `<button class="tp-zoom" type="button">`; the script delegates one click handler on `document`.** A button gives focus, Enter and Space without extra code, and a screen reader announces it as a button whose name is the image's alt text. The handler copies the clicked image's `src`, `alt`, `width` and `height` into the shared dialog's `<img>`. Alternative: attach `click` to each `img` with `tabindex` and key handlers — re-implements a button.

**The dialog's image is fitted by CSS, not scaled by script:** `max-width: 100vw; max-height: 100vh; object-fit: contain`, with the image's own width and height attributes so layout does not jump. The WebP is already larger than the page column, so no second, larger file is needed. Small images are not stretched beyond their size.

**Click anywhere closes** (the dialog is the image plus a close button on a backdrop; a click on the image, the backdrop or the button all call `close()`). A reader who wants to read details does not need a second gesture. Escape is native.

**Close label per language from a small map in `website/src/lib/` keyed by the page's language** (`Close` / `Fermer` / `Cerrar` / `Schließen`), read by `Shot` and given to the script as a data attribute on the button. The dialog is created in the script, so the label is taken from the clicked button at open time. Alternative: Starlight's i18n collection for custom keys — more files for four words; and the app's Fluent files are for the app, not the site.

**Scroll lock:** the top-layer modal dialog does not scroll the page by itself with wheel input on all browsers, so the script sets `overflow: hidden` on the root while it is open and removes it on the `close` event (which fires for every way of closing).

**Only screenshots.** `Clip` keeps its own behavior: the video plays inline, and a modal on top of a looping, observer-driven video would fight its play/pause logic.

**Pointer:** `.tp-zoom { cursor: zoom-in }`; the existing `.tp-media` border and radius are kept on the image, and the button is reset to look like nothing but the image. In the preview the image has no border radius, to use the whole window.

## Risks / Trade-offs

- [Old browsers without `<dialog>`] → The site targets current browsers; the guard `typeof dialog.showModal === 'function'` makes the button a no-op there, and the screenshot is still readable inline.
- [A button around an image changes layout (inline box, focus ring)] → Make the button `display: block; width: 100%` with no padding or background, and a visible focus ring from the accent token.
- [Image still loading lazily when opened] → The page image is the same URL, so the browser has already fetched or will fetch it once; the dialog's image just reuses the cache.
- [Mobile: the screenshot is wide and short, so the preview is no bigger than inline] → Accepted; fitting to the window is still the fullest view without panning, and pinch zoom is available in the browser.
