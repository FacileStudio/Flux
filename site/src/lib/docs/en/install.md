# Install Flux

The installer opens by itself when the live session starts. If you closed it, press **Mod+Space**, type `install` and press Enter.

Read [Before you switch](/docs/before-you-switch) first: the install erases a whole disk.

## The steps

1. **Welcome.** Choose **Install Flux**. **Try Flux** closes the installer and changes nothing.
2. **Language and keyboard.** Pick both, then type in the test field to check the layout.
3. **Disk.** Pick the disk Flux goes on. Flux takes the whole disk, and the installer asks you to confirm that it will be erased.
4. **Your account.** Your name, a username, a password, and a name for the computer.
5. **Time zone.** Click the map or search for a city.
6. **What to install.** Facile tools and everyday apps are on. Gaming is off until you turn it on. The NVIDIA driver is offered only when the computer has an NVIDIA card.
7. **Ready to install.** Check your choices, then choose **Install Flux**.

The disk is erased when you choose **Install Flux** on the last step, not before. The install takes a few minutes. Keep the computer on and plugged in.

When the installer says **Flux is installed**, remove the USB stick and choose **Restart now**.

## Internet

With the choices the installer starts with, Flux is copied from the USB stick and needs no internet. Any other choice downloads what the stick does not carry, so connect first: open Settings with **Mod+Comma**, then Network.

The Facile tools download later, after your first sign-in.

## If it stops

The installer shows what went wrong. **Copy log** puts the details in the clipboard so you can [report the bug](/docs/bugs). Go back and try again once the cause is fixed.

## From a terminal

```sh
sudo flux-install
```

It asks the same questions as text, and makes you type the disk name twice before it erases anything. `flux-install --help` lists the flags for a scripted install.

The install writes your machine's configuration to `/etc/flux`, a flake that imports Flux. It is yours to read and edit.
