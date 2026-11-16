# Download and verify

Download the ISO for your processor from the [downloads page](/downloads):

- `flux-0.1.1-x86_64-linux.iso` for Intel and AMD
- `flux-0.1.1-aarch64-linux.iso` for ARM64 UEFI machines: VMs (UTM, Parallels, QEMU), ARM servers and EDK2 boards

Then check the file is intact. In the folder that holds the ISO:

```sh
curl -LO https://flux.facile.studio/releases/v0.1.1/SHA256SUMS
sha256sum --check --ignore-missing SHA256SUMS
```

The command should print `OK`. If it does not, download the ISO again.
