# Update

Open Settings with **Mod+Comma**, then Updates.

1. **Check for updates** looks for a newer version of Flux.
2. **Update now** downloads and installs it. You can close Settings: the update keeps going.
3. When Settings says **Restart to finish**, restart the computer.

Parts of Flux are built on your computer during an update, so it can take a while. Keep the computer plugged in.

**Update automatically** does this once a week, while the computer is plugged in. It is off until you turn it on.

Each update keeps the version before it. If something breaks, [roll back](/docs/rollback).

## From a terminal

```sh
pkexec flux-config update
```

## What an update does

Your machine's configuration is a flake in `/etc/flux`. Its `flux` input follows the `stable` branch of Flux, which only moves to tested releases. An update moves that input to the latest release, records it in `flake.lock`, builds the new system and switches to it.

`/etc/flux` belongs to root. Plain `nix flake update` there fails with a permission error: use the command above.
