## 1. Preview behavior

- [x] 1.1 Add a per-language close label (en, fr, es, de) in `website/src/lib/` and verify it returns the right word for each of the four languages and falls back to English for an unknown one
- [x] 1.2 Wrap the image of `website/src/components/Shot.astro` in a `button.tp-zoom` carrying the close label as a data attribute, and verify `pnpm build` (in `website/`) passes and the built page contains the button around the image with its alt text unchanged
- [x] 1.3 Add the page script to `Shot.astro`: one shared `<dialog>` created on first use, one delegated click handler, `showModal()`, click-anywhere close, `overflow: hidden` on the root while open; verify in `pnpm dev` that a click opens the preview and a click, Escape and the close button each close it with the page at the same scroll position
- [x] 1.4 Add the styles to `website/src/styles/theme.css` (zoom-in pointer, button reset with a focus ring from the accent token, dimmed backdrop, image fitted without cropping and without border radius, close button), and verify in dark and light themes and at a phone width that the preview fits and the close button is visible

## 2. Checks

- [x] 2.1 Verify keyboard use: Tab to a screenshot, Enter and Space open it, Escape closes it, focus is back on the screenshot
- [x] 2.2 Verify on a French and a German page that the close button's accessible name is in that language, and that clicking an animation (`Clip`) opens nothing
- [x] 2.3 Run `pnpm build` in `website/` and confirm no page fails (all `<Shot />` usages in the four languages still build), and confirm `git status` shows no changed media or token files
