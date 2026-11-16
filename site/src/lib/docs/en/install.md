# Install Flux

In the live session, open a terminal with **Mod+Return** and run:

```sh
sudo flux-install
```

The installer:

1. lists the machine's disks
2. asks which one to use, then asks you to **type its name again** to confirm
3. **erases that disk** and installs Flux on it
4. writes your machine's config to `/etc/flux`

`/etc/flux` is a flake that imports Flux. That config is yours to read and edit.

Remove the USB stick and restart when the install is done.
