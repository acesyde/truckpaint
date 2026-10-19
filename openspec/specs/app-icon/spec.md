# app-icon Specification

## Purpose
Defines the icon of the TruckPaint app: where players see it, what its picture files are, and what happens when the picture cannot be read.

## Requirements

### Requirement: The app has an icon
TruckPaint SHALL use its icon as the icon of its window and as its icon in the taskbar or task switcher (Windows and Linux, where the system lets a program set it from code) and in the Dock (macOS), whichever way the title bar is shown (native, drawn or system). The icon SHALL show a front view of a semi-truck cab, in off-white with a red livery stripe, on a dark rounded square.

#### Scenario: Dock on macOS
- **WHEN** TruckPaint is running on macOS
- **THEN** the Dock shows the TruckPaint icon, not the generic icon of a program

#### Scenario: Taskbar on Windows
- **WHEN** TruckPaint is running on Windows
- **THEN** its window and its taskbar button show the TruckPaint icon

#### Scenario: Every title bar mode
- **WHEN** the title bar is set to the native, the drawn or the system one
- **THEN** the icon is shown in the taskbar or the Dock in each of them

### Requirement: Icon pictures
The repository SHALL hold the icon as a square PNG master of 1024 px with transparency, where the rounded square fills about 80 % of the canvas, is centered, and leaves a transparent margin. It SHALL also hold PNGs of 512, 256, 128, 64, 32 and 16 px made from the master by reduction, with the same picture. A note next to the files SHALL record where the picture comes from and the commands that make the sizes.

#### Scenario: Files of the icon
- **WHEN** a contributor opens the icon folder
- **THEN** it holds the 1024 px master, the six smaller sizes and the note

#### Scenario: Transparent margin
- **WHEN** the master is read pixel by pixel
- **THEN** the four corner pixels are fully transparent and the center pixel is opaque (an alpha of at least 250 out of 255)

### Requirement: Icon content
The icon picture SHALL NOT contain text, nor the logo or badge of a vehicle maker, nor the logo of a game or of its studio.

#### Scenario: Generic truck
- **WHEN** the icon is looked at at 1024 px
- **THEN** it shows a generic truck cab with no text and no logo

### Requirement: The icon is part of the program
The icon SHALL be built into the TruckPaint program, so that every download shows it without any file next to the program, and the same picture SHALL be used on every platform.

#### Scenario: A download on its own
- **WHEN** the program of a download is copied alone to another folder and started
- **THEN** it still shows the TruckPaint icon

### Requirement: A broken icon never stops the app
When the icon data cannot be read, TruckPaint SHALL start without an icon and SHALL write a warning in its log.

#### Scenario: Unreadable picture
- **WHEN** the built-in icon data is not a valid picture
- **THEN** TruckPaint starts normally with the system's generic icon and the log holds a warning
