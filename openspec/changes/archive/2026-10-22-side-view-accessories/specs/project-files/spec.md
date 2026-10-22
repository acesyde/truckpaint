## ADDED Requirements

### Requirement: Side View accessory choices
A project SHALL record, for each vehicle, the option chosen in each accessory slot of the Side View (see the side-view capability), None included, and SHALL reopen with the same choices. A vehicle with no recorded choice SHALL use the game's default configuration. A recorded option that the installed package no longer has SHALL fall back to that slot's default, without a message. Changing a choice SHALL mark the project as changed; it SHALL NOT be an undo step.

#### Scenario: Choices reopen
- **WHEN** the user picks None for the DAF XD's roof deflector, saves, closes and reopens the project
- **THEN** the Side View shows the DAF XD without its roof deflector

#### Scenario: Option gone after a package update
- **WHEN** a project records the bumper option `robust_p` and the vehicle's new package has no such option
- **THEN** the bumper slot shows its default option

#### Scenario: Not an undo step
- **WHEN** the user moves a logo, then picks None for the roof deflector, then chooses Edit › Undo
- **THEN** the logo goes back and the roof deflector stays at None
