# workspace-layout Specification

## Purpose

Defines the editor workspace layout (menus, tools, sidebar, canvas area, panels and status bar) that all editing features plug into.

## Requirements

### Requirement: Menu bar
The application SHALL offer the menus File, Edit, Object, Layer, View, Vehicle and Help, in that order, on the home screen and in every space. Where they are shown depends on the platform (see the window-title-bar capability):
- **macOS:** in the system menu bar at the top of the screen, after the application menu **TruckPaint**, and followed by the system's **Window** menu; nothing is drawn in the window;
- **Windows and Linux:** in the window's title bar, or in a row inside the window with the system title bar option.

Each menu item SHALL display its shortcut (when it has one) in the platform's notation, be enabled and disabled as its command is, and show its check mark when it is a toggle, wherever the menu is shown. Items whose feature is not yet available SHALL be shown disabled. Changing the language SHALL relabel the menus at once, including the macOS menu bar.

The File menu SHALL end with **Export Texture…** and **Export Mod…** before Quit: there is no Export menu. Shortcuts don't change.

On macOS:
- the application menu SHALL hold **About TruckPaint**, **Settings…** (`Cmd+,`, the Preferences command), the system's Services, Hide TruckPaint, Hide Others and Show All, and **Quit TruckPaint** (`Cmd+Q`); About and Preferences SHALL NOT be repeated in Help and Edit, nor Quit in File;
- Quit SHALL ask to save unsaved changes as closing the window does;
- a shortcut shown in the menu SHALL run its command once, whether the key or the menu item is used;
- while a text is being typed in a field or on the canvas, Edit › Undo, Redo, Cut, Copy, Paste and Select All and their shortcuts SHALL act on that text, as they do without the system menu.

The View menu SHALL list, in groups: Command Palette… (see the command-palette capability); the spaces Project, Workshop and Brand; the left tabs Textures, Layers and Resources; Hide Panels; then Show Template, Show Grid, Show Guides, Clear Guides and Snapping; then Zoom In, Zoom Out, Fit to Screen and Actual Size; then Reset Workspace. Toggles SHALL show their state with a check mark. The Vehicle menu SHALL NOT have a Vehicle Information item, and the View menu SHALL NOT have Sidebar nor panel items: the spaces and the left tabs replace them.

The menus' structure (their items, submenus and groups) SHALL be the one the command palette uses to show each command's menu path, so that a command's path in the palette always names the menu, and submenu, where the command is.

#### Scenario: Menus present
- **WHEN** a project is open
- **THEN** the menus are File, Edit, Object, Layer, View, Vehicle, Help in that order, with no Export menu

#### Scenario: Closing a project from the File menu
- **WHEN** the user chooses File > Close
- **THEN** the project closes and the home screen is shown

#### Scenario: View menu
- **WHEN** the user opens the View menu in the Workshop with the Layers tab shown
- **THEN** it lists Command Palette… with its shortcut, then Project, Workshop (checked) and Brand, then Textures, Layers (checked) and Resources, then Hide Panels, then the template, grid, guides and snapping items, then the zoom commands, then Reset Workspace

#### Scenario: Command Palette from the View menu on the home screen
- **WHEN** no project is open and the user chooses View › Command Palette…
- **THEN** the command palette opens

#### Scenario: Palette path follows the menu
- **WHEN** the user types "align left" in the command palette
- **THEN** the Align Left row's path reads "Object › Align", the menu and submenu where Align Left is

#### Scenario: No sidebar item
- **WHEN** the user opens the Vehicle menu
- **THEN** it has no Vehicle Information item, and pressing F5 does nothing

#### Scenario: Export items in File
- **WHEN** a project is open and the user opens the File menu
- **THEN** it lists Export Texture… with Shift+Cmd/Ctrl+E and Export Mod… with Cmd/Ctrl+E, before Quit

#### Scenario: System menu bar on macOS
- **WHEN** a project is open on macOS
- **THEN** the system menu bar shows TruckPaint, File, Edit, Object, Layer, View, Vehicle, Window and Help, and the window draws no menu row

#### Scenario: Shortcut runs once on macOS
- **WHEN** a rectangle is selected on macOS and the user presses Cmd+D
- **THEN** exactly one duplicate is made

#### Scenario: Disabled state follows the command
- **WHEN** nothing is selected on macOS
- **THEN** Edit › Duplicate is disabled in the system menu bar, and pressing Cmd+D does nothing

#### Scenario: Copy in a text field on macOS
- **WHEN** the user selects the text of the project name field in the Project space and presses Cmd+C, then Cmd+V in another field
- **THEN** the name is pasted in the other field, and no object of the canvas is copied

#### Scenario: Quit with unsaved changes on macOS
- **WHEN** a project has unsaved changes and the user chooses TruckPaint › Quit TruckPaint
- **THEN** the save prompt is shown, and Cancel keeps the application open

#### Scenario: Language switch relabels the system menu
- **WHEN** the user picks "Deutsch" in Preferences on macOS
- **THEN** the system menu bar reads Datei, Bearbeiten, Objekt, Ebene, Ansicht, Fahrzeug and Hilfe at once

### Requirement: Tool rail
The Workshop SHALL show a vertical tool rail, 48 px wide, on the left, with, in order: Selection, Direct Selection, Move, Rectangle, Ellipse, Polygon, Pen, Line, Text, Image, Eyedropper, Gradient, Zoom, Hand. Each tool SHALL be shown as an icon without text; hovering it SHALL show a tooltip with its translated name and its shortcut in the platform's notation (the Gradient tool's shortcut is Shift+G), and each tool SHALL announce its name to assistive technologies. Exactly one tool SHALL be active at a time and the active tool SHALL be identifiable without relying on color alone. Clicking a tool SHALL make it active. The tool rail SHALL stay shown when the panels are hidden.

#### Scenario: Default tool
- **WHEN** a project is opened
- **THEN** the Selection tool is active

#### Scenario: Switching tools
- **WHEN** the user clicks the Rectangle tool
- **THEN** the Rectangle tool shows the active state and the previously active tool no longer does

#### Scenario: Tooltip
- **WHEN** the user hovers the Gradient tool with the interface in French
- **THEN** a tooltip shows its French name and the shortcut Shift+G

### Requirement: Canvas area
The canvas area SHALL occupy, in the Workshop, all space not used by the bars, the tool rail, the left panel and the inspector, SHALL show rulers along its top and left edges (see `canvas-guides`), and SHALL display the project's active texture surface as an artboard framed against a neutral pasteboard background, with the breadcrumb inlaid (see the workspace-spaces capability). When a project is opened, the view SHALL be fitted so the whole artboard is centered and visible; afterwards the view is controlled by the user (see `canvas-navigation`). Resizing the canvas area, including by resizing, hiding or showing the left panel or the inspector, SHALL keep the document point at the center of the canvas area fixed and SHALL NOT change the zoom level, unless the user has not navigated since the last fit, in which case the view is refitted.

#### Scenario: Artboard visible
- **WHEN** a new 4096×4096 project is opened and the Workshop is shown
- **THEN** a square artboard is displayed centered in the canvas area and entirely visible

#### Scenario: Window resize
- **WHEN** the window is resized and the user has not zoomed or panned since the project was opened or last fitted
- **THEN** the view is refitted so the artboard remains centered and fully visible

#### Scenario: Window resize after navigating
- **WHEN** the user has zoomed to 200% and then resizes the window
- **THEN** the zoom stays at 200% and the document point that was at the center of the canvas area stays at its center

#### Scenario: Hiding the panels after navigating
- **WHEN** the user has zoomed to 200% and then presses Tab to hide the panels
- **THEN** the canvas area widens, the zoom stays at 200% and the document point that was at its center stays at its center

#### Scenario: Rectangular artboard
- **WHEN** a project whose active texture is 8192 × 1024 is opened in the Workshop
- **THEN** the artboard is eight times wider than tall, centered and entirely visible, and the rulers run from 0 to 8192 across and from 0 to 1024 down

### Requirement: Status bar
The workspace SHALL show a status bar, 28 px high, in every space.

In the Workshop it SHALL display:
- the current zoom level;
- the pointer position in texture pixels when over the artboard;
- the template settings of the active texture: a Template toggle, its opacity, and the key G (see the vehicle-projects capability);
- toggles for Snapping, Grid and Guides, which do the same as View › Snapping, View › Show Grid and View › Show Guides and show their state without relying on color alone;
- the document save state.

In the Project and Brand spaces it SHALL display only the save state.

The save state SHALL be shown as text ("Saved" / "Unsaved changes") together with an icon, never by color alone.

#### Scenario: Pointer coordinates
- **WHEN** the pointer moves over the artboard
- **THEN** the status bar shows the pointer position in texture pixel coordinates

#### Scenario: Save state indicator for a new project
- **WHEN** a new project has just been created and never saved
- **THEN** the status bar shows "Unsaved changes" with its icon

#### Scenario: Grid toggle
- **WHEN** the grid is hidden and the user clicks Grid in the status bar
- **THEN** the grid is shown on the canvas, the Grid toggle shows its on state, and View › Show Grid is checked

#### Scenario: Status bar in the Brand space
- **WHEN** the Brand space is shown
- **THEN** the status bar shows only the save state

### Requirement: Workspace frame
The window of an open project SHALL be arranged, from top to bottom, as: the window's title bar holding the top bar (see the window-title-bar and workspace-spaces capabilities; with the system title bar option, the menu row and then the top bar), in the Workshop the tool options bar, then the space's content, then the status bar.

In the Workshop, the content SHALL be, from left to right: the tool rail, the left panel, the canvas area and the inspector. The Project and Brand spaces SHALL use the full width between the top bar and the status bar, without tool rail, tool options bar, left panel nor inspector.

View › Reset Workspace SHALL show the left panel and the inspector at their default widths, with the Textures tab.

#### Scenario: Workshop frame
- **WHEN** the Workshop is shown
- **THEN** the tool rail, the left panel, the canvas area and the inspector are shown from left to right, between the tool options bar and the status bar

#### Scenario: Project frame
- **WHEN** the Project space is shown
- **THEN** no tool rail, tool options bar, left panel nor inspector is shown, and the status bar shows only the save state

#### Scenario: Reset Workspace
- **WHEN** the panels are hidden, the Layers tab is shown and the inspector was widened, and the user chooses View › Reset Workspace
- **THEN** the left panel and the inspector are shown at their default widths and the Textures tab is shown

### Requirement: Tool options bar
In the Workshop, a tool options bar SHALL run under the top bar and show the settings of the active tool, labelled in plain words:
- **Polygon:** a Sides field (3 to 12), a Star toggle and, when Star is on, an Inner radius field (10% to 90%). They set the sides and star settings used for new polygons. When every selected object is a polygon, a change SHALL also apply to them as in the path-tools capability (Polygon settings), one undo step per change, and differing values SHALL show "Mixed". The inspector's Polygon section edits the same settings (see the properties-panel capability), and both places SHALL always show the same values.
- **Every other tool:** the tool's name only; no other setting is shown.

The tool options bar SHALL stay shown when the panels are hidden.

#### Scenario: Polygon settings for new polygons
- **WHEN** the Polygon tool is active, nothing is selected, and the user sets Sides to 5 and turns Star on in the tool options bar
- **THEN** the next polygon drawn is a five-point star, and no undo step was recorded by the settings

#### Scenario: Editing selected polygons
- **WHEN** two hexagons are selected, the Polygon tool is active and the user types 8 in Sides
- **THEN** both become octagons in their bounds, and one Undo restores both hexagons

#### Scenario: Tool without settings
- **WHEN** the Hand tool is active
- **THEN** the tool options bar shows "Hand" and no setting

### Requirement: Left panel
In the Workshop, a left panel between the tool rail and the canvas area SHALL show one of three tabs, in this order: **Textures** (see the vehicle-projects capability), **Layers** (see the layers-panel capability) and **Resources** (see the color-panel, symbols, shared-styles and assets-panel capabilities). The shown tab SHALL be identifiable without relying on color alone.

The Resources tab SHALL list, in this order, compact sections for the project's **Palette**, **Symbols**, **Styles** and **Images**, each headed by its name and number of elements, with the Import from Library… button (see the shared-library capability), and a footer pinned under the list saying that an element is dragged onto the canvas to place it and that the Brand space manages them all. The Palette section's swatches SHALL behave as the brand palette of the color popover (see the color-panel capability, Project palette): clicking a swatch applies its color to the current target, or to the selected stop when the target is a gradient, and links it to the swatch; the swatch linked to the current target is marked the same way; and each swatch's context menu offers Edit Swatch…, Delete Swatch and Add to Library or Update in Library.

Clicking a tab, View › Textures (1), View › Layers (2) and View › Resources (3) SHALL show that tab. Choosing a tab from the View menu or its key while another space is shown SHALL show the Workshop with that tab, and while the panels are hidden SHALL show them again. The single keys 1, 2 and 3 SHALL act only when no text field has keyboard focus, no text is being edited on the canvas and no modal dialog is open; otherwise they type as usual.

The panel's width SHALL be resizable by dragging its edge, clamped between a minimum and a maximum width. The shown tab and the width SHALL be remembered across sessions; the first launch shows the Textures tab.

#### Scenario: Tabs from the keyboard
- **WHEN** the Workshop is shown with the Textures tab and the user presses 2
- **THEN** the Layers tab is shown, listing the active texture's layers

#### Scenario: Keys in a text field
- **WHEN** the user types "2" in the Opacity field of the inspector
- **THEN** the field receives "2" and the shown tab does not change

#### Scenario: Resources tab sections
- **WHEN** the Resources tab is shown for a project with a palette, symbols, styles and images
- **THEN** it shows the Palette, Symbols, Styles and Images sections in that order

#### Scenario: Applying a palette swatch from the Resources tab
- **WHEN** a rectangle is selected with the fill as target, and the user clicks the swatch "Company red" in the Palette section of the Resources tab
- **THEN** the rectangle's fill takes that color, linked to "Company red", the Fill row reads "Company red" with a link icon, and one Undo restores the previous fill

#### Scenario: Swatch context menu in the Resources tab
- **WHEN** the user right-clicks the swatch "Company red" in the Resources tab
- **THEN** the context menu offers Edit Swatch…, Delete Swatch and Add to Library, as in the color popover

#### Scenario: Resizing the left panel
- **WHEN** the user drags the right edge of the left panel
- **THEN** its width changes live, clamped between a minimum and a maximum, and the canvas area takes the rest

#### Scenario: Tab remembered
- **WHEN** the user shows the Resources tab, quits and reopens a project
- **THEN** the Workshop shows the Resources tab

### Requirement: Inspector
In the Workshop, an inspector on the right of the canvas area SHALL show the settings of the selection, or of the active texture when nothing is selected; its content is specified by the properties-panel capability. It SHALL never be shown as an empty panel. Its width SHALL be resizable by dragging its left edge, clamped between a minimum and a maximum width, and remembered across sessions.

#### Scenario: Resizing the inspector
- **WHEN** the user drags the left edge of the inspector
- **THEN** its width changes live, clamped between a minimum and a maximum width, and it keeps that width after a restart

#### Scenario: Nothing selected
- **WHEN** the Workshop is shown and nothing is selected
- **THEN** the inspector shows the active texture's properties

### Requirement: Focus mode
View › Hide Panels (Tab) SHALL hide the left panel and the inspector of the Workshop so the canvas area takes their place; choosing it again SHALL show them at their previous widths. The menu bar, the top bar, the tool options bar, the tool rail and the status bar SHALL stay. The canvas view follows the resizing rule of the Canvas area requirement. Whether the panels are hidden SHALL be remembered across sessions. Hide Panels SHALL be enabled only in the Workshop.

Tab SHALL toggle the panels only when no text field has keyboard focus, no text is being edited on the canvas and no modal dialog is open; otherwise Tab moves the keyboard focus as usual, including inside dialogs.

#### Scenario: Hiding the panels
- **WHEN** the Workshop is shown and the user presses Tab
- **THEN** the left panel and the inspector are hidden, the tool rail, the bars and the breadcrumb on the canvas stay, and pressing Tab again shows the panels

#### Scenario: Tab in a field
- **WHEN** the keyboard focus is in the X field of the inspector and the user presses Tab
- **THEN** the focus moves to the next field and the panels stay shown

#### Scenario: Tab in a dialog
- **WHEN** the Export Mod dialog is open and the user presses Tab
- **THEN** the focus moves to the next control of the dialog and the panels behind it do not change

### Requirement: Canvas and tool context menus
Right-clicking in the canvas area and on a tool of the tool rail SHALL open a context menu showing the commands relevant to that location, with their shortcuts.

#### Scenario: Context menu on a tool
- **WHEN** the user right-clicks the Pen tool
- **THEN** a context menu offers the Pen tool with its shortcut and Keyboard Shortcuts

### Requirement: MCP state in the status bar
While the MCP server is on (see the mcp-server capability), the status bar SHALL show, in every space, an item with an icon and a text: "MCP" when no client has called in the last minute, and "MCP · <n> changes" when a client has, where n is the number of write calls applied in the last minute (or "MCP · connected" when they were only reads). Its tooltip SHALL give the server's URL. Clicking it SHALL open Preferences on the AI assistant tab. The item SHALL NOT be shown while the server is off, and its state SHALL NOT rely on color alone.

#### Scenario: Server on, idle
- **WHEN** the server is on and no client has called yet
- **THEN** the status bar shows the MCP item with "MCP"

#### Scenario: Client drawing
- **WHEN** a client has added three shapes in the last minute
- **THEN** the status bar shows "MCP · 3 changes"

#### Scenario: Server off
- **WHEN** the server is off
- **THEN** the status bar shows no MCP item

#### Scenario: Click opens the AI assistant tab
- **WHEN** the server is on and the user clicks the MCP item in the status bar
- **THEN** Preferences opens on the AI assistant tab, with the switch, the address and the port in view
