import type { Row } from '@facile/muse';

/** The downloads and docs pages, in the order en, fr, es, de. */
export const pages = {
	'downloads.title': ['Download Flux', 'Télécharger Flux', 'Descargar Flux', 'Flux herunterladen'],
	'downloads.heading': ['Download Flux', 'Télécharger Flux', 'Descargar Flux', 'Flux herunterladen'],
	'downloads.line': [
		'A live system on a USB stick. Try Flux without touching your disk, then install it if it suits you.',
		'Un système live sur clé USB. Essayez Flux sans toucher à votre disque, puis installez-le s’il vous convient.',
		'Un sistema live en una memoria USB. Pruebe Flux sin tocar su disco y luego instálelo si le convence.',
		'Ein Live-System auf dem USB-Stick. Testen Sie Flux, ohne Ihre Festplatte anzurühren, und installieren Sie es, wenn es passt.'
	],
	'downloads.version': ['Version', 'Version', 'Versión', 'Version'],
	'downloads.notes': ['Release notes', 'Notes de version', 'Notas de la versión', 'Versionshinweise'],
	'downloads.all': ['All versions', 'Toutes les versions', 'Todas las versiones', 'Alle Versionen'],
	'downloads.x86.name': ['PC', 'PC', 'PC', 'PC'],
	'downloads.x86.line': [
		'Most laptops and desktops, Intel or AMD.',
		'La plupart des portables et des fixes, Intel ou AMD.',
		'La mayoría de portátiles y sobremesas, Intel o AMD.',
		'Die meisten Laptops und Desktops, Intel oder AMD.'
	],
	'downloads.arm.name': ['ARM', 'ARM', 'ARM', 'ARM'],
	'downloads.arm.line': [
		'UEFI ARM64 only: VMs (UTM, Parallels, QEMU), ARM servers and EDK2 boards. Not a stock Raspberry Pi or Apple Silicon bare metal.',
		'ARM64 UEFI uniquement : VM (UTM, Parallels, QEMU), serveurs ARM et cartes EDK2. Pas un Raspberry Pi d’origine ni un Mac Apple Silicon sans VM.',
		'Solo ARM64 UEFI: VM (UTM, Parallels, QEMU), servidores ARM y placas EDK2. No una Raspberry Pi de serie ni un Mac Apple Silicon sin VM.',
		'Nur ARM64 mit UEFI: VMs (UTM, Parallels, QEMU), ARM-Server und EDK2-Boards. Kein Raspberry Pi ab Werk und kein Apple Silicon ohne VM.'
	],
	'downloads.recommended': ['For this device', 'Pour cet appareil', 'Para este dispositivo', 'Für dieses Gerät'],
	'downloads.button': ['Download the ISO', 'Télécharger l’ISO', 'Descargar la ISO', 'ISO herunterladen'],
	'downloads.checksum': ['Checksum', 'Somme de contrôle', 'Suma de verificación', 'Prüfsumme'],
	'downloads.steps': ['Then, in three steps', 'Ensuite, en trois étapes', 'Después, en tres pasos', 'Danach in drei Schritten'],
	'downloads.verify.title': ['Check the file', 'Vérifier le fichier', 'Comprobar el archivo', 'Datei prüfen'],
	'downloads.verify.line': [
		'Download the checksums next to the ISO, then run this. It prints OK when the file is intact.',
		'Téléchargez les sommes de contrôle à côté de l’ISO, puis lancez ceci. La commande affiche OK si le fichier est intact.',
		'Descargue las sumas junto a la ISO y ejecute esto. Muestra OK si el archivo está intacto.',
		'Laden Sie die Prüfsummen neben das ISO und führen Sie das aus. Es zeigt OK, wenn die Datei intakt ist.'
	],
	'downloads.usb.title': ['Write it to a USB stick', 'L’écrire sur une clé USB', 'Grabarla en una memoria USB', 'Auf einen USB-Stick schreiben'],
	'downloads.usb.line': [
		'Use Impression on Linux or balenaEtcher on macOS and Windows. In a terminal, replace /dev/sdX with your stick: everything on it is erased.',
		'Utilisez Impression sur Linux ou balenaEtcher sur macOS et Windows. Dans un terminal, remplacez /dev/sdX par votre clé : tout ce qu’elle contient est effacé.',
		'Use Impression en Linux o balenaEtcher en macOS y Windows. En una terminal, sustituya /dev/sdX por su memoria: todo su contenido se borra.',
		'Nutzen Sie Impression unter Linux oder balenaEtcher unter macOS und Windows. Im Terminal ersetzen Sie /dev/sdX durch Ihren Stick: alles darauf wird gelöscht.'
	],
	'downloads.boot.title': ['Boot and install', 'Démarrer et installer', 'Arrancar e instalar', 'Starten und installieren'],
	'downloads.boot.line': [
		'Start your computer from the stick. Flux opens on its desktop; run sudo flux-install when you want to keep it.',
		'Démarrez l’ordinateur sur la clé. Flux s’ouvre sur son bureau ; lancez sudo flux-install quand vous voulez le garder.',
		'Arranque el ordenador desde la memoria. Flux se abre en su escritorio; ejecute sudo flux-install cuando quiera quedárselo.',
		'Starten Sie den Computer vom Stick. Flux öffnet seinen Desktop; führen Sie sudo flux-install aus, wenn Sie es behalten wollen.'
	],
	'downloads.next': ['Installation guide', 'Guide d’installation', 'Guía de instalación', 'Installationsanleitung'],
	'downloads.other': ['Other ways to run Flux', 'D’autres façons d’utiliser Flux', 'Otras formas de usar Flux', 'Weitere Wege zu Flux'],
	'downloads.vm.title': ['Try it in a VM', 'L’essayer dans une VM', 'Probarlo en una VM', 'In einer VM testen'],
	'downloads.vm.line': ['With Nix installed, one command boots the desktop.', 'Avec Nix installé, une commande démarre le bureau.', 'Con Nix instalado, un comando arranca el escritorio.', 'Mit Nix startet ein Befehl den Desktop.'],
	'downloads.nixos.title': ['Already on NixOS', 'Déjà sous NixOS', 'Ya en NixOS', 'Schon auf NixOS'],
	'downloads.nixos.line': ['Start a machine config that imports Flux.', 'Créez une config machine qui importe Flux.', 'Cree una configuración que importa Flux.', 'Legen Sie eine Maschinenkonfiguration an, die Flux importiert.'],
	'downloads.boite.title': ['In Boîte', 'Dans Boîte', 'En Boîte', 'In Boîte'],
	'downloads.boite.line': ['Every Boîte dev VM runs Flux headless.', 'Chaque VM de dev Boîte tourne sous Flux, sans bureau.', 'Cada VM de desarrollo de Boîte ejecuta Flux sin escritorio.', 'Jede Boîte-Dev-VM läuft mit Flux ohne Desktop.'],
	'docs.title': ['Flux docs', 'Documentation Flux', 'Documentación de Flux', 'Flux-Doku'],
	'docs.heading': ['Install and use Flux', 'Installer et utiliser Flux', 'Instalar y usar Flux', 'Flux installieren und nutzen'],
	'docs.line': [
		'From the download to the first update, one short page per step.',
		'Du téléchargement à la première mise à jour, une courte page par étape.',
		'De la descarga a la primera actualización, una página corta por paso.',
		'Vom Download bis zum ersten Update, eine kurze Seite pro Schritt.'
	],
	'docs.nav': ['Pages', 'Pages', 'Páginas', 'Seiten'],
	'docs.prev': ['Previous', 'Précédent', 'Anterior', 'Zurück'],
	'docs.next': ['Next', 'Suivant', 'Siguiente', 'Weiter'],
	'docs.all': ['All pages', 'Toutes les pages', 'Todas las páginas', 'Alle Seiten']
} satisfies Record<string, Row>;
