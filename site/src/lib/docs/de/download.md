# Herunterladen und prüfen

Laden Sie das ISO für Ihren Prozessor auf der [Download-Seite](/downloads) herunter:

- `flux-0.1.0-x86_64-linux.iso` für Intel und AMD
- `flux-0.1.0-aarch64-linux.iso` für ARM64

Prüfen Sie dann, ob die Datei unversehrt ist. Im Ordner mit dem ISO:

```sh
curl -LO https://flux.facile.studio/releases/v0.1.0/SHA256SUMS
sha256sum --check --ignore-missing SHA256SUMS
```

Der Befehl sollte `OK` ausgeben. Sonst laden Sie das ISO erneut herunter.
