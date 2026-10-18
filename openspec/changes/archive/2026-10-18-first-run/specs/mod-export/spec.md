## MODIFIED Requirements

### Requirement: Destination
The dialog SHALL show the destination: the full path of the file the mod will be written to, in the monospace face, shortened from its start with an ellipsis when it doesn't fit (so the file name stays visible), and shown in full on hover. When the dialog opens, it SHALL propose "<Name>.scs", with the characters that can't be used in file names replaced by `-`, in the mod folder chosen at first launch for the game when there is one and it exists, else in the game's mod folder when that folder exists:

| | Euro Truck Simulator 2 | American Truck Simulator |
|---|---|---|
| Windows | `Documents\Euro Truck Simulator 2\mod` | `Documents\American Truck Simulator\mod` |
| macOS | `~/Library/Application Support/Euro Truck Simulator 2/mod` | `~/Library/Application Support/American Truck Simulator/mod` |
| Linux | `~/.local/share/Euro Truck Simulator 2/mod` | `~/.local/share/American Truck Simulator/mod` |

Otherwise it SHALL propose the folder of the last mod export of the session, or else the user's Documents folder (their home folder when there is none).

**Change…** SHALL open the native save dialog in the destination's folder, proposing its file name. Choosing a file SHALL make it the destination (with `.scs` added when the user left it out); cancelling SHALL keep the destination.

**Export** SHALL write to the destination. When a file already exists there, the dialog SHALL first ask whether to replace it, naming the file, with **Replace** and **Cancel**; Cancel SHALL write nothing and keep the dialog open. A file chosen with Change… whose replacement the native save dialog already confirmed SHALL NOT be asked about again.

#### Scenario: Exporting into the game
- **WHEN** the user opens Export Mod… on an ETS2 project named "ACE Logistics" on Windows, with the folder `Documents\Euro Truck Simulator 2\mod` present
- **THEN** the dialog shows the destination `Documents\Euro Truck Simulator 2\mod\ACE Logistics.scs`, and Export writes the mod there

#### Scenario: Mod folder chosen at first launch
- **WHEN** the player chose `D:\Mods\ETS2` as the ETS2 mod folder at first launch and opens Export Mod… on an ETS2 project named "ACE Logistics"
- **THEN** the destination is `D:\Mods\ETS2\ACE Logistics.scs`

#### Scenario: No game folder
- **WHEN** no ATS mod folder exists and the user opens Export Mod… on an ATS project named "Blue Line" for the first time in the session
- **THEN** the destination is "Blue Line.scs" in the user's Documents folder

#### Scenario: Changing the destination
- **WHEN** the user clicks Change… and chooses `ace` in another folder
- **THEN** the destination shows `ace.scs` in that folder, and Export writes the mod there

#### Scenario: Cancelling Change…
- **WHEN** the user clicks Change… and cancels the save dialog
- **THEN** the destination is unchanged and nothing is written

#### Scenario: Replacing an existing file
- **WHEN** the proposed destination "ACE Logistics.scs" already exists and the user clicks Export
- **THEN** the dialog asks whether to replace "ACE Logistics.scs"; Cancel writes nothing and keeps the dialog open, Replace starts the export

