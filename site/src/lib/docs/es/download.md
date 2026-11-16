# Descargar y comprobar

Descargue la ISO de su procesador en la [página de descargas](/downloads):

- `flux-0.1.0-x86_64-linux.iso` para Intel y AMD
- `flux-0.1.0-aarch64-linux.iso` para ARM64

Después compruebe que el archivo está intacto. En la carpeta que contiene la ISO:

```sh
curl -LO https://github.com/FacileStudio/Flux/releases/download/v0.1.0/SHA256SUMS
sha256sum --check --ignore-missing SHA256SUMS
```

El comando debe mostrar `OK`. Si no, descargue la ISO de nuevo.
