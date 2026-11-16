# settings-audio

The Sound page of Flux Settings, plus the statusbar quick menu and a volume-step call for the
volume keys.

```rust
settings_audio::page()            // the page: ID "audio", ICON, title()
settings_audio::quick_menu()      // popover content for the statusbar
settings_audio::set_volume_step(5) // Some(new percent); blocking, call off the GTK thread
```

## Backend: the PulseAudio protocol through libpulse

The page talks to `pipewire-pulse` with `libpulse-binding`, not with `pactl` subprocesses.

- **Live updates.** libpulse subscription events arrive on a socket the moment a device,
  stream, card or default changes. `pactl subscribe` would mean parsing a text stream and then
  running one `pactl --format=json list` per event.
- **Meters.** Peak-detect record streams (25 Hz, one float per fragment) give the live level
  meters for almost no CPU. There is no `pactl` equivalent; it would need a `parec` process
  and our own peak math.
- **Test sound.** The left-then-right tone is generated in memory and played on the chosen sink
  through a libpulse stream. No sound file and no `paplay`.
- **Volume semantics.** Channel balance is kept with `pa_cvolume_scale`, the same call
  pavucontrol uses.

The cost is one link dependency, `libpulseaudio`, which every NixOS desktop with PipeWire
already ships (the pulse client library is how most apps reach `pipewire-pulse`).

The backend sits behind the `Backend` trait (`src/backend.rs`), so a `pactl` or native
PipeWire backend can replace it without touching the view.

## Layout

| Path | What |
|---|---|
| `src/model.rs` | pure: device names and kinds, port and profile choices, volume math (cubic), app grouping, test tone, options |
| `src/backend/pulse.rs` | one thread owning the libpulse threaded main loop; commands in, snapshots and levels out |
| `src/backend/preview.rs` | sample data from `fixtures/laptop.json`, for design checks |
| `src/hub.rs` | one connection per GTK thread, events dispatched on the main loop |
| `src/view*` | GTK only |
| `src/ui.rs` | local copies of `shell_common::ui` helpers until the settings core lands |

Options ("Allow volume above 100%") live in `~/.config/flux/audio.json`.

## Build

The crate links `libpulse`. The dev shell and `pkgs/flux-shell` need `libpulseaudio` in
`buildInputs`.

## Checks

```sh
cargo test -p settings-audio
FLUX_AUDIO_PREVIEW=1 cargo run -p settings-audio --example preview          # page, sample data
FLUX_AUDIO_PREVIEW=bluetooth cargo run -p settings-audio --example preview  # headset selected
FLUX_AUDIO_PREVIEW=unavailable cargo run -p settings-audio --example preview
FLUX_AUDIO_PREVIEW=1 cargo run -p settings-audio --example preview quick    # quick menu
```

The backend test runs against a private daemon, never the session's:

```sh
export XDG_RUNTIME_DIR=$(mktemp -d)
pulseaudio --daemonize=no --exit-idle-time=-1 -n --disable-shm=yes \
  -L "module-native-protocol-unix auth-anonymous=1 socket=$XDG_RUNTIME_DIR/native" \
  -L "module-null-sink sink_name=a" -L "module-null-sink sink_name=b" \
  -L "module-null-source source_name=mic" &
PULSE_SERVER=unix:$XDG_RUNTIME_DIR/native cargo test -p settings-audio -- --ignored --nocapture
```

It lists devices, switches the default and checks a running `pacat` stream moves with it, sets
device and stream volume, mutes, reads both level meters, plays the test sound, and runs
`set_volume_step`.
