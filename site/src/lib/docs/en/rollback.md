# Roll back

Each update keeps the version of the system before it. Going back changes the system only: your files and settings stay where they are.

## Flux still starts

Open Settings with **Mod+Comma**, then Updates. Under **Previous versions**, choose **Go back** next to the version you want.

From a terminal:

```sh
pkexec flux-config rollback
```

## Flux does not start, or the screen stays black

1. Hold the power button until the computer turns off.
2. Turn it on again. The boot menu shows by itself, because the last start did not end with a clean shutdown. It waits 30 seconds.
3. Choose **Flux - All configurations**, then the line dated before the update.

Flux starts as it was on that date. This choice lasts for one start. To keep it, open Settings, then Updates, and choose **Go back** next to the same version.

## Open the boot menu yourself

The boot menu is hidden on a normal start. To show it, turn the computer on and tap **Esc** right away, several times. On computers where Esc opens the firmware menu, hold **Shift** instead.

## How long versions stay

The boot menu lists the ten most recent versions. Once a week, Flux deletes the versions older than 30 days and removes them from the menu.
