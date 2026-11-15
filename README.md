# SigmaOS-omarchy

> **Omarchy CLI** — the `omarchy doctor`, `theme`, and `run` interface for SigmaOS.

Inspired by [Omarchy Linux](https://omarchy.com) — a beautiful Arch-based Hyprland distro.

## Quickstart

```bash
cargo run -- doctor       # health diagnostics
cargo run -- theme list   # list available themes
cargo run -- theme set catppuccin-mocha
cargo run -- run          # launch Omarchy persona
```

## Commands

| Command | Description |
|---|---|
| `doctor` | Full system health check: Wayland, Hyprland, audio, network, disk, updates, battery |
| `theme list` | List available Wallust-compatible themes |
| `theme set <name>` | Apply a named colour palette |
| `run` | Launch the Omarchy aesthetic desktop persona |

## Doctor Output Example

```
[ ok ] wayland-session    running under Wayland
[ ok ] hyprland           compositor running
[warn] waybar             panel not running
       fix: omarchy-refresh or restart waybar
[ ok ] audio              PipeWire/PulseAudio running
[ ok ] network            link up
[ ok ] disk               200 GiB free of 500
[ ok ] updates            system up to date

9 passed, 1 warnings, 0 failures
```

## Component Role

This crate is a component of the [SigmaOS](https://github.com/AaryanSinghChauhan09/SigmaOS) ecosystem.  
See the [GitHub Wiki](https://github.com/AaryanSinghChauhan09/SigmaOS/wiki) for full ecosystem documentation.

## License

MIT
