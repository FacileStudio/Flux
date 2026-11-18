# Changelog

Every notable change to Flux is listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and Flux uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- The boot screen is black with the Flux mark in the middle, and no start-up messages show before it. AMD and Intel graphics drivers load at the start of boot, so the splash and the login screen use the screen's own resolution.
- The app launcher stays ready in the background and opens at once. GTK apps no longer scan every icon folder at start.
- Arrows and chevrons use the Material Design Icons set, as in muse.
- The top bar keeps a 4 px inset on every side, and every item on it is 32 px tall.
- Switch knobs are round.
- Users and Network use filled buttons for their actions, like the other pages.
- The wallpaper choice lives only in Settings, Appearance. The sidebar no longer has a picker.

### Fixed

- The installer's Details button no longer jumps when it opens or closes, and the install log follows its newest line. The update log in Settings does the same.

## [0.5.0] - 2026-11-17

A polish release after a full review of the shell, Settings, the installer, the system and kit.

### Added

- The installer asks what to install. Facile tools and everyday apps are on, and turning them off gives a minimal desktop. Gaming (Steam with Proton, GameMode, Gamescope, MangoHud and the ntsync kernel module) is off until you turn it on. The NVIDIA driver is offered, and preselected, only on GTX 16, RTX 20 and newer cards.
- Settings has a Software section, so those choices can change after the install.
- An Apps button and a close-window button on the bar, for people who use the mouse.
- A world map on the time zone step: click near your city, or search the list.
- The computer name sits next to the username in the installer. It follows the username until you edit it.
- Scroll direction, natural or traditional, for the touchpad and the mouse in Settings.
- A low-battery warning, and the battery level on the bar.
- A "Before you switch" page in the docs: what will not run on Linux, and what Flux does to your disk.
- The installer keeps the Wi-Fi you joined in the live session.

### Changed

- `Super+L` locks the screen and `Alt+F4` closes the window. Moving focus with H, J, K, L is now `Super+Alt` plus the letter. `Super+Space` and `Super+D` open the same launcher.
- The launcher lists every app, scrolls with the keyboard, and finds apps by their translated name and keywords.
- One design for buttons, fields, switches and lists across the installer, Settings, the lock screen and the login screen. Off switches are visible in dark mode.
- After a hard power-off, the boot menu shows by itself so you can pick an earlier version. Otherwise hold Esc or Shift while starting.
- Gaming no longer opens Steam's network ports by default.
- kit no longer tries to install ardoise, boite and gare, which need tools Flux does not ship. Its first sync waits for the network.
- Docs for installing, updating and going back describe the graphical screens.

### Fixed

- Installs failed at the last step with "--disk is required": leaving the disk screen cleared the chosen disk. Install now stays disabled until every choice is made.
- A failed update left the system pinned to the version that failed. Going back to an earlier version now stays in place when you change a setting.
- A change in Settings could switch off the NVIDIA driver or other options set by hand, and removed groups such as docker from accounts.
- The lock screen kept only one typed character, and dropped keys typed while it checked a password. Its password field is redesigned.
- The live USB session no longer locks itself, sleeps or suspends on lid close during an install.
- A wrong password in Settings no longer shows "This incident has been reported". Errors are plain sentences, in English and French.
- The time zone step fits small laptop screens. A legacy BIOS start and disks that are too small are caught at the start of the installer.
- The network indicator no longer makes Wi-Fi rescan every 30 seconds.
- The NVIDIA driver option could not build: Flux now allows exactly the unfree packages that Steam and the NVIDIA driver need.
- New accounts get their Documents, Downloads and Pictures folders.

## [0.4.0] - 2026-11-17

### Changed

- kit now installs the Facile tools. Flux ships `kit` and `facile` (for `facile login`) and seeds `/etc/kit/kit.yml` with every Facile CLI. `kit sync` installs them in each account at login and checks for updates every hour, so you can remove any of them with `kit del`.
- `flux.facile.enable` and `flux.facile.exclude` are now `flux.kit.enable` and `flux.kit.exclude`. The old names still work and print a rename warning. An excluded tool is left out of the seed.
- Flatpak is on by default on the desktop profile, with the system Flathub remote.
- `~/.local/state/kit/nix-profile/bin` is on `PATH`, so Nix packages added with `kit add` run.

## [0.3.2] - 2026-11-17

### Changed

- Folders in Files use Solar's bold-duotone folder, drawn lighter in dark mode and darker in light mode. The Files sidebar uses the matching icons.

## [0.3.1] - 2026-11-17

The first published release of the 0.3 changes below. The 0.3.0 tag was never published: its boot test could not see the new dark splash.

### Fixed

- The boot test detects a dark splash screen after a black loader.
- A flaky Settings backend test that sometimes saw its own lock still held.

## [0.3.0] - 2026-11-17 [not published]

### Fixed

- The installer's buttons work again. In 0.2.0, Install Flux, Continue and Back did nothing, and the account fields never enabled Continue.
- Terminal, Settings and every other window open at full width instead of half the screen. Super + R still cycles half, two thirds and full.

### Changed

- Flux starts in dark mode, from the boot menu and splash screen to the desktop. Light mode stays one click away in Settings.
- New default wallpaper, a concrete maze courtyard, with five new wallpapers beside it.
- Solar is the Flux icon theme: settings, the installer and the bar use its glyphs, with Adwaita behind it for everything else.
- The installer shows which step you are on, icons on its choices and disks, and its own icon in the launcher.

## [0.2.0] - 2026-11-17

Flux now looks and works like its own system, from the boot menu to the desktop. Still an open beta: tested end to end in virtual machines, not yet on a wide range of real hardware.

### Added

- Graphical installer in muse: try or install, language and keyboard, disk with an erase confirmation, account, time zone and progress. It refuses the USB stick it runs from and checks the system before touching the disk.
- Settings app with Network, Bluetooth, Sound, Displays, Keyboard and mouse, Appearance, Power, System, Users, Updates and About.
- Our own lock screen, idle and lid rules, power modes and night light.
- Muse boot menu, Flux splash screen, quiet boot, and a muse login screen over the wallpaper.
- Default apps: Firefox, Files, image, PDF and video viewers, archive manager, text editor, calculator, clipboard history.
- Printing, Bluetooth, NetworkManager, polkit prompts, keyring, automatic USB mounting, firmware updates, zram and weekly store cleanup.
- Shell, editor, tmux and git defaults: a two-line zsh prompt, fzf and zoxide, Neovim with telescope, treesitter and gitsigns.
- Keys for volume, brightness, media, screenshots, lock, workspaces 5 to 10 and Settings (Super + comma).

### Changed

- Installs are pinned to the release on the ISO and work without a network. Updates follow the `stable` branch.
- Your password is never written to `/etc/flux` or the Nix store.
- The bar, launcher, sidebar and exit menu now work under niri, on every monitor.
- Flux runs in virtual machines without 3D acceleration.

### Fixed

- Many shell fixes: workspaces, volume, the launcher finding apps, the sidebar toggle, notifications urgency.

## [0.1.1] - 2026-11-16

### Security

- The live ISO no longer runs an SSH server. In 0.1.0 it accepted the known login `flux` / `flux` with passwordless sudo, so anyone on the same network could take over a machine running the live session. If you booted the 0.1.0 ISO on a network you do not trust, treat that session as exposed. Installed systems were not affected.

### Fixed

- Installed systems now include redistributable firmware, so Wi-Fi, Bluetooth and AMD graphics work after the first reboot.
- The `facile` CLI is preinstalled, and binaries it installs run thanks to nix-ld.

## [0.1.0] - 2026-11-16

First open beta. Expect rough edges and breaking changes before 1.0.

### Added

- One flake that exports the Flux NixOS module, for `x86_64-linux` and `aarch64-linux`.
- `flux.profile`: `desktop` for the full graphical system, `headless` for servers and VMs.
- Desktop: niri, Ghostty, greetd with tuigreet, PipeWire, and the Inter font.
- Flux shell in GTK 4: statusbar, sidebar, launcher and exit menu, started by niri.
- muse colors applied to the shell, GTK apps, Ghostty and niri through `flux-theme`.
- Light and dark schemes, switched live from the sidebar or with `flux-scheme light|dark|toggle`.
- Wallpapers with a picker in the sidebar and `flux-wallpaper set|list`. The default is a monstera photo by Gilles Lambert (Unsplash License).
- Every public Facile CLI preinstalled, pinned to its latest release, in `facile-tools`.
- zsh by default, with vim, Neovim and vis.
- Dev basics: git, mise, bun, ripgrep, fd, fzf and jq.
- Live installer ISO with `flux-install`, which writes your machine flake to `/etc/flux`.
- Headless qcow2 image for Boite dev VMs, with Go, Rust, Python and Node, published with each release.
- The flux.facile.studio site: install docs, downloads and the open beta announcement.
- `nix run .#vm` to try the desktop in QEMU.
- A machine flake template: `nix flake init -t github:FacileStudio/Flux`.
- Release ISOs for both systems, with a `SHA256SUMS` file.

[Unreleased]: https://github.com/FacileStudio/Flux/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/FacileStudio/Flux/releases/tag/v0.2.0
[0.1.1]: https://github.com/FacileStudio/Flux/releases/tag/v0.1.1
[0.1.0]: https://github.com/FacileStudio/Flux/releases/tag/v0.1.0
