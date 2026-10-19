## ADDED Requirements

### Requirement: File dialogs keep the window alive
Every native file dialog of the application (Save As, Open, Export Texture, Export Mod, a vehicle package export or install, Place, the template, mod picture and folder choices) SHALL leave the application window drawing and answering the system while the dialog is open, on every operating system, so that the system never reports the application as not responding. While a file dialog is open, a small box "Waiting for the file dialog…" SHALL show over the window, and menus, shortcuts and other commands SHALL do nothing; a second file dialog SHALL NOT open. The box SHALL disappear when the dialog closes, whatever the answer.

The answer of a dialog SHALL be used once, only by what asked for it: choosing a file continues that action, cancelling continues nothing. An answer for something that is no longer there (its dialog closed while the file dialog was open) SHALL be dropped without a message.

#### Scenario: A long wait
- **WHEN** the user opens Save As and takes a minute to choose a name
- **THEN** the system never reports the application as not responding, the box "Waiting for the file dialog…" is shown, and choosing a file saves the project to it

#### Scenario: Commands during the dialog
- **WHEN** a file dialog is open and the user presses Cmd/Ctrl+N or Cmd/Ctrl+W in the window
- **THEN** nothing happens, and no second dialog opens

#### Scenario: Cancelling
- **WHEN** the user cancels a file dialog
- **THEN** the box disappears and nothing else changes

#### Scenario: Save before closing
- **WHEN** the user closes a project with unsaved changes, answers Save, and cancels the Save As dialog
- **THEN** the project stays open with its changes, and nothing is closed or quit

#### Scenario: Save before closing, file chosen
- **WHEN** the user closes a project with unsaved changes, answers Save, and chooses a file in the Save As dialog
- **THEN** the project is saved to it and then closed

#### Scenario: Dialog closed while waiting
- **WHEN** the user closes Export Texture while its save dialog is open and then chooses a file
- **THEN** nothing is exported and no message is shown
