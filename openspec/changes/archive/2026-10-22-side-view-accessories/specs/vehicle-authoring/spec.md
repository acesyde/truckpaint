## MODIFIED Requirements

### Requirement: Sample preview meshes
Every main texture of the sample truck (both versions) and of the sample trailer SHALL have a hand-made preview mesh (see the vehicle-packages capability), made of a few boxes whose faces map the labelled panels of its template: for the cab, the sides, front, rear and roof; for the trailer's Base, the chassis frame and the front wall. The sample truck SHALL also have a `sideskirt` slot (version 1.1.0, its Side skirts texture) and the sample trailer a `body` slot with the Curtain body 13.6 m (default) and 10.5 m options, each a box whose faces map the labelled panels of its own texture's template. The meshes SHALL be original work like the templates, kept as sources next to them and packed into the committed packages. A test SHALL fail when a face of a sample mesh does not lie inside a labelled panel of the template of the texture that paints it, so a template change cannot leave its mesh behind unnoticed.

#### Scenario: Sample truck in the Side View
- **WHEN** a painter opens the Standard cab of the sample truck and shows the Side View from the left
- **THEN** a cab shape is drawn with the artwork of the cab's left side panel on its left face

#### Scenario: Template panel moved
- **WHEN** a panel of the Standard cab template is moved and the preview mesh is not updated
- **THEN** the test suite fails and names the sample and the texture

#### Scenario: Sample trailer with its curtain
- **WHEN** a painter shows the Side View of the sample trailer from the left
- **THEN** the chassis and the 13.6 m curtain body are drawn, the curtain with the artwork of the Curtain body 13.6 m texture
