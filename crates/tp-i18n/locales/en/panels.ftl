
## Panel titles and empty states

panel-styles = Styles
panel-symbols = Symbols
panel-stroke = Stroke
panel-vehicle = Vehicles
empty-layers = No layers yet
empty-assets-hint = Imported logos and images will be listed here.

## Layers

layers-empty-hint = Shapes you draw and layers you add appear here.
layers-collapse = Collapse { $name }
layers-expand = Expand { $name }
layers-hide = Hide { $name }
layers-name-to-reposition = { $name }, to reposition
layers-to-reposition-tooltip = The texture’s layout changed in this version: check this object’s position.
layers-show = Show { $name }
layers-lock = Lock { $name }
layers-unlock = Unlock { $name }
layers-name = Layer name
layers-heading = { $texture } · { $count ->
    [one] { $count } layer
   *[other] { $count } layers
}

## Assets

assets-uses = { $count ->
    [one] { $count } use
   *[other] { $count } uses
}
assets-used-by = { $count ->
    [one] Used by { $count } object
   *[other] Used by { $count } objects
}
assets-vector = Vector
assets-item = Asset { $name }
assets-place = Place Asset
assets-drop-zone = Drop a logo here or
assets-import = Import…
resources-images = Images
resources-footer = Drag an element onto the canvas to place it. Open the Brand space to manage everything.
assets-name = Asset name

## Stroke panel

stroke-enabled = Stroke enabled
stroke-width = Stroke width
stroke-center = Center
stroke-center-name = Center stroke
stroke-inside = Inside
stroke-inside-name = Inside stroke
stroke-outside = Outside
stroke-outside-name = Outside stroke
undo-change-stroke-width = Change Stroke Width
undo-change-stroke-alignment = Change Stroke Alignment
undo-add-stroke = Add Stroke
undo-remove-stroke = Remove Stroke

## Transform panel

field-width = Width
field-height = Height
field-w = W
field-h = H
field-r = R
field-s = S
transform-x = X position
transform-y = Y position
transform-rotation = Rotation
transform-scale = Scale
transform-unlock-proportions = Unlock proportions
transform-lock-proportions = Lock proportions
transform-align-to = Align to

## Properties panel

undo-change-opacity = Change Opacity
undo-change-corner-radius = Change Corner Radius
undo-change-sides = Change Sides
undo-change-inner-radius = Change Inner Radius
undo-change-line-width = Change Line Width
undo-change-star = Change Star
props-sides = Sides
props-star = Star
props-inner = Inner
props-inner-radius = Inner radius
props-objects = { $count } objects
props-source-svg = SVG · Vector
props-source-image = Image · { $width } × { $height } px
props-opacity = Opacity
props-opacity-slider = Opacity slider
props-radius = Radius
props-corner-radius = Corner radius
props-line-width = Line width
props-fill = Fill
props-fill-color = Fill color
props-stroke-color = Stroke color

## Dashes, caps and joins

line-preset-dashed = Dashed 20/10
line-preset-dotted = Dotted 0/12
line-preset-long-dash = Long dash 60/20
undo-change-stroke-caps = Change Stroke Caps
undo-change-stroke-joins = Change Stroke Joins
undo-change-stroke-dashes = Change Stroke Dashes
undo-change-line-caps = Change Line Caps
undo-change-line-joins = Change Line Joins
undo-change-line-dashes = Change Line Dashes
line-dashed = Dashed
line-presets = Presets
line-dash-presets = Dash presets
line-dash = Dash
line-dash-length = Dash length
line-gap = Gap
line-gap-length = Gap length
line-cap = Cap
line-butt = Butt
line-butt-cap = Butt cap
line-round = Round
line-round-cap = Round cap
line-square = Square
line-square-cap = Square cap
line-join = Join
line-miter = Miter
line-miter-join = Miter join
line-round-join = Round join
line-bevel = Bevel
line-bevel-join = Bevel join
line-limit = Limit
line-miter-limit = Miter limit

## Colors panel

colors-no-stroke = No stroke
colors-solid = Solid
colors-solid-paint = Solid paint
colors-linear = Linear
colors-linear-gradient = Linear gradient
colors-radial = Radial
colors-radial-gradient = Radial gradient
colors-location = Location
colors-stop-location = Stop location
colors-angle = Angle
colors-gradient-angle = Gradient angle
colors-reverse = Reverse gradient
colors-red = Red
colors-green = Green
colors-blue = Blue
colors-hue = Hue
colors-saturation = Saturation
colors-value = Value
colors-lightness = Lightness
colors-alpha = Alpha
colors-hex = Hex
colors-hex-color = Hex color
colors-hex-invalid = Invalid hex color. Use #RGB, #RRGGBB or #RRGGBBAA.
colors-recent = Recent
colors-recent-item = Recent color { $color }
colors-palette = Palette
colors-add-to-palette = Add to Palette
colors-differ = Colors differ in the selection.
colors-palette-empty = Save colors you reuse with +.
colors-swatch-prefix = Color
colors-edit-swatch = Edit Swatch…
colors-delete-swatch = Delete Swatch
colors-linked-to = Linked to { $name }
colors-swatch-name = Name
colors-swatch-name-empty = Enter a name for the swatch.
colors-swatch-hex = Swatch hex color
colors-aspect = Aspect
colors-aspect-ratio = Gradient aspect ratio

## Character panel

weight-thin = Thin
weight-extra-light = Extra Light
weight-light = Light
weight-regular = Regular
weight-medium = Medium
weight-semi-bold = Semi Bold
weight-bold = Bold
weight-extra-bold = Extra Bold
weight-black = Black
char-font-weight = Font weight
char-italic = Italic
char-size = Size
char-font-size = Font size
char-align-left = Align left
char-align-center = Align center
char-align-right = Align right
char-tracking = Tracking
char-letter-spacing = Letter spacing
char-line = Line
char-line-height = Line height
char-family-missing-name = Font family: { $family } (Font not found: { $family })
char-family-name = Font family: { $family }
char-font-not-found = Font not found: { $family }
char-font-family = Font family
char-search-fonts = Search fonts
char-no-fonts = No matching fonts
undo-change-font-weight = Change Font Weight
undo-change-italic = Change Italic
undo-change-text-size = Change Text Size
undo-change-alignment = Change Alignment
undo-change-letter-spacing = Change Letter Spacing
undo-change-line-height = Change Line Height
undo-change-font = Change Font

## Vehicle panel

vehicle-panel-package = Package { $version } · game versions { $games }
vehicle-panel-package-missing = Package { $version } (not installed)
vehicle-panel-update = Version { $version } is available
vehicle-panel-texture = Texture { $name }, { $state }
texture-state-empty = Empty
texture-state-modified = Modified
texture-state-to-check = To check
texture-reason-layout-changed = Layout changed in { $version }
texture-reason-not-in-version = Not in this version
texture-mark-checked = Mark as Checked
texture-mark-checked-named = Mark { $name } as checked
vehicle-panel-opacity-name = Template opacity

## Fleet

vehicle-panel-update-named = Update the template of { $name }
vehicles-textures = Textures…
vehicles-main-textures = Main textures
vehicles-accessories = Accessories
vehicles-remove-from-project = Remove from Project

## Project

vehicle-panel-game = Vehicles · { $game }
project-game-versions = Game versions
project-game-versions-supported = Supported by every vehicle: { $versions }
project-no-common-version = No game version is supported by every vehicle
vehicle-actions-named = Actions for { $name }
texture-notice-not-in-version = Not in this version, left out of the mod

## Styles panel

styles-graphic = Graphic styles
styles-text = Text styles
styles-graphic-empty = Save a fill, stroke and opacity to reuse on every texture: select a shape and click +.
styles-text-empty = Save lettering settings to reuse on every texture: select a text and click +.
styles-new-graphic = New Graphic Style from Selection
styles-new-text = New Text Style from Selection
styles-new = New Style
styles-need-shape = Select one shape or text.
styles-need-text = Select one text.
styles-graphic-item = Graphic style { $name }
styles-text-item = Text style { $name }
styles-graphic-prefix = Style
styles-text-prefix = Text style
styles-apply = Apply Style
styles-rename = Rename
styles-redefine = Redefine from Selection
styles-select-users = Select Users on This Texture
styles-delete = Delete Style
styles-name = Style name

## Symbols panel

symbols-prefix = Symbol
symbols-copy-suffix = copy
symbols-empty = Draw a logo or a lettering once, select it and choose Convert to Symbol: place it on every texture, and edit it once for all.
symbols-instances = { $count ->
    [one] { $count } instance
   *[other] { $count } instances
}
symbols-item = Symbol { $name }
symbols-place = Place { $name }
symbols-edit = Edit { $name }
symbols-rename = Rename
symbols-duplicate = Duplicate
symbols-delete = Delete Symbol
symbols-delete-confirm = Delete { $name }? { $count ->
    [one] Its instance becomes a group that looks the same.
   *[other] Its { $count } instances become groups that look the same.
}
symbols-name = Symbol name
symbol-bar-editing = Editing symbol “{ $name }”
symbol-bar-done = Done
props-instance-of = Instance of { $name }
instance-look-in-symbol = An instance shows its symbol: edit the symbol to change its look, or detach the instance.

## Library

library-add = Add to Library
library-update = Update in Library
library-added = Added to the library
library-updated = Library updated
undo-import-from-library = Import from Library

## Inspector

inspector-layout = Layout
inspector-text = Text
inspector-appearance = Appearance
inspector-polygon = Polygon
inspector-image = Image
inspector-new-objects = New objects
inspector-hint = Select an object to set its layout, fill and stroke.
inspector-main-texture = Main texture
inspector-accessory-texture = Accessory texture
inspector-symbol = Symbol
inspector-texture-kind = { $kind } · { $size }
inspector-on-texture = On this texture
inspector-in-symbol = In this symbol
inspector-objects = Objects
inspector-instances = Symbol instances
inspector-off-palette = Off-palette colors
inspector-off-palette-select = Select objects with off-palette colors
inspector-off-palette-locked = { $count ->
    [one] { $count } locked object is left out of the selection
   *[other] { $count } locked objects are left out of the selection
}
inspector-style = Style
inspector-style-none = None
stroke-none = None
stroke-width-px = { $width } px
stroke-options = Stroke options
line-settings = Line settings
colors-brand-palette = Brand palette
colors-add-short = + Add

## Project space

project-vehicles-count = { $vehicles ->
    [one] { $vehicles } vehicle
   *[other] { $vehicles } vehicles
} · { $textures ->
    [one] { $textures } texture
   *[other] { $textures } textures
}
project-kind-package = { $kind } · package { $version }
project-cabins = Cabins
project-main-texture = Main texture
project-mode-per-layout = One per cabin layout
project-mode-every-cabin = One for every cabin
project-mode-single = Single main texture
project-update-available = Update { $version } available
vehicle-textures-named = Textures of { $name }
project-texture-main = Main
project-texture-accessory = Accessory
project-mod-information = Mod information
project-picture-generated = Generated from the first main texture
project-cabin-ids = Internal names: { $ids }
project-texture-states = { $modified ->
    [0] {""}
   *[other] {" · "}{ $modified } modified
}{ $check ->
    [0] {""}
   *[other] {" · "}{ $check } to check
}
project-filter-all = All
project-filter-to-do = To do
project-filter-to-check = To check
project-nothing-to-do = Nothing to do
project-before-exporting = Before exporting
project-nothing-to-check = Nothing to check
project-open = Open
project-open-named = Open { $name }
project-show = Show
project-show-named = Show { $name }
project-picture-drop = or drop a PNG or JPEG here
project-game-versions-add = Add
project-game-version-remove = Remove { $version }

## Brand space

brand-new-color = New Color
brand-new-style = New Style from Selection
brand-create-from-selection = Create from Selection
brand-actions-named = Actions for { $name }
brand-usage = { $textures ->
    [one] { $textures } texture
   *[other] { $textures } textures
} · { $objects ->
    [one] { $objects } object
   *[other] { $objects } objects
}
brand-unused = Unused
brand-symbol-usage = { $instances ->
    [one] { $instances } instance
   *[other] { $instances } instances
} · { $textures ->
    [one] { $textures } texture
   *[other] { $textures } textures
}
brand-impact = Impact: { $textures ->
    [one] { $textures } texture
   *[other] { $textures } textures
}, { $objects ->
    [one] { $objects } object
   *[other] { $objects } objects
}
brand-impact-none = Not used yet
brand-edit-color = Edit color
brand-edit-style = Edit style
brand-before = Before
brand-after = After
brand-impact-note = Thumbnails update live. Nothing is saved before Apply to Fleet.
brand-impact-tile = Preview of { $name }
brand-apply-to-fleet = Apply to Fleet
brand-gradient-fill = Gradient
brand-gradient-fill-hint = This fill is a gradient: change it with Redefine from Selection.
styles-edit = Edit Style…
styles-name-empty = Enter a name for the style.
styles-name-taken = Another graphic style already has this name.

## Drop shadow
inspector-add-shadow = + Add a shadow
inspector-shadow = Shadow
inspector-shadow-summary = x { $x } · y { $y }
inspector-shadow-summary-blur = · blur { $blur }
inspector-shadow-summary-full = Offset { $x } / { $y }, blur { $blur }
inspector-shadow-remove = Remove shadow
shadow-opacity = Shadow opacity
shadow-offset-x = Offset X
shadow-offset-y = Offset Y
shadow-blur = Blur
undo-add-shadow = Add Shadow
undo-change-shadow = Change Shadow
undo-remove-shadow = Remove Shadow
