## Tailles d’export et erreurs

size-mb = { $value } Mo
size-kb = { $value } Ko
size-about = environ { $value }
export-stopped = l’export s’est arrêté de manière inattendue

## Enregistrement et ouverture des projets

file-save-failed-title = Enregistrement impossible
file-save-failed = Impossible d’enregistrer { $file }.
file-save-failed-reason = Impossible d’enregistrer { $file } ({ $reason }). Vos modifications sont toujours ouvertes.
file-open-failed-title = Impossible d’ouvrir le projet
file-restore-failed-title = Impossible de restaurer le projet
file-recovered-copy = La copie récupérée
file-not-a-project = { $file } n’est pas un projet TruckPaint.
file-damaged = { $file } est endommagé et ne peut pas être ouvert.
opening-project = Ouverture de { $name }…
file-dialog-waiting = En attente de la fenêtre de fichier…
file-newer-version = { $file } a été créé avec une version plus récente de TruckPaint.
file-io = Impossible de lire ou d’écrire { $file } ({ $reason }).

## Boîtes de dialogue de fichiers

dialog-open-project = Ouvrir un projet
dialog-save-project = Enregistrer le projet
dialog-export-texture = Exporter la texture
filter-project = Projet TruckPaint
filter-images = Images

## Écran d’accueil

home-recovered = Projets récupérés
home-recovered-hint = TruckPaint s’est fermé de manière inattendue. Ces projets contenaient des modifications non enregistrées.
home-never-saved = Jamais enregistré
home-recovery-damaged = Copie de récupération endommagée
home-recovery-damaged-hint = Cette copie ne peut pas être restaurée.
home-recovered-item = { $name } récupéré
home-restore = Restaurer
home-discard = Ignorer
app-tagline = Éditeur de livrées pour Euro Truck Simulator 2 et American Truck Simulator
home-new-project = Nouveau projet
home-new-project-tip = Créer un nouveau projet de livrée ({ $shortcut })
home-open-tip = Ouvrir un fichier de projet ({ $shortcut })
home-preferences = Préférences
home-recent = Projets récents
home-recent-empty = Aucun projet récent
home-recent-empty-hint = Les projets que vous ouvrez apparaîtront ici. Commencez par créer un nouveau projet.
home-recent-empty-create = Créer votre premier projet
home-recent-missing-item = { $name } (fichier introuvable)
home-remove-recent = Retirer de la liste
home-file-not-found = Fichier introuvable
home-open-recent = Ouvrir { $path }
home-file-moved = Le fichier de ce projet a été déplacé ou supprimé.

## Boîtes de dialogue

unsaved-title = Enregistrer les modifications de « { $name } » avant de fermer ?
unsaved-hint = Vos modifications seront perdues si vous ne les enregistrez pas.
button-dont-save = Ne pas enregistrer
button-cancel = Annuler
button-ok = OK
button-done = Terminé
button-close = Fermer
new-project-name = Nom du projet
prefs-interface = Interface
prefs-ui-scale = Échelle de l’interface
prefs-text-size = Taille du texte
prefs-scale-hint = Les modifications s’appliquent immédiatement. 100 % suit le facteur d’échelle de votre écran.
prefs-canvas = Zone de travail
prefs-grid-spacing = Espacement de la grille
prefs-reset = Rétablir les valeurs par défaut
prefs-tab-general = Général
prefs-tab-games = Jeux
prefs-tab-ai = Assistant IA
shortcuts-temporary-hand = Outil Main temporaire
shortcuts-hold-space = Maintenir Espace
about-version = Version { $version }
about-tagline = Éditeur de livrées pour Euro Truck Simulator 2 et American Truck Simulator.

## Boîte de dialogue Exporter la texture

export-done = { $file } exporté
export-failed-title = Export impossible
export-failed = Impossible d’écrire { $file } ({ $reason }).
export-preview = Aperçu de l’export
export-info = { $width } × { $height } px · { $format } · { $bytes }
export-progress = Export de { $file }…
export-start = Exporter…
export-format = Format
export-dds-bc3 = BC3 / DXT5 (recommandé)
export-dds-rgba = RGBA non compressé
export-dds-encoding = Encodage DDS
export-size = Taille
export-background = Arrière-plan
export-transparent = Transparent
export-background-color = Couleur d’arrière-plan
prefs-language = Langue
prefs-system-title-bar = Utiliser la barre de titre du système
prefs-system-title-bar-hint = Le cadre et la barre de titre de la fenêtre sont ceux du système, avec les menus sur une ligne en dessous. Utile si la fenêtre ne peut pas être déplacée ou redimensionnée.

## Vehicle packages

pkg-install-failed = { $file } n’a pas pu être installé : { $reason }
pkg-no-library = aucun dossier de bibliothèque de véhicules n’est disponible.
pkg-io = il n’a pas pu être lu ou écrit ({ $reason }).
pkg-not-a-zip = ce n’est pas un paquet de véhicule.
pkg-no-manifest = il ne contient pas de manifeste vehicle.json.
pkg-bad-manifest = son manifeste est invalide ({ $reason }).
pkg-newer-format = il a été créé pour une version plus récente de TruckPaint.
pkg-bad-id = son identifiant « { $id } » est invalide.
pkg-no-main-texture = il n’a aucune texture principale.
pkg-bad-changelog = son journal des modifications dépasse 1000 caractères.
pkg-bad-preview = son maillage d’aperçu « { $path } » est invalide : { $cause }
pkg-bad-game-path = son chemin dans le jeu « { $path } » est invalide (mots en a–z, 0–9 et _ séparés par des points).
pkg-bad-game-id = l’identifiant du jeu « { $id } » est invalide (mots en a–z, 0–9 et _ séparés par des points).
pkg-missing-game-ids = « { $texture } » n’a aucun identifiant du jeu.
pkg-duplicate-part = deux textures ont l’identifiant « { $id } ».
pkg-duplicate-game-id = l’identifiant du jeu « { $id } » est listé deux fois.
pkg-bad-also-paints = l’accessoire « { $texture } » liste des accessoires qu’il peint aussi : seule une texture principale le peut.
pkg-bad-size = la texture « { $texture } » a une taille invalide ({ $size }) : elle doit être une largeur et une hauteur, [largeur, hauteur], chacune une puissance de deux de 64 à 8192.
pkg-unsafe-path = il contient un chemin non sûr ({ $path }).
pkg-too-large = il est trop volumineux.
pkg-missing-template = le gabarit de « { $texture } » est manquant.
pkg-bad-template = le gabarit de « { $texture } » n’est pas une image PNG ou SVG lisible.
pkg-template-too-large = le gabarit de « { $texture } » est trop grand.

## Custom vehicles

custom-build-failed = { $name } n’a pas pu être créé : { $reason }
custom-template-unconvertible = le gabarit de « { $texture } » ne peut pas être converti.
custom-file-not-image = { $file } ne peut pas être utilisé : ce n’est pas une image PNG, DDS ou SVG.
custom-file-unsupported-dds = { $file } ne peut pas être lu : sa compression DDS ({ $format }) n’est pas prise en charge. Enregistrez-le en BC1, BC2, BC3 ou RGB(A) non compressé.
custom-file-damaged = { $file } ne peut pas être lu : le fichier est endommagé ou incomplet.
custom-file-too-large = { $file } ne peut pas être utilisé : il dépasse { $size } px.
custom-file-io = { $file } ne peut pas être lu ({ $reason }).
custom-build-stopped = { $name } n’a pas pu être créé : la création s’est arrêtée de façon inattendue.
custom-title = Véhicule personnalisé
custom-new-version-title = Nouvelle version de { $name }
custom-open = Véhicule personnalisé…
custom-open-library = Véhicule personnalisé…
custom-version = Version
custom-what-changed = Modifications
custom-what-changed-hint = Gabarit des portes redessiné après la mise à jour du jeu…
custom-what-changed-count = { $count } / { $max } caractères
custom-problem-changelog = « Modifications » dépasse { $max } caractères.
custom-name = Nom
custom-brand = Marque
custom-kind = Type
custom-game = Jeu
custom-game-path = Chemin dans le jeu
custom-game-path-hint = Comme dans les définitions du jeu, p. ex. scania.r_2016
custom-game-versions = Versions du jeu
custom-game-versions-hint = Toutes les versions, ou une plage comme >=1.53
custom-alt-uv = Jeu d’UV alternatif
custom-colour-picker = Choix de couleur
custom-textures = Textures
custom-drop-hint = Déposez ici les gabarits du jeu (PNG, DDS ou SVG), ou ajoutez-les. Chaque fichier est une texture.
custom-add-templates = Ajouter des gabarits…
custom-row-name = Nom de la texture { $n }
custom-row-role = Rôle de la texture { $n }
custom-row-width = Largeur de la texture { $n }
custom-row-height = Hauteur de la texture { $n }
custom-row-role-short = Rôle
custom-row-width-short = Largeur
custom-row-height-short = Hauteur
custom-row-game-ids = Identifiants de jeu de la texture { $n }
custom-row-replace = Remplacer…
custom-row-replace-name = Remplacer le gabarit de la texture { $n }
custom-row-remove = Supprimer la texture { $n }
custom-role-main = Texture principale
custom-role-accessory = Accessoire
custom-hint-cabins = Noms internes des cabines qui utilisent cette disposition, p. ex. highline, highline_8x4
custom-hint-cabins-optional = Noms internes des cabines qui utilisent cette disposition ; laissez vide pour peindre toutes les cabines
custom-hint-accessories = Identifiants des accessoires couverts par cette texture, p. ex. mirror.painted, s_mirror.painted
custom-file = { $file } · { $width } × { $height } px
custom-template-scaled = Le gabarit sera mis à l’échelle en { $size }.
custom-scs-reminder = Les gabarits des jeux de base appartiennent à SCS Software : utilisez-les pour vos propres livrées, et vérifiez leur licence avant de partager un paquet.
custom-create = Créer
custom-building = Création du paquet… ({ $done }/{ $total })
custom-building-plain = Création du paquet…
custom-problem-name = Saisissez le nom du véhicule.
custom-problem-brand = Saisissez la marque du véhicule.
custom-problem-game-path = Le chemin dans le jeu est fait de mots en a–z, 0–9 et _ séparés par des points, comme dans les définitions du jeu, p. ex. scania.r_2016.
custom-problem-game-versions = Saisissez une plage de versions comme >=1.53, ou laissez vide pour toutes les versions.
custom-problem-version = Saisissez une version comme 1.1.0.
custom-problem-version-not-higher = La version doit être supérieure à { $version }.
custom-problem-installed = { $id } est déjà installé. Créez-en une nouvelle version avec Nouvelle version… dans la bibliothèque de véhicules, ou changez le nom.
custom-problem-no-main = Faites de l’une des textures une texture principale.
custom-problem-trailer-main = Une remorque a une seule texture principale : faites des autres des accessoires.
custom-problem-texture-name = Saisissez le nom de la texture.
custom-problem-cabins = Saisissez les noms internes des cabines qui utilisent cette disposition.
custom-problem-accessory-ids = Saisissez les identifiants des accessoires couverts par cette texture.
custom-problem-game-id = « { $id } » n’est pas un identifiant de jeu valide : utilisez des mots en a–z, 0–9 et _ séparés par des points.
custom-problem-duplicate-game-id = « { $id } » est déjà utilisé par une autre texture.

## Vehicle library and updates

filter-packages = Paquets de véhicules
filter-templates = Gabarits (PNG, DDS, SVG)
dialog-export-package = Exporter le paquet de véhicule (les gabarits des jeux de base appartiennent à SCS Software)
vehicles-search = Rechercher un véhicule
vehicles-all-games = Tous les jeux
vehicles-game-filter = Jeu
vehicles-all-kinds = Camions et remorques
vehicles-kind-filter = Type
vehicles-truck = Camion
vehicles-trailer = Remorque
vehicles-installed = { $name } { $version } installé
vehicles-empty = Aucun véhicule installé
vehicles-empty-hint = Les paquets de véhicules (.tpv) contiennent les gabarits d’un camion ou d’une remorque. Installez-en un pour y peindre une livrée.
vehicles-details-any-version = Toutes les versions du jeu
vehicles-details-versions = Versions du jeu : { $versions }
vehicles-details-main = { $count ->
    [one] Texture principale
   *[other] Textures principales
} : { $names }
vehicles-details-accessories = { $count ->
    [0] Aucun accessoire
    [one] { $count } accessoire
   *[other] { $count } accessoires
}
vehicles-version = Version { $version }
vehicles-remove-version = Supprimer { $name } { $version }
vehicles-remove-confirm = Supprimer { $name } de la bibliothèque ? Les projets conservent leurs gabarits.
vehicles-remove = Supprimer
vehicles-install = Installer…
vehicles-install-sample = Installer les véhicules d’exemple

# Plages de versions du jeu en mots (« >=1.56 » se lit « 1.56 ou ultérieure »).
version-range-any = toutes les versions
version-range-from = { $version } ou ultérieure
version-range-until = { $version } ou antérieure
version-range-exact = { $version }
version-range-between = de { $from } à { $to } exclue

vehicles-new-version = Nouvelle version…
vehicles-new-version-of = Nouvelle version de { $name }
vehicles-export-version = Exporter { $name } { $version }
vehicles-tab-installed = Installés · { $count }
vehicles-tab-mine = Mes véhicules · { $count }
vehicles-status-custom = Personnalisé
vehicles-status-update = Mise à jour
vehicles-status-current = À jour
vehicles-mine-empty = Les véhicules personnalisés sont créés à partir des fichiers de gabarit du jeu (DDS, PNG ou SVG) et de leurs données de jeu. Cliquez sur Véhicule personnalisé… pour en créer un.
vehicles-preview = Gabarit de { $name }
vehicles-details-of = Détails de { $name }
vehicles-installed-versions = Versions installées
vehicles-update-summary = Résumé de la mise à jour : { $version }
vehicles-changes-layout = Disposition modifiée : { $names }
vehicles-changes-resized = Taille changée : { $names }
vehicles-changes-added = Nouveau : { $names }
vehicles-changes-removed = Retiré : { $names }
vehicles-no-changes = Aucune modification indiquée
vehicles-projects = Projets utilisant une version plus ancienne
vehicles-projects-reading = Lecture des projets récents…
vehicles-recent-only = Seuls les projets récents sont listés
vehicles-later = Plus tard
vehicles-update-vehicle = Mettre à jour { $name }
vehicles-update-hint = Ouvrez un projet qui utilise ce véhicule pour le mettre à jour.
vehicles-impact-textures = { $count ->
    [one] 1 texture
   *[other] { $count } textures
}
vehicles-impact-resized = { $count ->
    [one] 1 texture redimensionnée
   *[other] { $count } textures redimensionnées
}
vehicles-impact-removed = { $count ->
    [one] 1 absente de cette version
   *[other] { $count } absentes de cette version
}
vehicles-impact-nothing = Rien à repositionner
vehicles-impact-line = { $name } : { $impact }
vehicles-impact-unreadable = { $name } : illisible
vehicles-export = Exporter…
vehicles-exported = { $file } exporté
vehicles-export-failed = { $file } n’a pas pu être écrit : { $reason }
update-versions = { $name } : version { $from } → { $to }
update-replaced = { $name } : gabarit remplacé
update-layout-changed = { $name } : disposition modifiée, { $objects }
update-objects = { $count ->
    [0] rien à repositionner
    [one] 1 objet à repositionner
   *[other] { $count } objets à repositionner
}
update-changelog = Modifications de la version { $version } :
update-resized = { $name } : taille { $old } → { $new } px, motif mis à l’échelle
update-new-textures = Nouveau dans cette version :
update-removed = { $name } : absente de cette version, motif conservé
update-artwork-kept = Votre motif est conservé. Vous pouvez annuler la mise à jour.
update-apply = Mettre à jour

## New Project

new-project-no-vehicles-hint = Les paquets de véhicules contiennent les gabarits d’un camion ou d’une remorque. Essayez les véhicules d’exemple, installez un paquet ou créez un véhicule personnalisé.
new-project-create = Créer le projet
new-project-add-later = Vous pourrez ajouter des véhicules au projet à tout moment.
new-project-no-game-vehicles = Aucun véhicule { $game } n’est encore installé.
new-project-no-match = Aucun véhicule ne correspond à la recherche.
new-project-custom-title = Véhicule sans paquet
new-project-custom-hint = Créez-le à partir de ses fichiers de gabarit (DDS, PNG ou SVG) et de ses données de jeu.
new-project-fleet = Votre flotte
new-project-check-vehicles = Cochez des véhicules dans la liste.
new-project-cabins = Cabines
new-project-column-vehicle = Véhicule
new-project-column-type = Type
new-project-column-package = Package
new-project-counts = { $accessories ->
    [0] { $main } { $main ->
        [one] principale
       *[other] principales
    }
    [one] { $main } { $main ->
        [one] principale
       *[other] principales
    } + { $accessories } accessoire
   *[other] { $main } { $main ->
        [one] principale
       *[other] principales
    } + { $accessories } accessoires
}
new-project-created = Textures créées
new-project-texture = { $texture } ({ $size })
new-project-total = { $count ->
    [one] { $count } texture au total
   *[other] { $count } textures au total
}
new-project-vehicle-count = { $count ->
    [one] { $count } véhicule
   *[other] { $count } véhicules
}
new-project-remove = Retirer
vehicles-kind-all = Tous
vehicles-kind-trucks = Camions
vehicles-kind-trailers = Remorques

## Fleet

add-vehicle-none = Aucun autre véhicule de ce jeu n’est installé. Installez un paquet pour en ajouter un.
add-vehicle-add = Ajouter
fleet-error-other-game = Ce véhicule est pour un autre jeu que le projet.
fleet-error-already-there = Ce véhicule est déjà dans le projet.
fleet-error-textures = Choisissez au moins une texture principale de ce véhicule.
fleet-error-version = La version installée diffère de celle du projet.
fleet-error-unknown = Ce véhicule n’est pas dans le projet.
textures-title = Textures de { $name }
textures-missing-version = La version { $version } de ce véhicule n’est pas installée. Mettez à jour son gabarit pour changer ses textures.
textures-remove-confirm = Le travail sur { $textures } sera supprimé avec elles.
textures-remove = Supprimer
textures-apply = Appliquer
copy-cabin-hint = Copie tous les objets de la cabine choisie sur { $texture }, aux mêmes positions.
copy-cabin-objects = { $count ->
    [one] { $count } objet
   *[other] { $count } objets
    }
copy-cabin-empty = Cette texture ne contient aucun objet à copier.
copy-cabin-copy = Copier
remove-vehicle-confirm = Retirer { $name } et son travail du projet ?

# Export Mod
dialog-export-mod = Exporter le mod
filter-mods = Mods de jeu
filter-mod-images = Images (PNG, JPEG)
mod-name = Nom
mod-version = Version
mod-author = Auteur
mod-description = Description
mod-price = Prix
mod-unlock = Niveau de déblocage
mod-advanced = Avancé
mod-internal-name = Nom interne
mod-internal-name-help = Nomme la peinture dans le jeu : a–z, 0–9 et _. Choisissez-le unique.
mod-icon = Icône de la boutique
mod-image = Image du gestionnaire de mods
mod-choose-icon = Choisir l’icône…
mod-generated-icon = Utiliser l’icône générée
mod-choose-image = Choisir l’image…
mod-generated-image = Utiliser l’image générée
mod-picture-failed = { $file } ne peut pas être utilisé ({ $reason }).
mod-summary = Dans le mod
mod-summary-cabins = { $texture } ({ $count ->
    [one] cabine
   *[other] cabines
} : { $cabins })
mod-summary-every-cabin = { $texture } (toutes les cabines)
mod-summary-not-painted = { $texture } (non peinte)
mod-summary-accessories = Accessoires : { $accessories }
mod-problem-name-empty = Le mod doit avoir un nom.
mod-problem-name-invalid = Le nom ne peut pas contenir ", \ ni de retour à la ligne.
mod-problem-version-invalid = La version ne peut pas contenir ", \ ni de retour à la ligne.
mod-problem-author-invalid = L’auteur ne peut pas contenir ", \ ni de retour à la ligne.
mod-problem-internal-empty = Le nom interne ne peut pas être vide.
mod-problem-internal-invalid = Le nom interne ne peut utiliser que a–z, 0–9 et _.
mod-problem-internal-long = Le nom interne peut avoir au plus { $max } caractères.
mod-problem-price = Le prix doit être supérieur à 0.
mod-problem-same-path = { $first } et { $second } sont le même véhicule dans le jeu ({ $path }) : retirez-en un.
mod-problem-game-data = { $vehicle } { $version } doit être installé pour exporter le mod.
mod-problem-bad-game-version = { $version } n’est pas une version du jeu : écrivez-la comme 1.56.* ou 1.56.2.
mod-problem-unsupported-game-version = { $vehicle } ({ $range }) ne prend pas en charge la version du jeu { $version }.
mod-problem-no-common-game-version = { $first } ({ $first_range }) et { $second } ({ $second_range }) n’ont aucune version du jeu en commun : mettez à jour ou retirez l’un des deux.
mod-warning-layout-changed = { $texture } : disposition modifiée
mod-warning-not-in-version = { $texture } : absente de cette version, exclue du mod
mod-warning-empty-one = { $texture } est vide, exportée avec la teinte du jeu
mod-warning-empty-many = { $count } textures vides, exportées avec la teinte du jeu
mod-export-done = Mod exporté dans { $file }
mod-export-start = Exporter
mod-destination = Destination
mod-change-destination = Modifier…
mod-export-replace-question = { $file } existe déjà. Le remplacer ?
mod-export-replace = Remplacer

## Import from Library

library-hint = Vos symboles, nuances et styles, partagés par tous les projets des deux jeux. Les éléments importés sont des copies, avec tout ce qu'ils utilisent.
library-symbols = Symboles
library-swatches = Nuances
library-graphic-styles = Styles graphiques
library-text-styles = Styles de texte
library-in-project = Dans ce projet
library-import = Importer
library-empty = Votre bibliothèque est vide. Faites un clic droit sur un symbole, une nuance ou un style dans l'espace Marque ou l'onglet Ressources et choisissez Ajouter à la bibliothèque pour l'utiliser dans tous vos projets.
library-remove = Retirer de la bibliothèque
library-remove-confirm = Retirer { $name } de la bibliothèque ? Les projets qui l'ont importé gardent leur copie.
library-remove-button = Retirer
library-unreadable-title = Bibliothèque
library-unreadable = La bibliothèque n'a pas pu être lue : elle repart vide. Une copie du fichier a été conservée sous le nom { $file }.
library-unreadable-no-backup = La bibliothèque n'a pas pu être lue : elle repart vide.

prefs-mcp = Assistant IA (MCP)
prefs-mcp-enable = Permettre à un assistant IA de dessiner sur le projet ouvert
prefs-mcp-not-allowed = Non autorisé
prefs-mcp-allowed = Autorisé
prefs-mcp-cannot-save = Enregistrer ou exporter des fichiers
prefs-mcp-can-import = Importer une image depuis un chemin de fichier qu’il indique
prefs-mcp-can-draw = Dessiner sur les textures (chaque modification peut être annulée)
prefs-mcp-can-read = Lire le projet ouvert
prefs-mcp-can-title = Ce que l’assistant peut faire
prefs-mcp-enable-hint = Via MCP. Chaque modification peut être annulée.
prefs-mcp-hint = Un client MCP, comme Claude Desktop ou Claude Code, peut se connecter à cette adresse.
prefs-mcp-port = Port
prefs-mcp-url = Adresse
prefs-mcp-copy = Copier
prefs-mcp-copied = Copié
prefs-mcp-off = Désactivé
prefs-mcp-on = Activé, en attente d’un client
prefs-mcp-connected = Activé, un client est connecté
prefs-mcp-port-in-use = Le port { $port } est utilisé par un autre programme. Désactivez le serveur, choisissez un autre port, puis réactivez-le.
prefs-mcp-failed = Le serveur n’a pas pu démarrer : { $error }

# Onglet Mon jeu de la bibliothèque de véhicules, et section Jeux des préférences.
vehicles-tab-game = Mon jeu
game-header = { $game } { $version }
game-refresh = Actualiser
game-preferences = Préférences…
game-select-all = Tout sélectionner
game-select-none = Ne rien sélectionner
game-select-updates = Sélectionner les mises à jour
game-generate = Générer ({ $count })
game-progress = { $done } sur { $total }
game-making = Création de { $name }…
game-stopping = Arrêt après { $name }…
game-status-not-generated = Non généré
game-status-generated = Généré { $version }
game-status-updated = Jeu mis à jour
game-row-waiting = En attente
game-row-working = En cours
game-row-installed = Installé { $version }
game-row-up-to-date = À jour
game-row-failed = Échec
game-scanning = Lecture du jeu…
game-scan-failed = Le jeu ne peut pas être lu : { $reason }
game-folder-problem = Le dossier du jeu est inutilisable
game-no-folder = Aucun dossier de jeu défini
game-no-folder-hint = Indiquez dans les préférences le dossier où { $game } est installé. Les camions et remorques officiels en sont tirés, sur cet ordinateur.
game-detected = Trouvé dans vos bibliothèques Steam : { $folder }
game-use-detected = Utiliser ce dossier
game-ats-title = American Truck Simulator n’est pas encore pris en charge
game-ats-unsupported = La génération des véhicules d’American Truck Simulator n’est pas encore prise en charge. Son dossier est défini et vérifié : il sera prêt dès que ce sera le cas.
game-empty = Aucun véhicule trouvé
game-empty-hint = Ce dossier de jeu ne contient aucun camion ni remorque qui puisse être peint.
game-result = Installés : { $installed } · À jour : { $skipped } · Échecs : { $failed }
game-result-cancelled = Annulé. Installés : { $installed } · À jour : { $skipped } · Échecs : { $failed }
game-failed-line = { $name } : { $reason }
game-finished-title = Mon jeu
game-notice = Le contenu du jeu appartient à SCS Software. Ces paquets sont créés sur cet ordinateur pour votre usage personnel et ne sont jamais partagés. TruckPaint n’est pas approuvé par SCS Software. Un correctif du jeu qui garde la même série de versions n’est pas signalé : cochez un véhicule et générez-le de nouveau pour l’actualiser.
prefs-games-hint = TruckPaint lit ces dossiers pour créer les camions et remorques officiels, pour votre usage personnel. Rien n’est envoyé nulle part.
prefs-game-detect = Détecter automatiquement
prefs-game-detect-hint = Chercher le jeu dans vos bibliothèques Steam
prefs-game-not-detected = Le jeu n’a pas été trouvé dans vos bibliothèques Steam. Choisissez plutôt son dossier.
prefs-game-choose = Choisir…
prefs-game-choose-title = Choisir le dossier de { $game }
prefs-game-clear = Effacer
prefs-game-version = Version { $version }
prefs-mod-folder = Dossier des mods de { $game }
prefs-game-folder-label = Dossier du jeu
prefs-mod-folder-label = Dossier des mods
prefs-folder-not-set = Non défini
prefs-button-for = { $action }, { $target }
prefs-game-folder-target = Dossier du jeu { $game }
prefs-game-state-problem = Problème de dossier
prefs-mod-choose-title = Choisir le dossier des mods de { $game }
games-not-found = Ce dossier est introuvable.
games-missing-file = { $file } est absent de ce dossier.
games-not-a-game = Ce dossier ne contient ni Euro Truck Simulator 2 ni American Truck Simulator.
games-unreadable = Les fichiers du jeu ne peuvent pas être lus ({ $cause }).
games-is-ets2 = C’est un dossier d’Euro Truck Simulator 2.
games-is-ats = C’est un dossier d’American Truck Simulator.

first-run-step-language = Langue et jeu
first-run-step-vehicles = Véhicules à peindre
first-run-step-project = Premier projet
first-run-step-of = Étape { $n } sur 3 : { $name }
first-run-welcome = Bienvenue dans TruckPaint
first-run-intro = Indiquez à TruckPaint où votre jeu est installé. Il crée vos camions et vos remorques à partir de votre propre jeu, sur cet ordinateur, pour que vous puissiez commencer à peindre.
first-run-language = Langue
first-run-games = Vos jeux
first-run-game-found = Jeu trouvé
first-run-game-not-found = Introuvable
first-run-next = Suivant
first-run-back = Retour
first-run-skip = Passer
first-run-vehicles-intro = Cochez les camions et les remorques que vous voulez peindre. Ils sont créés à partir de votre propre jeu, sur cet ordinateur.
first-run-checked = Cochés : { $count }
first-run-filter-count = { $name } { $count }
first-run-scan-cancelled = La lecture du jeu a été annulée. Vous pouvez continuer sans véhicule ou réessayer.
first-run-try-again = Réessayer
first-run-generate = Créer et continuer
first-run-continue = Continuer
first-run-ets2-missing = Euro Truck Simulator 2 est introuvable : ses véhicules ne peuvent pas être créés.
first-run-ats-unsupported = La création de véhicules pour American Truck Simulator n’est pas encore prise en charge.
first-run-samples-hint = Vous pouvez installer les véhicules d’exemple pour essayer TruckPaint tout de suite. Véhicule personnalisé… dans Nouveau projet permet d’en ajouter un autre plus tard.
