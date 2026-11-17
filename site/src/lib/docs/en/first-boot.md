# First boot

Sign in with the password you chose in the installer. Flux opens on an empty desktop with a bar at the top.

**Mod** is the Super key: the Windows key on most keyboards, Cmd on a Mac keyboard.

## Find your way

- **Open an app:** press **Mod+Space**, type a few letters, press Enter. The bar also has a button for it.
- **Close a window:** **Mod+Q** or **Alt+F4**. The bar also has a button for it.
- **Move between windows:** windows sit side by side in a row that scrolls. **Mod+Left** and **Mod+Right** go from one to the next. **Alt+Tab** goes back to the previous one.
- **See everything that is open:** **Mod+Tab**.
- **Lock the screen:** **Mod+L**.
- **Sign out, restart or shut down:** **Mod+Escape**.
- **See the main shortcuts:** **Mod+Shift+/**. The full list is on the [shortcuts](/docs/shortcuts) page.

## Settings

Press **Mod+Comma**. Settings covers Wi-Fi, Bluetooth, sound, displays, appearance, power, users and updates. The software choices from the installer can be changed there too.

To change your password, open Users, then **Change password**.

## Get more apps

Firefox, Files and the viewers are already installed. More apps come from [flathub.org](https://flathub.org). Flux has no app store yet. Find the app on Flathub, copy its ID, then run in the terminal (**Mod+Return**):

```sh
kit add flathub:org.gimp.GIMP
```

The app then shows up in the launcher. `kit del flathub:org.gimp.GIMP` removes it, `kit list` shows what you added, `kit update` updates it.

Two things to know:

- Do not `kit add` an app Flux already ships, such as Firefox. You would get a second copy with its own profile.
- kit installs for your account only. `flatpak install` installs for every account, and kit does not list those apps.

## Facile tools

If you kept **Facile tools** in the installer, kit downloads the Facile command line tools into your account after your first sign-in, once the computer is online. It takes a few minutes. Then run `sablier`, `nuage` or `casier` in the terminal, and sign in with `facile login`.

- `systemctl --user status kit-sync` says whether the download worked. `kit sync` runs it again.
- kit checks every hour that nothing is missing. It does not update the tools by itself: run `kit update`.
- Ardoise, Boîte and Gare are not in the default list, because they need programs Flux does not ship. Install those programs, then `kit add ardoise`.
- Your first `kit add` or `kit del` gives your account its own list, in `~/.config/kit/kit.yml`. From then on, a tool that a Flux update adds to the default list no longer arrives by itself: add it with `kit add`.
