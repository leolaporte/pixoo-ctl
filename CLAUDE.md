# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

Rust CLI tool for pushing images to Divoom Pixoo 64 LED displays over the local HTTP API. No cloud, no SDK — direct JSON POSTs to `http://<device-ip>/post`. ~400 lines total.

## Commands

```bash
cargo build                # Dev build
cargo build --release      # Release build
cargo test                 # Run tests (none currently)
cargo clippy               # Lint
cargo fmt                  # Format
cp target/release/pixoo-ctl ~/.local/bin/,pixoo-ctl   # Install (rename with comma prefix)
```

## Architecture

Four modules, each under 200 lines:

- **main.rs** — CLI entry point using clap derive. Four commands: `push-image`, `get-settings`, `update-schedule`, `resume`. The `-d` flag selects a device by name (or `all`).
- **config.rs** — Loads `~/.config/pixoo-ctl/config.toml`. Resolves device names to IPs. `ScheduleEntry` has hardcoded `stage-left`/`stage-right` fields (not generic).
- **device.rs** — HTTP client via reqwest. `push_image()` always calls `Draw/ResetHttpGifId` before `Draw/SendHttpGif` (fixes a caching bug where the display wouldn't update without resetting the GIF ID first). Images are sent as base64-encoded raw RGB.
- **image.rs** — Loads image files, resizes to 64x64 with Lanczos3, converts to raw RGB bytes (12,288 bytes).
- **schedule.rs** — Finds the most recent schedule entry matching current day/time. Tracks last-pushed images in `state.json` to skip redundant pushes. Hold mechanism via presence/absence of a `hold` file.

### Data flow

```
CLI args → config.rs (load TOML, resolve device) → image.rs (load/resize) → device.rs (HTTP POST to Pixoo)
                                                  ↑
                                        schedule.rs (time-based entry selection + state tracking)
```

## Key Design Decisions

- **`Draw/ResetHttpGifId` before every push**: This is intentional and required. Without it, the Pixoo device caches the previous GIF ID and won't update the display. Don't remove this call.
- **Hardcoded device names in schedule**: `ScheduleEntry` uses `stage-left` and `stage-right` as explicit serde-renamed fields (not a generic map). The schedule system in `schedule.rs:131-134` also hardcodes these names. Generalizing would require changing both.
- **No `divoom` crate**: Design originally planned to use it, but switched to direct reqwest calls for simplicity and control.

## Runtime Files

All under `~/.config/pixoo-ctl/`:
- `config.toml` — device IPs and schedule entries
- `state.json` — auto-managed last-pushed tracking
- `hold` — empty sentinel file; present = schedule paused
