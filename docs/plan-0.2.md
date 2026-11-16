# Flux 0.2 plan: our own boot, login and installer

Goal: from power-on to desktop, nothing says NixOS or looks generic. Every screen is Flux and muse.

## Rules for every stream

- No `nix build` of images, ISOs or Rust closures on ruche. Evaluate locally (`nix eval ... drvPath`);
  GitHub CI builds. Rust may use `cargo check` with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0` and a
  `CARGO_TARGET_DIR` deleted at the end, only while `df -h /` shows more than 20 GB free.
- muse rules: colors only from the `fc_*` palette (`/etc/flux/palette-<scheme>.css`, GTK) or the token
  values in `theme/tokens.css` (images), no borders anywhere, Inter for UI text, light and dark.
- The Flux mark is the Solar `filters-bold-duotone` icon: `site/static/flux-mark.svg`.
- No inline code comments. Conventional Commits, author saravenpi, no Co-Authored-By. Do not push.

## Streams

| Stream | Owns |
|---|---|
| boot | `modules/boot.nix`, `pkgs/flux-grub-theme/`, `pkgs/flux-plymouth/`, boot lines in `hosts/iso/`, the loader lines of `template/configuration.nix` |
| greeter | `modules/greeter.nix`, the `services.greetd` block of `modules/desktop.nix` |
| installer | `shell/crates/installer/`, `pkgs/flux-install/`, `pkgs/flux-shell/` (add the binary), the autostart lines of `hosts/iso/`, the rest of `template/` |
| boot-test | `.github/workflows/boot-test.yml`, `tests/` |

## Contracts

Branding (boot stream): `system.nixos.distroName = "Flux"`, `distroId = "flux"`, `vendorName = "Facile Studio"`,
so os-release, boot entries and `nixos-version` callers say Flux. ISO volume ID `FLUX_<version>`.

Boot loader: installed systems use GRUB (EFI) with the Flux theme, `timeoutStyle = "hidden"` and a
short timeout, so the menu only shows when the user holds Esc or Shift. Rollback still works from that
menu. The live ISO shows the themed GRUB menu (entries: "Try or install Flux", "Safe graphics").
Plymouth shows the Flux mark on the muse page color with a small spinner; boot is quiet
(`quiet splash`, `boot.consoleLogLevel = 0`, `boot.initrd.verbose = false`, systemd initrd).

Login (greeter stream): ReGreet under greetd, styled with the muse palette, the current wallpaper as
background, the Flux mark, Inter. The ISO keeps autologin.

Installer backend (installer stream): `flux-install` keeps its interactive mode and gains a
non-interactive one that the GUI drives:

```
flux-install --disk /dev/vda --hostname flux --full-name "Ada" --user ada \
  --password-hash '<sha-512 crypt>' --timezone Europe/Paris --keymap fr --locale fr_FR.UTF-8 \
  --yes --progress json
```

With `--progress json` it prints one JSON object per line on stdout:
`{"step":"partition|format|config|install|bootloader|done","percent":0-100,"message":"..."}`
and `{"step":"error","message":"..."}` before a non-zero exit. The template gains timezone, keymap and
locale placeholders. `--yes` replaces the typed disk confirmation; the GUI asks for it instead.

Installer GUI: `flux-installer`, a GTK 4 binary in the shell workspace, reusing shell-common for the
palette and scheme. On the live ISO it opens full screen at login with two choices: Try Flux, Install
Flux. Install flow: language and keyboard, disk (whole disk only, shows model and size, red confirm
step that names the disk), account (full name, username derived from it, password twice), time zone,
summary, progress (friendly step names from the JSON, never raw logs unless "Details" is opened), done
(restart now). It runs the backend through `sudo -n flux-install ... --progress json`.

Boot test (CI): boots the x86_64 ISO in QEMU with KVM on a GitHub runner, takes screenshots of the GRUB
menu, the Plymouth splash and the live desktop with the installer open, runs a non-interactive install
to a blank virtual disk, reboots into it, and screenshots the greeter. Screenshots are workflow
artifacts so a human (or an agent) can look at every screen.
