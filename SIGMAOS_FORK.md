# SigmaOS Fork of Omarchy

This repository is a fork of [omacom/omarchy](https://github.com/omacom/omarchy)
(MIT License). Original copyright and license notices are retained per the
MIT terms — see `LICENSE`.

## Fork policy

- SigmaOS identity: user-facing branding, docs, and defaults are normalized
  to SigmaOS. Upstream names remain only where required for compatibility
  (package names, upstream file paths, license attributions).
- Divergence is intentional: compositor decision (Hyprland ships), theme
  system (sigmaos-theme CLI), package baseline (pacman/ALPM), recovery
  (generation rollback), and Rust core components are SigmaOS additions.
- Upstream syncs: pull upstream periodically, resolve conflicts preserving
  SigmaOS identity and decisions documented in the main SigmaOS repo
  (`docs/COMPOSITOR_DECISION.md`, `docs/THEME_DECISION.md`,
  `docs/COMPETITIVE_STRATEGY.md`).

## Provenance

- Upstream: omacom/omarchy @ fee089f (shallow clone base)
- License: MIT (see LICENSE)
