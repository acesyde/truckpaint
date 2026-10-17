## ADDED Requirements

### Requirement: Objects to reposition
In the Layers tab, an object that a template update flagged (see the template-update-impact capability) SHALL be marked in its row with an icon and the label "To reposition" in its name for assistive technologies, never by color alone; resting the pointer on the mark SHALL say that the texture's layout changed in the version. A group SHALL show the mark when any object inside it is flagged. The header of the tab SHALL NOT change. A mark SHALL disappear when its object is moved or transformed, or when the texture is marked as checked, and SHALL come back with Undo.

#### Scenario: Flagged object marked
- **WHEN** Update Template flagged "Standard cab" "Layout changed" while it held the logo "Ace logo" and the user opens the Layers tab on it
- **THEN** the row "Ace logo" shows the "To reposition" mark, and so does every other object that was on the texture

#### Scenario: Moving clears one mark
- **WHEN** the user moves "Ace logo" on that texture
- **THEN** its mark disappears and the others stay; Undo brings it back

#### Scenario: Group
- **WHEN** a group holds a flagged logo and the group is collapsed
- **THEN** the group's row shows the mark

#### Scenario: Object drawn after the update
- **WHEN** the user draws a rectangle on the flagged texture after the update
- **THEN** its row has no mark
