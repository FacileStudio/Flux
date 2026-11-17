# Before you switch

Flux replaces the system on the disk you give it. Read this page before you erase anything.

## What will not run

- **Adobe apps.** Photoshop, Lightroom, Illustrator, InDesign and Premiere Pro have no Linux version.
- **Microsoft Office desktop apps.** Word, Excel, PowerPoint and Outlook do not install on Flux. Office on the web works in Firefox.
- **Games with kernel anti-cheat.** Some online games refuse to start on Linux. Valorant and Fortnite are two of them at the time of writing. Look up every game you play on [areweanticheatyet.com](https://areweanticheatyet.com/).
- **Windows programs in general.** Flux does not run `.exe` files out of the box.

Many Windows games do run through Steam, which the installer can add. Check each one on [protondb.com](https://www.protondb.com/).

If you need one of the programs above for work or school, keep a computer that runs it.

## Flux takes the whole disk

The installer erases the disk you pick: files, Windows and every other system on it. It cannot put Flux beside Windows on the same disk.

- Back up your files to another disk or to the cloud first.
- To keep Windows, install Flux on a second disk. You then choose the system in the boot menu of the computer (often F12, F10 or Esc at startup).

## Check the computer

- It must start in **UEFI** mode, with **Secure Boot turned off** in the firmware settings. Flux does not start with Secure Boot on.
- The other needs are on the [requirements](/docs/requirements) page.

## Try it first

The USB stick you install from also runs Flux without installing it.

1. [Create the USB stick](/docs/usb) and [start from it](/docs/live).
2. Choose **Try Flux**. Nothing on the computer changes.
3. Check the screen, Wi-Fi, sound, Bluetooth, the touchpad and the keyboard.

The live session forgets everything when you shut down. If something does not work there, it will not work after the install either: [tell us](/docs/bugs) before you switch.

## What you get

Firefox, a file manager, a text editor, a calculator and viewers for images, PDFs and videos. More apps come from Flathub. Flux has no app store yet, so installing one takes a command in the terminal: see [First boot](/docs/first-boot).

Flux is in open beta. Expect rough edges.
