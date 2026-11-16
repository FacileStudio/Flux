# Flux site

The public site of Flux: the landing page, `/downloads` and the install docs under `/docs`.
SvelteKit with Svelte 5, Tailwind v4, muse and mise-en-scene, prerendered to static files and
served by nginx.

## Develop

```sh
bun install
bun run dev
bun run check
bun run build
```

## Where things live

| Path | What |
|---|---|
| `src/lib/release.ts` | the release version and every ISO, checksum and command URL |
| `src/lib/messages/` | page copy, one row per string in the order en, fr, es, de |
| `src/lib/docs/<locale>/<slug>.md` | the docs, one Markdown file per page and locale |
| `src/lib/docs/index.ts` | the docs page order |
| `src/lib/components/FluxDesktop.svelte` | the hero demo |
| `src/app.css` | `--flux-wallpaper`, the demo desktop background |
| `static/flux-mark.svg` | the Flux mark, monochrome (`currentColor`) |

To publish a new release, change `VERSION` in `src/lib/release.ts`. The docs repeat the version
in their download and USB commands; update those lines too.

To show a real wallpaper in the hero demo, set `--flux-wallpaper` in `src/app.css` to
`url('/wallpaper.jpg') center / cover` and add the image to `static/`. Only use an image the
project holds a licence for.

## Deploy

`docker compose up -d --build` builds the site and serves it on port 80 of the `flux-site`
container, on the external `dokploy-network`. The domain is set in Dokploy.

## Credits

- The Nix snowflake comes from https://github.com/NixOS/nixos-artwork
  (`logo/nix-snowflake-white.svg`), flattened to one monochrome path. It is licensed under
  CC BY 4.0: https://creativecommons.org/licenses/by/4.0/
- App marks and feature icons are from Solar by 480 Design, CC BY 4.0.
