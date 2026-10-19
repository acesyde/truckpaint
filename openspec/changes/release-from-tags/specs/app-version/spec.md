## Purpose

Defines the version number of the app: where it comes from, where players and tools can read it, and what a build that is not a release shows.

## ADDED Requirements

### Requirement: One version for the whole app
The app SHALL have a single version number, and every place that shows or reports it (the About dialog, the startup log, the MCP server information and the vehicle extraction report) SHALL show that same value.

#### Scenario: Same everywhere
- **WHEN** the app is built from the tag `v0.2.0`
- **THEN** About, the log, the MCP server information and the extraction report all say `0.2.0`

### Requirement: The version comes from the release tag
A build made for a release or pre-release SHALL carry the version of its tag without the leading `v` (`0.2.0`, `0.2.0-beta.1`), whatever the version written in the project's manifest.

#### Scenario: Pre-release version
- **WHEN** the downloaded app comes from `v0.2.0-beta.1`
- **THEN** About shows `Version 0.2.0-beta.1`

#### Scenario: Manifest is not the app's version
- **WHEN** the manifest says `0.1.0` and the tag is `v0.3.0`
- **THEN** the downloaded app says `0.3.0`

### Requirement: Development builds
A build that is not made for a release (for example `mise run run`) SHALL show `dev` as its version, in every place of the first requirement.

#### Scenario: Local run
- **WHEN** a contributor runs the app from the source tree
- **THEN** About shows `Version dev`

### Requirement: Version in About
The About dialog SHALL show the version under the name of the app, with its label in the four languages of the app. The number itself SHALL be shown as is: it is not translated or reformatted.

#### Scenario: About in French
- **WHEN** the language is French and About is open in a release build `0.2.0`
- **THEN** the dialog shows the French label for the version followed by `0.2.0`
