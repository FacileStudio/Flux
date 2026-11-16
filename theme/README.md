# Theme

`tokens.css` is a verbatim copy of `src/lib/styles/tokens.css` from muse
(https://github.com/FacileStudio/muse). It is the only design source for Flux.

Vendored from muse commit `266a6f8cb14107577bbc6365969927e755be2897`.

## Bump muse

```sh
cp ../muse/src/lib/styles/tokens.css theme/tokens.css
```

Then update the commit above. Nothing else changes: `pkgs/flux-theme/generate.py` reads every
`--color-fc-*` token at build time, so a new token reaches `palette.css` on the next build.

## What the build produces

`pkgs.flux-theme` runs the generator and installs:

| Path | Used by |
|---|---|
| `share/flux/palette.css`, `palette-light.css`, `palette-dark.css` | the shell, through `/etc/flux/palette.css` |
| `share/flux/gtk-4.0/gtk.css` | libadwaita apps, linked to `~/.config/gtk-4.0/gtk.css` |
| `share/themes/Flux`, `share/themes/Flux-dark` | GTK 3 apps, as the `gtk-theme` |
| `share/flux/ghostty/themes/flux-{light,dark}` | Ghostty, through `/etc/flux/ghostty/config` |
| `share/flux/fuzzel/{light,dark}.ini` | fuzzel, through `/etc/xdg/fuzzel/fuzzel.ini` |

OKLCH values become sRGB hex because GTK CSS does not parse `oklch()`. Colors outside sRGB keep
their lightness and hue and lose chroma until they fit. Translucent tokens (`fc-scrim`, dark
`fc-border`) become `rgba()`. The build fails if a token is not `oklch()` or an output is empty.

`modules/niri.kdl` carries the light `fc-ring` (`#484848`) and `fc-border` (`#dedede`) as literal
focus-ring colors, since niri does not read GTK CSS.
