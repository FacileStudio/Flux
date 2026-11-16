# Roll back

Each update keeps the previous version of the system, called a generation.

## From the boot menu

Restart, then pick the previous generation in the menu. Flux starts exactly as it was before the update.

## From the terminal

```sh
sudo nixos-rebuild switch --rollback
```

The system goes back to the previous generation without a restart.

Your personal files stay where they are: only the system changes version.
