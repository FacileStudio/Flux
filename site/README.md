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
| `src/lib/components/DesktopDemo.svelte` | the hero demo script; `FluxDesktop.svelte` draws the desktop |
| `src/lib/components/desk/` | the faux Flux Settings window, shared by the hero and the tiles |
| `src/lib/components/tiles/` | the Settings, login, installer, boot menu and Boîte demos |
| `src/app.css` | `--flux-wallpaper` and `.flux-pal`, the demo palette that flips between light and dark |
| `static/flux-mark.svg` | the Flux mark, monochrome (`currentColor`) |

To publish a new release, change `VERSION` in `src/lib/release.ts`. The docs repeat the version
in their download and USB commands; update those lines too.

## Deploy

`docker compose up -d --build` builds the site and serves it on port 80 of the `flux-site`
container, on the external `dokploy-network`. The domain is set in Dokploy.

## Credits

- The Nix snowflake comes from https://github.com/NixOS/nixos-artwork
  (`logo/nix-snowflake-white.svg`), flattened to one monochrome path. It is licensed under
  CC BY 4.0: https://creativecommons.org/licenses/by/4.0/
- App marks and feature icons are from Solar by 480 Design, CC BY 4.0.
- `static/monstera.jpg`, the default Flux wallpaper shown in the demos, is a photo by Gilles Lambert,
  https://unsplash.com/photos/mSK5nNsAsLY, under the Unsplash License: https://unsplash.com/license.
  It is resized from `pkgs/flux-wallpapers/images/monstera.jpg`.
