## ADDED Requirements

### Requirement: One example livery in the media
Every documentation screenshot and animation that shows a finished paint job SHALL show the same example livery, "Blue Wave Biarritz" (a fictional carrier), not ad-hoc test shapes. Media that teach one gesture or one drawing feature (drawing a first shape, the Pen, shapes and paths, gradients) MAY keep their own scene, on an empty texture or with the feature's own artwork. The livery SHALL be painted on the sample truck and the sample trailer, so a fleet is shown, and SHALL be built from the elements the app teaches: a palette of three named colors (White, Light blue, Blue), a logo symbol with one instance per texture, a graphic style and a text style, with objects linked to the swatches and styles they use. Its artwork (logo, wave graphics, coast silhouette) SHALL be original vector drawings kept in the repository under the repository's license; no third-party logo, trademark or generated image SHALL be committed. Its brand name and tagline are artwork and SHALL read the same in the four documentation languages.

#### Scenario: Same livery on every page
- **WHEN** a reader goes from the Workshop pages to the Brand and Export pages
- **THEN** every screenshot and animation that shows a finished paint job shows the Blue Wave Biarritz livery, on the sample truck or the sample trailer

#### Scenario: A feature demo keeps its own scene
- **WHEN** a reader opens the screenshot of gradients or of shapes and paths
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
