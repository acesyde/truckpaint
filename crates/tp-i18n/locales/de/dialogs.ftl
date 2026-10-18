## Exportgrößen und Fehler

size-mb = { $value } MB
size-kb = { $value } KB
size-about = etwa { $value }
export-stopped = der Export wurde unerwartet abgebrochen

## Projekte speichern und öffnen

file-save-failed-title = Speichern nicht möglich
file-save-failed = { $file } konnte nicht gespeichert werden.
file-save-failed-reason = { $file } konnte nicht gespeichert werden ({ $reason }). Ihre Änderungen sind weiterhin geöffnet.
file-open-failed-title = Projekt kann nicht geöffnet werden
file-restore-failed-title = Projekt kann nicht wiederhergestellt werden
file-recovered-copy = Die wiederhergestellte Kopie
file-not-a-project = { $file } ist kein TruckPaint-Projekt.
file-damaged = { $file } ist beschädigt und kann nicht geöffnet werden.
file-newer-version = { $file } wurde mit einer neueren Version von TruckPaint erstellt.
file-io = { $file } konnte nicht gelesen oder geschrieben werden ({ $reason }).

## Dateidialoge

dialog-open-project = Projekt öffnen
dialog-save-project = Projekt speichern
dialog-export-texture = Textur exportieren
filter-project = TruckPaint-Projekt
filter-images = Bilder

## Startbildschirm

home-recovered = Wiederhergestellte Projekte
home-recovered-hint = TruckPaint wurde unerwartet beendet. Diese Projekte enthielten nicht gespeicherte Änderungen.
home-never-saved = Nie gespeichert
home-recovery-damaged = Beschädigte Wiederherstellungskopie
home-recovery-damaged-hint = Diese Kopie kann nicht wiederhergestellt werden.
home-recovered-item = { $name } wiederhergestellt
home-restore = Wiederherstellen
home-discard = Verwerfen
app-tagline = Lackierungseditor für Euro Truck Simulator 2 und American Truck Simulator
home-new-project = Neues Projekt
home-new-project-tip = Neues Lackierungsprojekt erstellen ({ $shortcut })
home-open-tip = Eine Projektdatei öffnen ({ $shortcut })
home-preferences = Einstellungen
home-recent = Zuletzt verwendete Projekte
home-recent-empty = Keine zuletzt verwendeten Projekte
home-recent-empty-hint = Geöffnete Projekte werden hier angezeigt. Erstellen Sie zunächst ein neues Projekt.
home-recent-empty-create = Erstes Projekt erstellen
home-recent-missing-item = { $name } (Datei nicht gefunden)
home-remove-recent = Aus Liste entfernen
home-file-not-found = Datei nicht gefunden
home-open-recent = { $path } öffnen
home-file-moved = Die Datei dieses Projekts wurde verschoben oder gelöscht.

## Dialoge

unsaved-title = Änderungen an „{ $name }“ vor dem Schließen speichern?
unsaved-hint = Ihre Änderungen gehen verloren, wenn Sie sie nicht speichern.
button-dont-save = Nicht speichern
button-cancel = Abbrechen
button-ok = OK
button-done = Fertig
button-close = Schließen
new-project-name = Projektname
prefs-interface = Oberfläche
prefs-ui-scale = Skalierung der Oberfläche
prefs-text-size = Textgröße
prefs-scale-hint = Änderungen werden sofort übernommen. 100 % entspricht dem Skalierungsfaktor Ihres Bildschirms.
prefs-canvas = Arbeitsfläche
prefs-grid-spacing = Rasterabstand
prefs-reset = Auf Standard zurücksetzen
prefs-tab-general = Allgemein
prefs-tab-games = Spiele
prefs-tab-ai = KI-Assistent
shortcuts-temporary-hand = Vorübergehend Hand-Werkzeug
shortcuts-hold-space = Leertaste gedrückt halten
about-version = Version { $version }
about-tagline = Lackierungseditor für Euro Truck Simulator 2 und American Truck Simulator.

## Dialog „Textur exportieren“

export-done = { $file } exportiert
export-failed-title = Export nicht möglich
export-failed = { $file } konnte nicht geschrieben werden ({ $reason }).
export-preview = Exportvorschau
export-info = { $width } × { $height } px · { $format } · { $bytes }
export-progress = { $file } wird exportiert…
export-start = Exportieren…
export-format = Format
export-dds-bc3 = BC3 / DXT5 (empfohlen)
export-dds-rgba = Unkomprimiertes RGBA
export-dds-encoding = DDS-Kodierung
export-size = Größe
export-background = Hintergrund
export-transparent = Transparent
export-background-color = Hintergrundfarbe
prefs-language = Sprache
prefs-system-title-bar = Titelleiste des Systems verwenden
prefs-system-title-bar-hint = Rahmen und Titelleiste des Fensters kommen vom System, die Menüs stehen in einer Zeile darunter. Hilfreich, wenn sich das Fenster nicht verschieben oder in der Größe ändern lässt.
file-dev-format = { $file } verwendet ein Entwicklungsformat, das diese Version nicht öffnen kann.

## Vehicle packages

pkg-install-failed = { $file } konnte nicht installiert werden: { $reason }
pkg-no-library = es gibt keinen Ordner für die Fahrzeugbibliothek.
pkg-io = sie konnte nicht gelesen oder geschrieben werden ({ $reason }).
pkg-not-a-zip = es ist kein Fahrzeugpaket.
pkg-no-manifest = es enthält kein vehicle.json-Manifest.
pkg-bad-manifest = sein Manifest ist ungültig ({ $reason }).
pkg-newer-format = es wurde für eine neuere TruckPaint-Version erstellt.
pkg-bad-id = seine ID „{ $id }“ ist ungültig.
pkg-no-main-texture = es hat keine Haupttextur.
pkg-bad-changelog = sein Änderungsprotokoll ist länger als 1000 Zeichen.
pkg-bad-game-path = sein Spielpfad „{ $path }“ ist ungültig (Wörter aus a–z, 0–9 und _, durch Punkte getrennt).
pkg-bad-game-id = die Spiel-ID „{ $id }“ ist ungültig (Wörter aus a–z, 0–9 und _, durch Punkte getrennt).
pkg-missing-game-ids = „{ $texture }“ hat keine Spiel-ID.
pkg-duplicate-part = zwei Texturen haben die ID „{ $id }“.
pkg-duplicate-game-id = die Spiel-ID „{ $id }“ ist doppelt aufgeführt.
pkg-bad-also-paints = das Zubehör „{ $texture }“ führt Zubehörteile auf, die es mitlackiert: das kann nur eine Haupttextur.
pkg-bad-size = die Textur „{ $texture }“ hat eine ungültige Größe ({ $size }): sie muss eine Breite und eine Höhe sein, [Breite, Höhe], jeweils eine Zweierpotenz von 64 bis 8192.
pkg-unsafe-path = es enthält einen unsicheren Pfad ({ $path }).
pkg-too-large = es ist zu groß.
pkg-missing-template = die Vorlage von „{ $texture }“ fehlt.
pkg-bad-template = die Vorlage von „{ $texture }“ ist kein lesbares PNG- oder SVG-Bild.
pkg-template-too-large = die Vorlage von „{ $texture }“ ist zu groß.

## Custom vehicles

custom-build-failed = { $name } konnte nicht erstellt werden: { $reason }
custom-template-unconvertible = die Vorlage von „{ $texture }“ kann nicht konvertiert werden.
custom-file-not-image = { $file } kann nicht verwendet werden: Es ist kein PNG-, DDS- oder SVG-Bild.
custom-file-unsupported-dds = { $file } kann nicht gelesen werden: Seine DDS-Kompression ({ $format }) wird nicht unterstützt. Speichern Sie es als BC1, BC2, BC3 oder unkomprimiertes RGB(A).
custom-file-damaged = { $file } kann nicht gelesen werden: Die Datei ist beschädigt oder unvollständig.
custom-file-too-large = { $file } kann nicht verwendet werden: Es ist größer als { $size } px.
custom-file-io = { $file } kann nicht gelesen werden ({ $reason }).
custom-build-stopped = { $name } konnte nicht erstellt werden: Die Erstellung wurde unerwartet beendet.
custom-title = Eigenes Fahrzeug
custom-new-version-title = Neue Version von { $name }
custom-open = Eigenes Fahrzeug…
custom-open-library = Eigenes Fahrzeug…
custom-version = Version
custom-what-changed = Änderungen
custom-what-changed-hint = Türvorlage nach dem Spielupdate neu gezeichnet …
custom-what-changed-count = { $count } / { $max } Zeichen
custom-problem-changelog = „Änderungen“ ist länger als { $max } Zeichen.
custom-name = Name
custom-brand = Marke
custom-kind = Art
custom-game = Spiel
custom-game-path = Pfad im Spiel
custom-game-path-hint = Wie in den Definitionen des Spiels, z. B. scania.r_2016
custom-game-versions = Spielversionen
custom-game-versions-hint = Jede Version, oder ein Bereich wie >=1.53
custom-alt-uv = Alternativer UV-Satz
custom-colour-picker = Farbwahl
custom-textures = Texturen
custom-drop-hint = Ziehen Sie die Vorlagendateien des Spiels hierher (PNG, DDS oder SVG), oder fügen Sie sie hinzu. Jede Datei ist eine Textur.
custom-add-templates = Vorlagen hinzufügen…
custom-row-name = Name der Textur { $n }
custom-row-role = Rolle der Textur { $n }
custom-row-width = Breite der Textur { $n }
custom-row-height = Höhe der Textur { $n }
custom-row-role-short = Rolle
custom-row-width-short = Breite
custom-row-height-short = Höhe
custom-row-game-ids = Spielkennungen der Textur { $n }
custom-row-replace = Ersetzen…
custom-row-replace-name = Vorlage der Textur { $n } ersetzen
custom-row-remove = Textur { $n } entfernen
custom-role-main = Haupttextur
custom-role-accessory = Zubehör
custom-hint-cabins = Interne Namen der Kabinen mit diesem Layout, z. B. highline, highline_8x4
custom-hint-cabins-optional = Interne Namen der Kabinen mit diesem Layout; leer lassen, um alle Kabinen zu lackieren
custom-hint-accessories = Zubehörkennungen, die diese Textur abdeckt, z. B. mirror.painted, s_mirror.painted
custom-file = { $file } · { $width } × { $height } px
custom-template-scaled = Die Vorlage wird auf { $size } skaliert.
custom-scs-reminder = Vorlagen der Basisspiele gehören SCS Software: Verwenden Sie sie für Ihre eigenen Lackierungen und prüfen Sie ihre Lizenz, bevor Sie ein Paket weitergeben.
custom-create = Erstellen
custom-building = Paket wird erstellt… ({ $done }/{ $total })
custom-building-plain = Paket wird erstellt…
custom-problem-name = Geben Sie den Namen des Fahrzeugs ein.
custom-problem-brand = Geben Sie die Marke des Fahrzeugs ein.
custom-problem-game-path = Der Pfad im Spiel besteht aus Wörtern aus a–z, 0–9 und _, getrennt durch Punkte, wie in den Definitionen des Spiels, z. B. scania.r_2016.
custom-problem-game-versions = Geben Sie einen Versionsbereich wie >=1.53 ein, oder lassen Sie das Feld für jede Version leer.
custom-problem-version = Geben Sie eine Version wie 1.1.0 ein.
custom-problem-version-not-higher = Die Version muss höher als { $version } sein.
custom-problem-installed = { $id } ist bereits installiert. Erstellen Sie mit Neue Version… in der Fahrzeugbibliothek eine neue Version davon, oder ändern Sie den Namen.
custom-problem-no-main = Machen Sie eine der Texturen zur Haupttextur.
custom-problem-trailer-main = Ein Anhänger hat eine Haupttextur: Machen Sie die anderen zu Zubehör.
custom-problem-texture-name = Geben Sie den Namen der Textur ein.
custom-problem-cabins = Geben Sie die internen Namen der Kabinen mit diesem Layout ein.
custom-problem-accessory-ids = Geben Sie die Zubehörkennungen ein, die diese Textur abdeckt.
custom-problem-game-id = „{ $id }“ ist keine gültige Spielkennung: Verwenden Sie Wörter aus a–z, 0–9 und _, getrennt durch Punkte.
custom-problem-duplicate-game-id = „{ $id }“ wird bereits von einer anderen Textur verwendet.

## Vehicle library and updates

filter-packages = Fahrzeugpakete
filter-templates = Vorlagen (PNG, DDS, SVG)
dialog-export-package = Fahrzeugpaket exportieren (Vorlagen der Basisspiele gehören SCS Software)
vehicles-search = Fahrzeuge suchen
vehicles-all-games = Alle Spiele
vehicles-game-filter = Spiel
vehicles-all-kinds = Lkw und Anhänger
vehicles-kind-filter = Art
vehicles-truck = Lkw
vehicles-trailer = Anhänger
vehicles-installed = { $name } { $version } installiert
vehicles-empty = Kein Fahrzeug installiert
vehicles-empty-hint = Fahrzeugpakete (.tpv) enthalten die Vorlagen eines Lkw oder Anhängers. Installieren Sie eines, um eine Lackierung zu beginnen.
vehicles-details-any-version = Jede Spielversion
vehicles-details-versions = Spielversionen: { $versions }
vehicles-details-main = { $count ->
    [one] Haupttextur
   *[other] Haupttexturen
}: { $names }
vehicles-details-accessories = { $count ->
    [0] Kein Zubehör
    [one] { $count } Zubehörteil
   *[other] { $count } Zubehörteile
}
vehicles-version = Version { $version }
vehicles-remove-version = { $name } { $version } entfernen
vehicles-remove-confirm = { $name } aus der Bibliothek entfernen? Projekte behalten ihre Vorlagen.
vehicles-remove = Entfernen
vehicles-install = Installieren…
vehicles-install-sample = Beispielfahrzeuge installieren

# Spielversionsbereiche in Worten („>=1.56“ heißt „1.56 oder neuer“).
version-range-any = jede Version
version-range-from = { $version } oder neuer
version-range-until = { $version } oder älter
version-range-exact = { $version }
version-range-between = { $from } bis vor { $to }

vehicles-new-version = Neue Version…
vehicles-new-version-of = Neue Version von { $name }
vehicles-export-version = { $name } { $version } exportieren
vehicles-tab-installed = Installiert · { $count }
vehicles-tab-mine = Meine Fahrzeuge · { $count }
vehicles-status-custom = Eigenes
vehicles-status-update = Update
vehicles-status-current = Aktuell
vehicles-mine-empty = Eigene Fahrzeuge werden aus den Vorlagendateien des Spiels (DDS, PNG oder SVG) und ihren Spieldaten erstellt. Klicken Sie auf Eigenes Fahrzeug…, um eines zu erstellen.
vehicles-preview = Vorlage von { $name }
vehicles-details-of = Details zu { $name }
vehicles-installed-versions = Installierte Versionen
vehicles-update-summary = Update-Übersicht: { $version }
vehicles-changes-layout = Layout geändert: { $names }
vehicles-changes-resized = Größe geändert: { $names }
vehicles-changes-added = Neu: { $names }
vehicles-changes-removed = Entfernt: { $names }
vehicles-no-changes = Keine Änderungen angegeben
vehicles-projects = Projekte mit einer älteren Version
vehicles-projects-reading = Letzte Projekte werden gelesen …
vehicles-recent-only = Nur die letzten Projekte werden aufgeführt
vehicles-later = Später
vehicles-update-vehicle = { $name } aktualisieren
vehicles-update-hint = Öffnen Sie ein Projekt, das dieses Fahrzeug verwendet, um es zu aktualisieren.
vehicles-impact-textures = { $count ->
    [one] 1 Textur
   *[other] { $count } Texturen
}
vehicles-impact-resized = { $count ->
    [one] 1 Textur in der Größe geändert
   *[other] { $count } Texturen in der Größe geändert
}
vehicles-impact-removed = { $count } nicht mehr in dieser Version
vehicles-impact-nothing = Nichts neu zu positionieren
vehicles-impact-line = { $name }: { $impact }
vehicles-impact-unreadable = { $name }: nicht lesbar
vehicles-export = Exportieren…
vehicles-exported = { $file } exportiert
vehicles-export-failed = { $file } konnte nicht geschrieben werden: { $reason }
update-versions = { $name }: Version { $from } → { $to }
update-replaced = { $name }: Vorlage ersetzt
update-layout-changed = { $name }: Layout geändert, { $objects }
update-objects = { $count ->
    [0] nichts neu zu positionieren
    [one] 1 Objekt neu zu positionieren
   *[other] { $count } Objekte neu zu positionieren
}
update-changelog = Änderungen in Version { $version }:
update-resized = { $name }: Größe { $old } → { $new } px, Grafik skaliert
update-new-textures = Neu in dieser Version:
update-removed = { $name }: in dieser Version nicht mehr enthalten, Grafik bleibt erhalten
update-artwork-kept = Ihre Grafik bleibt erhalten. Sie können die Aktualisierung rückgängig machen.
update-apply = Aktualisieren

## New Project

new-project-no-vehicles-hint = Fahrzeugpakete enthalten die Vorlagen eines Lkw oder Anhängers. Probieren Sie die Beispielfahrzeuge aus, installieren Sie ein Paket oder erstellen Sie ein eigenes Fahrzeug.
new-project-create = Projekt erstellen
new-project-add-later = Sie können dem Projekt jederzeit Fahrzeuge hinzufügen.
new-project-no-game-vehicles = Noch kein { $game }-Fahrzeug installiert.
new-project-no-match = Kein Fahrzeug passt zur Suche.
new-project-custom-title = Fahrzeug ohne Paket
new-project-custom-hint = Erstellen Sie es aus seinen Vorlagendateien (DDS, PNG oder SVG) und seinen Spieldaten.
new-project-fleet = Ihre Flotte
new-project-check-vehicles = Markieren Sie Fahrzeuge in der Liste.
new-project-cabins = Kabinen
new-project-column-vehicle = Fahrzeug
new-project-column-type = Typ
new-project-column-package = Paket
new-project-counts = { $accessories ->
    [0] { $main } { $main ->
        [one] Haupttextur
       *[other] Haupttexturen
    }
    [one] { $main } { $main ->
        [one] Haupttextur
       *[other] Haupttexturen
    } + { $accessories } Zubehörteil
   *[other] { $main } { $main ->
        [one] Haupttextur
       *[other] Haupttexturen
    } + { $accessories } Zubehörteile
}
new-project-created = Erstellte Texturen
new-project-texture = { $texture } ({ $size })
new-project-total = { $count ->
    [one] { $count } Textur insgesamt
   *[other] { $count } Texturen insgesamt
}
new-project-vehicle-count = { $count ->
    [one] { $count } Fahrzeug
   *[other] { $count } Fahrzeuge
}
new-project-remove = Entfernen
vehicles-kind-all = Alle
vehicles-kind-trucks = Lkw
vehicles-kind-trailers = Anhänger

## Fleet

add-vehicle-none = Kein weiteres Fahrzeug dieses Spiels ist installiert. Installieren Sie ein Paket, um eines hinzuzufügen.
add-vehicle-add = Hinzufügen
fleet-error-other-game = Dieses Fahrzeug gehört zu einem anderen Spiel als das Projekt.
fleet-error-already-there = Dieses Fahrzeug ist bereits im Projekt.
fleet-error-textures = Wählen Sie mindestens eine Haupttextur dieses Fahrzeugs.
fleet-error-version = Die installierte Version unterscheidet sich von der des Projekts.
fleet-error-unknown = Dieses Fahrzeug ist nicht im Projekt.
textures-title = Texturen von { $name }
textures-missing-version = Version { $version } dieses Fahrzeugs ist nicht installiert. Aktualisieren Sie seine Vorlage, um die Texturen zu ändern.
textures-remove-confirm = Die Gestaltung von { $textures } wird mit ihnen entfernt.
textures-remove = Entfernen
textures-apply = Anwenden
copy-cabin-hint = Kopiert alle Objekte der gewählten Kabine an denselben Positionen auf { $texture }.
copy-cabin-objects = { $count ->
    [one] { $count } Objekt
   *[other] { $count } Objekte
 }
copy-cabin-empty = Diese Textur enthält kein Objekt zum Kopieren.
copy-cabin-copy = Kopieren
remove-vehicle-confirm = { $name } und seine Gestaltung aus dem Projekt entfernen?

# Export Mod
dialog-export-mod = Mod exportieren
filter-mods = Spiel-Mods
filter-mod-images = Bilder (PNG, JPEG)
mod-name = Name
mod-version = Version
mod-author = Autor
mod-description = Beschreibung
mod-price = Preis
mod-unlock = Freischaltstufe
mod-advanced = Erweitert
mod-internal-name = Interner Name
mod-internal-name-help = Benennt die Lackierung im Spiel: a–z, 0–9 und _. Wählen Sie einen eindeutigen Namen.
mod-icon = Shop-Symbol
mod-image = Bild im Mod-Manager
mod-choose-icon = Symbol wählen…
mod-generated-icon = Erzeugtes Symbol verwenden
mod-choose-image = Bild wählen…
mod-generated-image = Erzeugtes Bild verwenden
mod-picture-failed = { $file } kann nicht verwendet werden ({ $reason }).
mod-summary = Im Mod
mod-summary-cabins = { $texture } ({ $count ->
    [one] Kabine
   *[other] Kabinen
}: { $cabins })
mod-summary-every-cabin = { $texture } (alle Kabinen)
mod-summary-not-painted = { $texture } (nicht lackiert)
mod-summary-accessories = Zubehör: { $accessories }
mod-problem-name-empty = Der Mod braucht einen Namen.
mod-problem-name-invalid = Der Name darf weder ", \ noch Zeilenumbrüche enthalten.
mod-problem-version-invalid = Die Version darf weder ", \ noch Zeilenumbrüche enthalten.
mod-problem-author-invalid = Der Autor darf weder ", \ noch Zeilenumbrüche enthalten.
mod-problem-internal-empty = Der interne Name darf nicht leer sein.
mod-problem-internal-invalid = Der interne Name darf nur a–z, 0–9 und _ enthalten.
mod-problem-internal-long = Der interne Name darf höchstens { $max } Zeichen haben.
mod-problem-price = Der Preis muss größer als 0 sein.
mod-problem-same-path = { $first } und { $second } sind im Spiel dasselbe Fahrzeug ({ $path }): entfernen Sie eines davon.
mod-problem-game-data = { $vehicle } { $version } muss installiert sein, um den Mod zu exportieren.
mod-problem-bad-game-version = { $version } ist keine Spielversion: Schreiben Sie sie wie 1.56.* oder 1.56.2.
mod-problem-unsupported-game-version = { $vehicle } ({ $range }) unterstützt die Spielversion { $version } nicht.
mod-problem-no-common-game-version = { $first } ({ $first_range }) und { $second } ({ $second_range }) haben keine gemeinsame Spielversion: Aktualisieren oder entfernen Sie eines davon.
mod-warning-layout-changed = { $texture }: Layout geändert
mod-warning-not-in-version = { $texture }: nicht in dieser Version, nicht im Mod enthalten
mod-warning-empty-one = { $texture } ist leer und wird in der Farbe des Spiels exportiert
mod-warning-empty-many = { $count } Texturen leer, in der Farbe des Spiels exportiert
mod-export-done = Mod exportiert nach { $file }
mod-export-start = Exportieren
mod-destination = Ziel
mod-change-destination = Ändern…
mod-export-replace-question = { $file } ist bereits vorhanden. Ersetzen?
mod-export-replace = Ersetzen

## Import from Library

library-hint = Ihre Symbole, Farbfelder und Stile, geteilt von allen Projekten beider Spiele. Importierte Elemente sind Kopien, mit allem, was sie verwenden.
library-symbols = Symbole
library-swatches = Farbfelder
library-graphic-styles = Grafikstile
library-text-styles = Textstile
library-in-project = In diesem Projekt
library-import = Importieren
library-empty = Ihre Bibliothek ist leer. Klicken Sie mit der rechten Maustaste auf ein Symbol, ein Farbfeld oder einen Stil im Bereich Marke oder auf der Registerkarte Ressourcen und wählen Sie Zur Bibliothek hinzufügen, um es in allen Projekten zu verwenden.
library-remove = Aus Bibliothek entfernen
library-remove-confirm = { $name } aus der Bibliothek entfernen? Projekte, die es importiert haben, behalten ihre Kopie.
library-remove-button = Entfernen
library-unreadable-title = Bibliothek
library-unreadable = Die Bibliothek konnte nicht gelesen werden: Sie beginnt leer. Eine Kopie der Datei wurde als { $file } aufbewahrt.
library-unreadable-no-backup = Die Bibliothek konnte nicht gelesen werden: Sie beginnt leer.

prefs-mcp = KI-Assistent (MCP)
prefs-mcp-enable = Einem KI-Assistenten erlauben, im geöffneten Projekt zu zeichnen
prefs-mcp-hint = Ein MCP-Client wie Claude Desktop oder Claude Code kann sich mit dieser Adresse verbinden, um das geöffnete Projekt zu lesen und auf seinen Texturen zu zeichnen. Jede seiner Änderungen lässt sich rückgängig machen, und er kann keine Dateien speichern, exportieren oder öffnen.
prefs-mcp-port = Port
prefs-mcp-url = Adresse
prefs-mcp-copy = Kopieren
prefs-mcp-copied = Kopiert
prefs-mcp-off = Aus
prefs-mcp-on = An, wartet auf einen Client
prefs-mcp-connected = An, ein Client ist verbunden
prefs-mcp-port-in-use = Port { $port } wird von einem anderen Programm verwendet. Schalte den Server aus, wähle einen anderen Port und schalte ihn wieder ein.
prefs-mcp-failed = Der Server konnte nicht starten: { $error }

# Reiter Mein Spiel der Fahrzeugbibliothek und Bereich Spiele der Einstellungen.
vehicles-tab-game = Mein Spiel
game-header = { $game } { $version }
game-refresh = Aktualisieren
game-preferences = Einstellungen…
game-select-all = Alle auswählen
game-select-none = Keine auswählen
game-select-updates = Aktualisierungen auswählen
game-generate = Erzeugen ({ $count })
game-progress = { $done } von { $total }
game-making = { $name } wird erstellt…
game-stopping = Stopp nach { $name }…
game-status-not-generated = Nicht erzeugt
game-status-generated = Erzeugt { $version }
game-status-updated = Spiel aktualisiert
game-row-waiting = Wartet
game-row-working = In Arbeit
game-row-installed = Installiert { $version }
game-row-up-to-date = Aktuell
game-row-failed = Fehlgeschlagen
game-scanning = Spiel wird gelesen…
game-scan-failed = Das Spiel kann nicht gelesen werden: { $reason }
game-folder-problem = Der Spielordner kann nicht verwendet werden
game-no-folder = Kein Spielordner festgelegt
game-no-folder-hint = Legen Sie in den Einstellungen den Ordner fest, in dem { $game } installiert ist. Die offiziellen Lkw und Anhänger werden daraus auf diesem Computer erstellt.
game-detected = In Ihren Steam-Bibliotheken gefunden: { $folder }
game-use-detected = Diesen Ordner verwenden
game-ats-title = American Truck Simulator wird noch nicht unterstützt
game-ats-unsupported = Das Erzeugen von Fahrzeugen für American Truck Simulator wird noch nicht unterstützt. Der Ordner ist festgelegt und geprüft und damit bereit, sobald es so weit ist.
game-empty = Kein Fahrzeug gefunden
game-empty-hint = Dieser Spielordner enthält keinen Lkw und keinen Anhänger, der lackiert werden kann.
game-result = Installiert: { $installed } · Aktuell: { $skipped } · Fehlgeschlagen: { $failed }
game-result-cancelled = Abgebrochen. Installiert: { $installed } · Aktuell: { $skipped } · Fehlgeschlagen: { $failed }
game-failed-line = { $name }: { $reason }
game-finished-title = Mein Spiel
game-notice = Die Inhalte des Spiels gehören SCS Software. Diese Pakete werden auf diesem Computer für Ihren eigenen Gebrauch erstellt und nie weitergegeben. TruckPaint wird nicht von SCS Software unterstützt. Ein Spiel-Patch innerhalb derselben Versionsreihe wird nicht markiert: Wählen Sie ein Fahrzeug aus und erzeugen Sie es erneut, um es zu aktualisieren.
prefs-games-hint = TruckPaint liest diese Ordner, um die offiziellen Lkw und Anhänger für Ihren eigenen Gebrauch zu erstellen. Es wird nichts versendet.
prefs-game-detect = Erkennen
prefs-game-detect-hint = Das Spiel in Ihren Steam-Bibliotheken suchen
prefs-game-not-detected = Das Spiel wurde in Ihren Steam-Bibliotheken nicht gefunden. Wählen Sie stattdessen seinen Ordner.
prefs-game-choose = Auswählen…
prefs-game-choose-title = Ordner von { $game } auswählen
prefs-game-clear = Entfernen
prefs-game-version = Version { $version }
prefs-mod-folder = Mod-Ordner von { $game }
prefs-game-folder-label = Spielordner
prefs-mod-folder-label = Mod-Ordner
prefs-folder-not-set = Nicht festgelegt
prefs-button-for = { $action }, { $target }
prefs-game-folder-target = Spielordner { $game }
prefs-mod-choose-title = Mod-Ordner von { $game } auswählen
games-not-found = Dieser Ordner wurde nicht gefunden.
games-missing-file = { $file } fehlt in diesem Ordner.
games-not-a-game = Dieser Ordner enthält weder Euro Truck Simulator 2 noch American Truck Simulator.
games-unreadable = Die Spieldateien können nicht gelesen werden ({ $cause }).
games-is-ets2 = Das ist ein Ordner von Euro Truck Simulator 2.
games-is-ats = Das ist ein Ordner von American Truck Simulator.

first-run-step-language = Sprache und Spiel
first-run-step-vehicles = Fahrzeuge zum Lackieren
first-run-step-project = Erstes Projekt
first-run-step-of = Schritt { $n } von 3: { $name }
first-run-welcome = Willkommen bei TruckPaint
first-run-intro = Zeigen Sie TruckPaint, wo Ihr Spiel installiert ist. Es erstellt Ihre Lkw und Anhänger aus Ihrem eigenen Spiel auf diesem Computer, damit Sie gleich mit dem Lackieren beginnen können.
first-run-language = Sprache
first-run-games = Ihre Spiele
first-run-game-found = Spiel gefunden
first-run-game-not-found = Nicht gefunden
first-run-mod-found = Mod-Ordner gefunden
first-run-mod-not-found = Mod-Ordner nicht gefunden
first-run-choose-folder = Ordner auswählen…
first-run-next = Weiter
first-run-back = Zurück
first-run-skip = Überspringen
first-run-vehicles-intro = Haken Sie die Lkw und Anhänger an, die Sie lackieren möchten. Sie werden aus Ihrem eigenen Spiel auf diesem Computer erstellt.
first-run-checked = Angehakt: { $count }
first-run-filter-count = { $name } { $count }
first-run-scan-cancelled = Das Lesen des Spiels wurde abgebrochen. Sie können ohne Fahrzeuge fortfahren oder es erneut versuchen.
first-run-try-again = Erneut versuchen
first-run-generate = Erstellen und weiter
first-run-continue = Weiter
first-run-ets2-missing = Euro Truck Simulator 2 wurde nicht gefunden, daher können seine Fahrzeuge nicht erstellt werden.
first-run-ats-unsupported = Fahrzeuge für American Truck Simulator können noch nicht erstellt werden.
first-run-samples-hint = Sie können die Beispielfahrzeuge installieren, um TruckPaint gleich auszuprobieren. Mit Eigenes Fahrzeug… in Neues Projekt lässt sich später ein weiteres hinzufügen.
