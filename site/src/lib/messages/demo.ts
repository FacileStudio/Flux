import type { Row } from '@facile/muse';

/** The words drawn inside the hero desktop, in the order en, fr, es, de. Commands stay as they are. */
export const demo = {
	'demo.project': ['flux-site', 'site-flux', 'sitio-flux', 'flux-seite'],
	'demo.synced': ['in sync', 'synchronisé', 'sincronizado', 'synchron'],
	'demo.started': ['timer started', 'minuteur lancé', 'temporizador iniciado', 'Timer gestartet'],
	'demo.search': ['Search apps', 'Chercher une app', 'Buscar una app', 'App suchen'],
	'demo.app.sablier': ['Track your time', 'Suivre son temps', 'Medir su tiempo', 'Zeit erfassen'],
	'demo.app.nuage': ['Your files', 'Vos fichiers', 'Sus archivos', 'Ihre Dateien'],
	'demo.running': ['Running', 'En cours', 'En curso', 'Läuft'],
	'demo.today': ['Today', 'Aujourd’hui', 'Hoy', 'Heute']
} satisfies Record<string, Row>;
