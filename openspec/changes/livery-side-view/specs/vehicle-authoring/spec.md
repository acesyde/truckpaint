## ADDED Requirements

### Requirement: Sample preview meshes
Every main texture of the sample truck (both versions) and of the sample trailer SHALL have a hand-made preview mesh (see the vehicle-packages capability), made of a few boxes whose faces map the labelled panels of its template: for the cab, the sides, front, rear and roof; for the trailer's Base, the chassis frame and the front wall. The meshes SHALL be original work like the templates, kept as sources next to them and packed into the committed packages. A test SHALL fail when a face of a sample mesh does not lie inside a labelled panel of its template, so a template change cannot leave its mesh behind unnoticed.

#### Scenario: Sample truck in the Side View
- **WHEN** a painter opens the Standard cab of the sample truck and shows the Side View from the left
- **THEN** a cab shape is drawn with the artwork of the cab's left side panel on its left face

#### Scenario: Template panel moved
- **WHEN** a panel of the Standard cab template is moved and the preview mesh is not updated
- **THEN** the test suite fails and names the sample and the texture
