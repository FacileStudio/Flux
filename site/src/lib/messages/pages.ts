import type { Row } from '@facile/muse';

/** The downloads and docs pages, in the order en, fr, es, de. */
export const pages = {
	'downloads.title': ['Download Flux', 'Télécharger Flux', 'Descargar Flux', 'Flux herunterladen'],
	'downloads.heading': ['Download the beta', 'Télécharger la bêta', 'Descargar la beta', 'Beta herunterladen'],
	'downloads.line': [
		'One live ISO per architecture. Boot it from a USB stick, try Flux, then install it if you like it.',
		'Une ISO live par architecture. Démarrez-la depuis une clé USB, essayez Flux, puis installez-le s’il vous plaît.',
		'Una ISO live por arquitectura. Arránquela desde una memoria USB, pruebe Flux y luego instálelo si le gusta.',
		'Ein Live-ISO pro Architektur. Starten Sie es vom USB-Stick, testen Sie Flux und installieren Sie es, wenn es Ihnen gefällt.'
	],
	'downloads.beta': [
		'Flux is in open beta. The stable version comes around March 2027.',
		'Flux est en bêta ouverte. La version stable arrive vers mars 2027.',
		'Flux está en beta abierta. La versión estable llega hacia marzo de 2027.',
		'Flux ist in der offenen Beta. Die stabile Version kommt um März 2027.'
	],
	'downloads.verify.title': ['Check the download', 'Vérifier le téléchargement', 'Comprobar la descarga', 'Download prüfen'],
	'downloads.verify.line': [
		'Run this in the folder that holds the ISO. It should print OK.',
		'Lancez ceci dans le dossier qui contient l’ISO. La commande doit afficher OK.',
		'Ejecute esto en la carpeta que contiene la ISO. Debe mostrar OK.',
		'Führen Sie das im Ordner mit dem ISO aus. Es sollte OK ausgeben.'
	],
	'downloads.usb.title': ['Write it to a USB stick', 'L’écrire sur une clé USB', 'Grabarla en una memoria USB', 'Auf einen USB-Stick schreiben'],
	'downloads.usb.line': [
		'Replace /dev/sdX with your stick. Everything on it is erased.',
		'Remplacez /dev/sdX par votre clé. Tout ce qu’elle contient est effacé.',
		'Sustituya /dev/sdX por su memoria. Todo su contenido se borra.',
		'Ersetzen Sie /dev/sdX durch Ihren Stick. Alles darauf wird gelöscht.'
	],
	'downloads.usb.gui': [
		'Prefer an app? Impression on Linux or balenaEtcher on macOS and Windows does the same in three clicks.',
		'Vous préférez une app ? Impression sur Linux ou balenaEtcher sur macOS et Windows fait la même chose en trois clics.',
		'¿Prefiere una app? Impression en Linux o balenaEtcher en macOS y Windows hace lo mismo en tres clics.',
		'Lieber eine App? Impression unter Linux oder balenaEtcher unter macOS und Windows erledigt das in drei Klicks.'
	],
	'downloads.next': ['Installation guide', 'Guide d’installation', 'Guía de instalación', 'Installationsanleitung'],
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
