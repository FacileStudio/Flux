# Descargar y comprobar

Descargue la ISO de su procesador en la [página de descargas](/downloads):

- `flux-0.2.0-x86_64-linux.iso` para Intel y AMD
- `flux-0.2.0-aarch64-linux.iso` para ARM64 UEFI (VM y servidores)

Después compruebe que el archivo está intacto. En la carpeta que contiene la ISO:

```sh
curl -LO https://flux.facile.studio/releases/v0.2.0/SHA256SUMS
sha256sum --check --ignore-missing SHA256SUMS
```

El comando debe mostrar `OK`. Si no, descargue la ISO de nuevo.
