# Create a bootable USB stick

Everything on the stick is erased.

## With an app

- On Linux, **Impression** writes the ISO in three clicks.
- On macOS and Windows, **balenaEtcher** does the same.

Pick the ISO, pick the stick, then start writing.

## From the command line

Find the stick's name with `lsblk`, then replace `/dev/sdX` with it:

```sh
sudo dd if=flux-0.3.1-x86_64-linux.iso of=/dev/sdX bs=4M status=progress oflag=sync
```

Check the name twice: `dd` overwrites the disk you name without asking.
