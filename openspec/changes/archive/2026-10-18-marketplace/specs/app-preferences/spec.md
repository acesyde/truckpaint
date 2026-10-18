## ADDED Requirements

### Requirement: Marketplace address
Preferences SHALL have an optional **Catalog address** for the Marketplace, empty by default (the official catalog is used). The environment variable `TRUCKPAINT_CATALOG_URL`, when set, SHALL take precedence over it. An address that is not HTTPS (or a local `file://` path, for testing) SHALL be refused with a message. Clearing the field SHALL return to the official catalog.

#### Scenario: Custom address
- **WHEN** the player enters an HTTPS address and opens the Marketplace tab
- **THEN** the catalog is read from that address

#### Scenario: Environment wins
- **WHEN** the setting and `TRUCKPAINT_CATALOG_URL` are both set
- **THEN** the environment variable's address is used, and Preferences says so next to the field

#### Scenario: Refused address
- **WHEN** the player enters `http://example.com/catalog.json`
- **THEN** the field shows an error and the previous address stays in use
