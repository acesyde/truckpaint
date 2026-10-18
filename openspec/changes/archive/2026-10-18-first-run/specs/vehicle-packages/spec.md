## MODIFIED Requirements

### Requirement: Built-in sample vehicle
TruckPaint SHALL include the newest versions of the sample truck and the sample trailer (see the vehicle-authoring capability). When no vehicle is installed, an **Install the sample vehicles** button SHALL appear in three places:
- the empty state of the Vehicle Library;
- the vehicle list of New Project;
- the Vehicles to paint step of the first launch, when the game is not found or is not supported for generation yet.

The button SHALL install both samples like any other package, with the same confirmation and the same errors. Once a vehicle is installed, the button SHALL no longer be shown. Installing the samples SHALL need no network access.

#### Scenario: First vehicle project
- **WHEN** a painter with no vehicle installed opens New Project and clicks Install the sample vehicles
- **THEN** "TruckPaint Sample Truck" and "TruckPaint Sample Trailer" appear in the list with their newest versions, a confirmation names them, and either can be picked to create a project

#### Scenario: First launch without a game
- **WHEN** the first launch reaches Vehicles to paint and the game installation is not found
- **THEN** Install the sample vehicles is offered and installs both samples with no network access

#### Scenario: Library with vehicles
- **WHEN** at least one vehicle is installed
- **THEN** neither the Vehicle Library nor New Project shows Install the sample vehicles

