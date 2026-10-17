## Purpose
Lets a player find and install vehicle packages from the public catalog inside TruckPaint, and see when an installed vehicle has a newer version, without searching for files.

## ADDED Requirements

### Requirement: Marketplace tab
The Vehicle Library SHALL have a **Marketplace** tab beside Installed and My vehicles. It SHALL list the catalog's vehicles, one row per vehicle at its newest version compatible with this app, with a search field (name and brand), game and kind filters with counts, and per row: name, brand, kind, game, version, download size, and a status as text with an icon: **Install**, **Update** (a lower version is installed), or **Installed**. Versions that need a newer app SHALL be left out of the list, and a vehicle with only such versions SHALL appear as "Needs a newer TruckPaint".

The detail pane SHALL show the preview, the authors, the license, the changelog and the older versions available.

#### Scenario: Browsing
- **WHEN** the player opens the Marketplace tab with a reachable catalog
- **THEN** the vehicles are listed with their status, and typing "volvo" keeps those whose name or brand matches

#### Scenario: Update available
- **WHEN** version 1.1.0 of a truck is installed and the catalog has 1.2.0
- **THEN** the row says Update, and the Installed tab is unchanged until the update is installed

#### Scenario: Too new for this app
- **WHEN** the newest version of a vehicle has `min_app` above the running app
- **THEN** the list offers the newest version the app can install, or says "Needs a newer TruckPaint" if there is none

### Requirement: Install from the catalog
Install and Update SHALL download the package, check that its size and SHA-256 match the entry, then install it with the same validation as a file install. Several vehicles MAY be selected and installed together, with progress and a Cancel. On any failure (network, size, checksum, validation) nothing SHALL be installed, and a message SHALL name the vehicle and the reason. A confirmation SHALL name each vehicle and version installed. The download SHALL be refused beyond a size limit, and when the address is not HTTPS.

#### Scenario: Successful install
- **WHEN** the player installs a catalog vehicle
- **THEN** it appears in the Installed tab and in New Project, and the row says Installed

#### Scenario: Checksum mismatch
- **WHEN** the downloaded file's SHA-256 differs from the catalog's
- **THEN** nothing is installed and the message says the download is damaged or was changed

#### Scenario: Cancel
- **WHEN** the player cancels while several packages are downloading
- **THEN** the ones already installed stay, and the others are not installed

### Requirement: Cache and offline
The catalog SHALL be downloaded when the Marketplace tab is first opened in a session and when the player chooses **Refresh**, never in the background. The last good catalog SHALL be kept on disk. When the catalog can't be reached, the tab SHALL show the cached catalog with "Offline, last updated <date>" and Install disabled for packages not already downloaded; with no cache, it SHALL say the catalog can't be reached and offer to retry. A damaged or unknown-format response SHALL never replace a good cache.

#### Scenario: Offline with a cache
- **WHEN** the network is down and a cached catalog exists
- **THEN** the tab lists the cached vehicles with the date it was last updated

#### Scenario: Offline without a cache
- **WHEN** the network is down and nothing was ever downloaded
- **THEN** the tab says the catalog can't be reached, offers Retry, and the rest of the Vehicle Library works

#### Scenario: Bad response keeps the cache
- **WHEN** the server returns text that isn't a valid catalog
- **THEN** the cached catalog is kept and the tab shows an error

### Requirement: Privacy and responsiveness
The app SHALL contact the catalog's server only for Marketplace actions the player starts (opening the tab, Refresh, Install). Requests SHALL carry no identifier beyond what a download needs, and SHALL not block the interface; they SHALL time out.

#### Scenario: Nothing at startup
- **WHEN** TruckPaint starts and the Marketplace tab is never opened
- **THEN** no request is sent to the catalog

### Requirement: Legal notice
The Marketplace tab SHALL show, always visible and in the player's language, a notice saying that the content of game-derived vehicles belongs to SCS Software, that TruckPaint and the Marketplace are unofficial and not endorsed by SCS Software, and that the packages are for personal modding and not for sale. A vehicle whose catalog entry has a `notice` SHALL show it in the detail pane. Installing SHALL not require accepting the notice.

#### Scenario: Notice shown
- **WHEN** the player opens the Marketplace tab
- **THEN** the notice is visible above the list without scrolling

#### Scenario: Entry notice
- **WHEN** a vehicle's entry has a `notice`
- **THEN** the detail pane shows its text
