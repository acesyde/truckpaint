## ADDED Requirements

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
