/** The docs pages in reading order. Each one is `src/lib/docs/<locale>/<slug>.md`, starting with a `# Title` line. */
export const SLUGS = [
	'requirements',
	'download',
	'usb',
	'live',
	'install',
	'first-boot',
	'update',
	'rollback',
	'nixos',
	'vm',
	'boite',
	'shortcuts',
	'bugs'
] as const;

export type Slug = (typeof SLUGS)[number];

/** The pages about Nix itself, which show the Nix mark next to their title. */
export const NIX_PAGES: readonly Slug[] = ['update', 'rollback', 'nixos', 'vm'];
