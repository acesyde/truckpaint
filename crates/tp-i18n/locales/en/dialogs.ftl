
## Export sizes and errors

size-mb = { $value } MB
size-kb = { $value } KB
size-about = about { $value }
export-stopped = the export stopped unexpectedly

## Saving and opening projects

file-save-failed-title = Could not save
file-save-failed = { $file } could not be saved.
file-save-failed-reason = { $file } could not be saved ({ $reason }). Your changes are still open.
file-open-failed-title = Cannot open project
file-restore-failed-title = Cannot restore project
file-recovered-copy = The recovered copy
file-not-a-project = { $file } is not a TruckPaint project.
file-damaged = { $file } is damaged and cannot be opened.
file-newer-version = { $file } was created with a newer version of TruckPaint.
file-io = { $file } could not be read or written ({ $reason }).

## File dialogs

dialog-open-project = Open Project
dialog-save-project = Save Project
dialog-export-texture = Export Texture
filter-project = TruckPaint project
filter-images = Images

## Home screen

home-recovered = Recovered projects
home-recovered-hint = TruckPaint closed unexpectedly. These projects had unsaved changes.
home-never-saved = Never saved
home-recovery-damaged = Damaged recovery copy
home-recovery-damaged-hint = This copy cannot be restored.
home-recovered-item = Recovered { $name }
home-restore = Restore
home-discard = Discard
app-tagline = Livery editor for Euro Truck Simulator 2 and American Truck Simulator
home-new-project = New Project
home-new-project-tip = Create a new livery project ({ $shortcut })
home-open-tip = Open a project file ({ $shortcut })
home-preferences = Preferences
home-recent = Recent projects
home-recent-empty = No recent projects
home-recent-empty-hint = Projects you open will appear here. Start by creating a new project.
home-recent-empty-create = Create Your First Project
home-recent-missing-item = { $name } (file not found)
home-remove-recent = Remove from list
home-file-not-found = File not found
home-open-recent = Open { $path }
home-file-moved = This project's file was moved or deleted.

## Dialogs

unsaved-title = Save changes to “{ $name }” before closing?
unsaved-hint = Your changes will be lost if you don't save them.
button-dont-save = Don't Save
button-cancel = Cancel
button-ok = OK
button-done = Done
button-close = Close
new-project-name = Project name
prefs-interface = Interface
prefs-ui-scale = UI scale
prefs-text-size = Text size
prefs-scale-hint = Changes apply immediately. 100% follows your display's scale factor.
prefs-canvas = Canvas
prefs-grid-spacing = Grid spacing
prefs-reset = Reset to defaults
shortcuts-temporary-hand = Temporary Hand tool
shortcuts-hold-space = Hold Space
about-version = Version { $version }
about-tagline = Livery editor for Euro Truck Simulator 2 and American Truck Simulator.

## Export Texture dialog

export-done = Exported { $file }
export-failed-title = Could not export
export-failed = { $file } could not be written ({ $reason }).
export-preview = Export preview
export-info = { $width } × { $height } px · { $format } · { $bytes }
export-progress = Exporting { $file }…
export-start = Export…
export-format = Format
export-dds-bc3 = BC3 / DXT5 (recommended)
export-dds-rgba = Uncompressed RGBA
export-dds-encoding = DDS encoding
export-size = Size
export-background = Background
export-transparent = Transparent
export-background-color = Background color
prefs-language = Language
prefs-system-title-bar = Use the system title bar
prefs-system-title-bar-hint = The window's frame and title bar come from the system, with the menus in a row below. Useful when the window can't be moved or resized.
file-dev-format = { $file } uses a development format that this version cannot open.

## Vehicle packages

pkg-install-failed = { $file } could not be installed: { $reason }
pkg-no-library = there is no vehicle library folder.
pkg-io = it could not be read or written ({ $reason }).
pkg-not-a-zip = it is not a vehicle package.
pkg-no-manifest = it has no vehicle.json manifest.
pkg-bad-manifest = its manifest is invalid ({ $reason }).
pkg-newer-format = it was made for a newer version of TruckPaint.
pkg-bad-id = its id “{ $id }” is invalid.
pkg-no-main-texture = it has no main texture.
pkg-bad-changelog = its changelog is longer than 1000 characters.
pkg-bad-game-path = its game path “{ $path }” is not valid (words of a–z, 0–9 and _ separated by dots).
pkg-bad-game-id = the game id “{ $id }” is not valid (words of a–z, 0–9 and _ separated by dots).
pkg-missing-game-ids = “{ $texture }” has no game id.
pkg-duplicate-part = two textures have the id “{ $id }”.
pkg-duplicate-game-id = the game id “{ $id }” is listed twice.
pkg-bad-also-paints = the accessory “{ $texture }” lists accessories it also paints: only a main texture can.
pkg-bad-size = the texture “{ $texture }” has an invalid size ({ $size }): it must be a width and a height, [width, height], each a power of two from 64 to 8192.
pkg-unsafe-path = it contains an unsafe path ({ $path }).
pkg-too-large = it is too large.
pkg-missing-template = the template of “{ $texture }” is missing.
pkg-bad-template = the template of “{ $texture }” is not a readable PNG or SVG image.
pkg-template-too-large = the template of “{ $texture }” is too large.

## Custom vehicles

custom-build-failed = { $name } could not be built: { $reason }
custom-template-unconvertible = the template of “{ $texture }” can't be converted.
custom-file-not-image = { $file } can't be used: it is not a PNG, DDS or SVG image.
custom-file-unsupported-dds = { $file } can't be read: its DDS compression ({ $format }) is not supported. Save it as BC1, BC2, BC3 or uncompressed RGB(A).
custom-file-damaged = { $file } can't be read: the file is damaged or incomplete.
custom-file-too-large = { $file } can't be used: it is larger than { $size } px.
custom-file-io = { $file } can't be read ({ $reason }).
custom-build-stopped = { $name } could not be built: the build stopped unexpectedly.
custom-title = Custom Vehicle
custom-new-version-title = New Version of { $name }
custom-open = Custom Vehicle…
custom-open-library = Custom Vehicle…
custom-version = Version
custom-what-changed = What changed
custom-what-changed-hint = Door layout redrawn after the game update…
custom-what-changed-count = { $count } / { $max } characters
custom-problem-changelog = “What changed” is longer than { $max } characters.
custom-name = Name
custom-brand = Brand
custom-kind = Kind
custom-game = Game
custom-game-path = Game path
custom-game-path-hint = As in the game's definitions, e.g. scania.r_2016
custom-game-versions = Game versions
custom-game-versions-hint = Any version, or a range such as >=1.53
custom-alt-uv = Alternate UV set
custom-colour-picker = Color picker
custom-textures = Textures
custom-drop-hint = Drop the game's template files here (PNG, DDS or SVG), or add them. Each file is one texture.
custom-add-templates = Add Templates…
custom-row-name = Name of texture { $n }
custom-row-role = Role of texture { $n }
custom-row-width = Width of texture { $n }
custom-row-height = Height of texture { $n }
custom-row-role-short = Role
custom-row-width-short = Width
custom-row-height-short = Height
custom-row-game-ids = Game ids of texture { $n }
custom-row-replace = Replace…
custom-row-replace-name = Replace the template of texture { $n }
custom-row-remove = Remove texture { $n }
custom-role-main = Main texture
custom-role-accessory = Accessory
custom-hint-cabins = Internal names of the cabins that use this layout, e.g. highline, highline_8x4
custom-hint-cabins-optional = Internal names of the cabins that use this layout; leave empty to paint every cabin
custom-hint-accessories = Accessory ids this texture covers, e.g. mirror.painted, s_mirror.painted
custom-file = { $file } · { $width } × { $height } px
custom-template-scaled = The template will be scaled to { $size }.
custom-scs-reminder = Templates from the base games belong to SCS Software: use them for your own liveries, and check their license before sharing a package.
custom-create = Create
custom-building = Building the package… ({ $done }/{ $total })
custom-building-plain = Building the package…
custom-problem-name = Enter the vehicle's name.
custom-problem-brand = Enter the vehicle's brand.
custom-problem-game-path = The game path is words of a–z, 0–9 and _ separated by dots, as in the game's definitions, e.g. scania.r_2016.
custom-problem-game-versions = Enter a version range such as >=1.53, or leave empty for any version.
custom-problem-version = Enter a version such as 1.1.0.
custom-problem-version-not-higher = The version must be higher than { $version }.
custom-problem-installed = { $id } is already installed. Make a new version of it with New Version… in the Vehicle Library, or change the name.
custom-problem-no-main = Make one of the textures a main texture.
custom-problem-trailer-main = A trailer has one main texture: make the others accessories.
custom-problem-texture-name = Enter the texture's name.
custom-problem-cabins = Enter the internal names of the cabins that use this layout.
custom-problem-accessory-ids = Enter the accessory ids this texture covers.
custom-problem-game-id = “{ $id }” is not a valid game id: use words of a–z, 0–9 and _ separated by dots.
custom-problem-duplicate-game-id = “{ $id }” is already used by another texture.

## Vehicle library and updates

filter-packages = Vehicle packages
filter-templates = Templates (PNG, DDS, SVG)
dialog-export-package = Export Vehicle Package (templates from the base games belong to SCS Software)
vehicles-search = Search vehicles
vehicles-all-games = All games
vehicles-game-filter = Game
vehicles-all-kinds = Trucks and trailers
vehicles-kind-filter = Kind
vehicles-truck = Truck
vehicles-trailer = Trailer
vehicles-installed = Installed { $name } { $version }
vehicles-empty = No vehicle installed
vehicles-empty-hint = Vehicle packages (.tpv) hold the templates of a truck or trailer. Install one to start a livery on it.
vehicles-details-any-version = Any game version
vehicles-details-versions = Game versions: { $versions }
vehicles-details-main = { $count ->
    [one] Main texture
   *[other] Main textures
}: { $names }
# A no-break space keeps the count with its noun.
vehicles-details-accessories = { $count ->
    [0] No accessories
    [one] { $count } accessory
   *[other] { $count } accessories
}
vehicles-version = Version { $version }
vehicles-remove-version = Remove { $name } { $version }
vehicles-remove-confirm = Remove { $name } from the library? Projects keep their templates.
vehicles-remove = Remove
vehicles-install = Install…
vehicles-install-sample = Install the sample vehicles

# Game version ranges in words (">=1.56" reads "1.56 or later").
version-range-any = any version
version-range-from = { $version } or later
version-range-until = { $version } or earlier
version-range-exact = { $version }
version-range-between = { $from } to before { $to }

vehicles-new-version = New Version…
vehicles-new-version-of = New version of { $name }
vehicles-export-version = Export { $name } { $version }
vehicles-tab-installed = Installed · { $count }
vehicles-tab-mine = My vehicles · { $count }
vehicles-status-custom = Custom
vehicles-status-update = Update
vehicles-status-current = Up to date
vehicles-mine-empty = Custom vehicles are made from the game's template files (DDS, PNG or SVG) and their game data. Click Custom Vehicle… to make one.
vehicles-preview = Template of { $name }
vehicles-details-of = Details of { $name }
vehicles-installed-versions = Installed versions
vehicles-update-summary = Update summary: { $version }
vehicles-changes-layout = Layout changed: { $names }
vehicles-changes-resized = Resized: { $names }
vehicles-changes-added = New: { $names }
vehicles-changes-removed = Removed: { $names }
vehicles-no-changes = No changes listed
vehicles-projects = Projects using an older version
vehicles-projects-reading = Reading recent projects…
vehicles-recent-only = Only recent projects are listed
vehicles-later = Later
vehicles-update-vehicle = Update { $name }
vehicles-update-hint = Open a project that uses this vehicle to update it.
vehicles-impact-textures = { $count ->
    [one] 1 texture
   *[other] { $count } textures
}
vehicles-impact-resized = { $count ->
    [one] 1 texture resized
   *[other] { $count } textures resized
}
vehicles-impact-removed = { $count } no longer in this version
vehicles-impact-nothing = Nothing to reposition
vehicles-impact-line = { $name }: { $impact }
vehicles-impact-unreadable = { $name }: Can’t be read
vehicles-export = Export…
vehicles-exported = Exported { $file }
vehicles-export-failed = { $file } could not be written: { $reason }
update-versions = { $name }: version { $from } → { $to }
update-replaced = { $name }: template replaced
update-layout-changed = { $name }: layout changed, { $objects }
update-objects = { $count ->
    [0] nothing to reposition
    [one] 1 object to reposition
   *[other] { $count } objects to reposition
}
update-changelog = What changed in { $version }:
update-resized = { $name }: size { $old } → { $new } px, artwork scaled
update-new-textures = New in this version:
update-removed = { $name }: no longer in this version, artwork kept
update-artwork-kept = Your artwork is kept. You can undo the update.
update-apply = Update

## New Project

new-project-no-vehicles-hint = Vehicle packages hold the templates of a truck or trailer. Try the sample vehicles, install a package, or make a custom vehicle.
new-project-create = Create Project
new-project-add-later = You can add vehicles to the project at any time.
new-project-no-game-vehicles = No { $game } vehicle is installed yet.
new-project-no-match = No vehicle matches the search.
new-project-custom-title = Vehicle without a package
new-project-custom-hint = Make it from its template files (DDS, PNG or SVG) and its game data.
new-project-fleet = Your fleet
new-project-check-vehicles = Check vehicles in the list.
new-project-cabins = Cabins
new-project-column-vehicle = Vehicle
new-project-column-type = Type
new-project-column-package = Package
new-project-counts = { $accessories ->
    [0] { $main } main
    [one] { $main } main + { $accessories } accessory
   *[other] { $main } main + { $accessories } accessories
}
new-project-created = Textures created
new-project-texture = { $texture } ({ $size })
new-project-total = { $count ->
    [one] { $count } texture in total
   *[other] { $count } textures in total
}
new-project-vehicle-count = { $count ->
    [one] { $count } vehicle
   *[other] { $count } vehicles
}
new-project-remove = Remove
vehicles-kind-all = All
vehicles-kind-trucks = Trucks
vehicles-kind-trailers = Trailers

## Fleet

add-vehicle-none = No other vehicle of this game is installed. Install a package to add one.
add-vehicle-add = Add
fleet-error-other-game = This vehicle is for another game than the project.
fleet-error-already-there = This vehicle is already in the project.
fleet-error-textures = Choose at least one main texture of this vehicle.
fleet-error-version = The installed version differs from the one the project uses.
fleet-error-unknown = This vehicle is not in the project.
textures-title = Textures of { $name }
textures-missing-version = Version { $version } of this vehicle is not installed. Update its template to change its textures.
textures-remove-confirm = The artwork of { $textures } will be removed with them.
textures-remove = Remove
textures-apply = Apply
copy-cabin-hint = Copies every object of the chosen cabin onto { $texture }, at the same positions.
copy-cabin-objects = { $count ->
    [one] { $count } object
   *[other] { $count } objects
}
copy-cabin-empty = This texture holds no object to copy.
copy-cabin-copy = Copy
remove-vehicle-confirm = Remove { $name } and its artwork from the project?

# Export Mod
dialog-export-mod = Export Mod
filter-mods = Game mods
filter-mod-images = Images (PNG, JPEG)
mod-name = Name
mod-version = Version
mod-author = Author
mod-description = Description
mod-price = Price
mod-unlock = Unlock level
mod-advanced = Advanced
mod-internal-name = Internal name
mod-internal-name-help = Names the paint job in the game: a–z, 0–9 and _. Keep it unique to you.
mod-icon = Shop icon
mod-image = Mod Manager image
mod-choose-icon = Choose Icon…
mod-generated-icon = Use Generated Icon
mod-choose-image = Choose Image…
mod-generated-image = Use Generated Image
mod-picture-failed = { $file } can't be used ({ $reason }).
mod-summary = In the mod
mod-summary-cabins = { $texture } ({ $count ->
    [one] cabin
   *[other] cabins
}: { $cabins })
mod-summary-every-cabin = { $texture } (every cabin)
mod-summary-not-painted = { $texture } (not painted)
mod-summary-accessories = Accessories: { $accessories }
mod-problem-name-empty = The mod needs a name.
mod-problem-name-invalid = The name can't contain ", \ or a line break.
mod-problem-version-invalid = The version can't contain ", \ or a line break.
mod-problem-author-invalid = The author can't contain ", \ or a line break.
mod-problem-internal-empty = The internal name can't be empty.
mod-problem-internal-invalid = The internal name can only use a–z, 0–9 and _.
mod-problem-internal-long = The internal name can have at most { $max } characters.
mod-problem-price = The price must be more than 0.
mod-problem-same-path = { $first } and { $second } are the same vehicle in the game ({ $path }): remove one of them.
mod-problem-game-data = { $vehicle } { $version } must be installed to export the mod.
mod-problem-bad-game-version = { $version } isn't a game version: write it like 1.56.* or 1.56.2.
mod-problem-unsupported-game-version = { $vehicle } ({ $range }) doesn't support game version { $version }.
mod-problem-no-common-game-version = { $first } ({ $first_range }) and { $second } ({ $second_range }) have no game version in common: update or remove one of them.
mod-warning-layout-changed = { $texture }: layout changed
mod-warning-not-in-version = { $texture }: not in this version, left out of the mod
mod-warning-empty-one = { $texture } is empty, exported with the game's color
mod-warning-empty-many = { $count } textures empty, exported with the game's color
mod-export-done = Mod exported to { $file }
mod-export-start = Export
mod-destination = Destination
mod-change-destination = Change…
mod-export-replace-question = { $file } already exists. Replace it?
mod-export-replace = Replace

## Import from Library

library-hint = Your symbols, swatches and styles, shared by every project of both games. Imported elements are copies, with everything they use.
library-symbols = Symbols
library-swatches = Swatches
library-graphic-styles = Graphic styles
library-text-styles = Text styles
library-in-project = In this project
library-import = Import
library-empty = Your library is empty. Right-click a symbol, a swatch or a style in the Brand space or the Resources tab and choose Add to Library to use it in every project.
library-remove = Remove from Library
library-remove-confirm = Remove { $name } from the library? Projects that imported it keep their copy.
library-remove-button = Remove
library-unreadable-title = Library
library-unreadable = The library could not be read: it starts empty. A copy of the file was kept as { $file }.
library-unreadable-no-backup = The library could not be read: it starts empty.

prefs-mcp = AI assistant (MCP)
prefs-mcp-enable = Let an AI assistant draw on the open project
prefs-mcp-hint = An MCP client, such as Claude Desktop or Claude Code, can connect to this address to read the open project and draw on its textures. Every change it makes can be undone, and it can't save, export or open files.
prefs-mcp-port = Port
prefs-mcp-url = Address
prefs-mcp-copy = Copy
prefs-mcp-copied = Copied
prefs-mcp-off = Off
prefs-mcp-on = On, waiting for a client
prefs-mcp-connected = On, a client is connected
prefs-mcp-port-in-use = Port { $port } is in use by another program. Turn the server off, choose another port, and turn it on again.
prefs-mcp-failed = The server could not start: { $error }

# My Game tab of the Vehicle Library, and the Games section of Preferences.
vehicles-tab-game = My Game
game-header = { $game } { $version }
game-refresh = Refresh
game-preferences = Preferences…
game-select-all = Select All
game-select-none = Select None
game-select-updates = Select Updates
game-generate = Generate ({ $count })
game-progress = { $done } of { $total }
game-making = Making { $name }…
game-stopping = Stopping after { $name }…
game-status-not-generated = Not generated
game-status-generated = Generated { $version }
game-status-updated = Game updated
game-row-waiting = Waiting
game-row-working = In progress
game-row-installed = Installed { $version }
game-row-up-to-date = Up to date
game-row-failed = Failed
game-scanning = Reading the game…
game-scan-failed = The game cannot be read: { $reason }
game-folder-problem = The game folder cannot be used
game-no-folder = No game folder set
game-no-folder-hint = Set the folder where { $game } is installed in Preferences. Official trucks and trailers are made from it, on this computer.
game-detected = Found in your Steam libraries: { $folder }
game-use-detected = Use This Folder
game-ats-title = American Truck Simulator is not supported yet
game-ats-unsupported = Generating American Truck Simulator vehicles is not supported yet. Its folder is set and checked, so it is ready when support comes.
game-empty = No vehicle found
game-empty-hint = This game folder holds no truck or trailer that can be painted.
game-result = Installed: { $installed } · Up to date: { $skipped } · Failed: { $failed }
game-result-cancelled = Cancelled. Installed: { $installed } · Up to date: { $skipped } · Failed: { $failed }
game-failed-line = { $name }: { $reason }
game-finished-title = My Game
game-notice = The game's content belongs to SCS Software. These packages are made on this computer for your own use and are never shared. TruckPaint is not endorsed by SCS Software. A game patch that keeps the same version series is not flagged: check a vehicle and generate it again to refresh it.
prefs-games = Games
prefs-games-hint = TruckPaint reads these folders to make the official trucks and trailers, for your own use. Nothing is sent anywhere.
prefs-game-folder-hint = Folder where the game is installed
prefs-game-detect = Detect
prefs-game-detect-hint = Look for the game in your Steam libraries
prefs-game-not-detected = The game was not found in your Steam libraries. Choose its folder instead.
prefs-game-choose = Choose…
prefs-game-choose-title = Choose the { $game } Folder
prefs-game-clear = Clear
prefs-game-version = Version { $version }
prefs-play = I play this game
prefs-play-hint = Keep at least one game checked.
prefs-mod-folder = { $game } mod folder
prefs-mod-none = Not found
prefs-mod-choose-title = Choose the { $game } Mod Folder
games-not-found = This folder cannot be found.
games-missing-file = { $file } is missing in this folder.
games-not-a-game = This folder does not hold Euro Truck Simulator 2 or American Truck Simulator.
games-unreadable = The game files cannot be read ({ $cause }).
games-is-ets2 = This is a Euro Truck Simulator 2 folder.
games-is-ats = This is an American Truck Simulator folder.
