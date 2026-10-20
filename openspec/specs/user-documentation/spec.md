# user-documentation Specification

## Purpose

The player documentation site: how to use TruckPaint, in the app's four languages, illustrated with screenshots and animations generated from the app itself, styled like the app in dark and light themes, and published on GitHub Pages.

## Requirements

### Requirement: Languages and fallback
The documentation SHALL be available in English, French, Spanish and German, the languages of the app, under one URL prefix per language (`/en/`, `/fr/`, `/es/`, `/de/`), with English as the default language. Every page SHALL offer a language picker that leads to the same page in the chosen language. When a page has no translation in a language, that language's URL SHALL show the English page with a notice saying it is not translated yet.

#### Scenario: Switching language
- **WHEN** a reader on the French page about mod export picks "Deutsch"
- **THEN** the German page about mod export opens

#### Scenario: Missing translation
- **WHEN** the Spanish version of a page does not exist and a reader opens its `/es/` URL
- **THEN** the English content is shown with a notice that the page is not translated yet

#### Scenario: Site root
- **WHEN** a reader opens the site's root URL
- **THEN** the English home page opens

### Requirement: Interface labels match the app
Each page SHALL quote menu items, buttons, dialog titles and settings with the exact wording the app shows in the page's language. A quoted label that has no translation in a language SHALL fail the site build rather than show a wrong or English label.

#### Scenario: French label
- **WHEN** the French page about mod export names the export command
- **THEN** the label is the one the French interface shows for that command

#### Scenario: Unknown label
- **WHEN** a page quotes a label that does not exist in the app's strings for its language
- **THEN** the site build fails and names the page and the label

### Requirement: Content sections
The documentation SHALL be organized in these sections, in this order, in every language: Getting started (installation, first launch, first project); Vehicles (Vehicle Library, official package extraction and manual installation, custom vehicles); Project and fleet (trucks, variants, trailers); Workshop (vector tools, text, images, gradients, layers); Brand (palette, symbols, styles); Export (textures, mod); AI assistant (MCP). Its wording SHALL follow the product's rules and vocabulary (a project is a fleet of one game; sharing across vehicles is by elements, never by copied placement).

#### Scenario: Sidebar
- **WHEN** a reader opens any page
- **THEN** the sidebar lists the seven sections in the order above, titled in the page's language

The official vehicle documentation SHALL describe the maintained ETS2 extraction command (which reads the game's archives itself, with no external program), installed vehicle/tuning DLC coverage, explicit package version, unique batch outputs, technical report and unsupported-case failures, followed by manual package installation, project creation and mod export. It SHALL explain that old Carsmaniac packages are retired without migration and that reusing an installed id/version replaces that version under the existing installer behavior. It SHALL NOT instruct players to use the removed catalog or template pack settings. Newly generated vehicles SHALL be described as requiring in-game checks unless matching evidence exists.

#### Scenario: Current official vehicle workflow
- **WHEN** a reader follows the official vehicle documentation
- **THEN** it leads from extraction or an available .tpv through manual installation to a project and mod export, without a Carsmaniac download or catalog selection step

### Requirement: Generated screenshots per language
Every screenshot in the documentation SHALL be rendered from the app by a local command, not captured by hand, once per documentation language with the app's interface in that language. The command SHALL render all screenshots in one run, from fixed scenes, so that running it twice without app changes gives the same images. Screenshots SHALL be WebP images at a single scale.

#### Scenario: Same screen, four languages
- **WHEN** the documentation media are generated
- **THEN** the Workshop screenshot exists four times, each showing the interface in English, French, Spanish or German

#### Scenario: Page shows its language's screenshot
- **WHEN** a reader opens the German Workshop page
- **THEN** its screenshots show the German interface

### Requirement: Animated features
Gestures that a still image cannot explain SHALL be shown as short looping animations rendered from the app by the same local command, in each language: at least creating a new project, drawing a shape, editing a text, applying a palette swatch or style, and exporting the mod. Animations SHALL show a pointer that moves and clicks where the gesture happens. They SHALL play muted, looped and without controls by default, SHALL be video files (not GIF), and SHALL show a still first frame and a pause control when the reader asks the system for reduced motion.

#### Scenario: Drawing a rectangle
- **WHEN** a reader opens the Workshop page about shapes
- **THEN** an animation shows the pointer picking the Rectangle tool, dragging on the texture and a rectangle appearing

#### Scenario: Reduced motion
- **WHEN** the reader's system asks for reduced motion
- **THEN** animations do not autoplay and show their first frame with a play control

### Requirement: Complete media
The site build SHALL fail when a page references a screenshot or animation that does not exist for the page's language. The media command SHALL record, with the media, the app version and commit they were generated from.

#### Scenario: Missing medium
- **WHEN** a French page references an animation that was only generated in English
- **THEN** the site build fails and names the page and the missing file

### Requirement: App look in dark and light themes
The documentation SHALL use the app's fonts (Geist for text, JetBrains Mono for code and values) served by the site itself, and colors taken from the app's design tokens. It SHALL have a dark theme using the app's palette and a light theme derived from it that keeps each accent's meaning (primary action, signal, link to the brand) and meets WCAG AA contrast for text. The theme SHALL follow the reader's system preference by default and SHALL be switchable, the choice being remembered. Screenshots and animations SHALL show the app as it is (dark) in both themes.

#### Scenario: Dark system
- **WHEN** a reader whose system is in dark mode opens the site for the first time
- **THEN** the site uses the dark theme with the app's surface colors

#### Scenario: Switching theme
- **WHEN** the reader switches to the light theme and opens another page
- **THEN** the other page is in the light theme

#### Scenario: Tokens follow the app
- **WHEN** a color token of the app changes and the documentation media are regenerated
- **THEN** the site's matching color changes with no hand edit

### Requirement: Build and deployment
The site SHALL build with a single command from a clean checkout using only the committed sources and media (no Rust build, no GPU). It SHALL be published to GitHub Pages, at its own domain `https://truckpaint.ace-dev.me/`, when a push to `main` changes the site, and its links and assets SHALL work at that address.

#### Scenario: Publishing a page change
- **WHEN** a pull request that edits a page under `website/` is merged into `main`
- **THEN** the updated page is online at the site's domain, with its images and styles loading

#### Scenario: Local preview
- **WHEN** a contributor runs the documentation preview command
- **THEN** the site is served locally in all four languages with live reload

### Requirement: Full-screen screenshot preview
A screenshot in the documentation SHALL open in a full-screen preview when the reader clicks it, or presses Enter or Space while it has keyboard focus. The preview SHALL show the same image (in the page's language) as large as the window allows without cropping, on a dimmed background, and SHALL keep the page behind it from scrolling. It SHALL close when the reader clicks anywhere, presses Escape or uses its close button, after which the page SHALL be at the same scroll position and keyboard focus SHALL return to the screenshot. The close button's label SHALL be in the page's language. Screenshots SHALL show a zoom-in pointer so that they are recognizable as clickable, and the preview SHALL work in the dark and light themes and on small screens. Animations SHALL NOT open a preview.

#### Scenario: Opening a screenshot
- **WHEN** a reader clicks a screenshot on the German Workshop page
- **THEN** the German screenshot fills the window, fitted without cropping, on a dimmed background, and the page behind does not scroll

#### Scenario: Closing the preview
- **WHEN** a reader presses Escape, clicks the image or clicks the close button
- **THEN** the preview closes, the page is at the same scroll position, and the screenshot has keyboard focus

#### Scenario: Keyboard only
- **WHEN** a reader tabs to a screenshot and presses Enter
- **THEN** the preview opens, and Escape closes it with focus back on the screenshot

#### Scenario: Close label follows the language
- **WHEN** a reader opens a screenshot on a French page
- **THEN** the close button's accessible name is in French

#### Scenario: Animation
- **WHEN** a reader clicks an animation
- **THEN** no preview opens

### Requirement: One example livery in the media
Every documentation screenshot and animation that shows a finished paint job SHALL show the same example livery, "Blue Wave Biarritz" (a fictional carrier), not ad-hoc test shapes. Media that teach one gesture or one drawing feature (drawing a first shape, the Pen, shapes and paths) MAY keep their own scene, on an empty texture or with the feature's own artwork. The livery SHALL be painted on the sample truck and the sample trailer, so a fleet is shown, and SHALL be built from the elements the app teaches: a palette of three named colors (White, Light blue, Blue), a logo symbol with one instance per texture, a graphic style and a text style, with objects linked to the swatches and styles they use. Its artwork (logo, wave graphics, coast silhouette) SHALL be original vector drawings kept in the repository under the repository's license; no third-party logo, trademark or generated image SHALL be committed. Its brand name and tagline are artwork and SHALL read the same in the four documentation languages.

#### Scenario: Same livery on every page
- **WHEN** a reader goes from the Workshop pages to the Brand and Export pages
- **THEN** every screenshot and animation that shows a finished paint job shows the Blue Wave Biarritz livery, on the sample truck or the sample trailer

#### Scenario: A feature demo keeps its own scene
- **WHEN** a reader opens the screenshot of shapes and paths
- **THEN** it may show that feature's own artwork rather than the livery, and the animation that draws a first rectangle starts from an empty texture

#### Scenario: Brand elements are real
- **WHEN** a reader opens the Brand screenshots
- **THEN** the palette lists White, Light blue and Blue, the symbols list holds the wave logo with its instances on the truck and on the trailer, and the objects that use these colors are linked to them

#### Scenario: Placement follows each texture
- **WHEN** the logo appears on the truck's texture and on the trailer's texture
- **THEN** each instance is placed for that texture's own layout; no placement is copied from one vehicle to the other

#### Scenario: No third-party artwork
- **WHEN** the repository is searched for the livery's artwork
- **THEN** it is made of SVG files drawn for the project, and no brand mark of a vehicle maker or trailer maker and no copy of the reference presentation image is present

### Requirement: Example livery follows its brief
The example livery SHALL be defined by a written brief kept in the repository (name, tagline, three colors with their meaning, logo lockup, motifs, composition rules) and by a layout that places each element in a named zone of each sample template, as fractions of that zone. Rendering the media twice SHALL give the same images, and a change to a zone of a template SHALL move the artwork with it. Zones the layout does not list SHALL stay unpainted.

#### Scenario: Same layout every time
- **WHEN** the documentation media are generated twice without any change
- **THEN** every image and animation frame is identical

#### Scenario: Artwork stays in its zone
- **WHEN** the layout places the logo on the trailer's curtain zone
- **THEN** the logo is inside that zone, at the same relative position and size on every render

#### Scenario: Unlisted zone
- **WHEN** a zone of a texture has no row in the layout
- **THEN** it is left unpainted

### Requirement: Example livery shared with the tests
The example livery SHALL be built by one scene that the documentation media and a headless test (no GPU) both use. The headless test SHALL fail when the livery is no longer valid: a missing vehicle or texture, an element placed outside its zone or in a zone the template does not have, a zone of the layout left unpainted, an object with a color outside the palette, a symbol without its instances, or a texture that exports as an empty one.

#### Scenario: Livery stays valid
- **WHEN** the test suite runs
- **THEN** the livery builds on the sample fleet, every color it uses is a palette swatch, the logo symbol has an instance on the truck and on the trailer, and a texture exported from it is not blank

#### Scenario: Images follow the livery
- **WHEN** the example livery changes and the documentation media are generated
- **THEN** every medium shows the changed livery, with no second copy of it to update
