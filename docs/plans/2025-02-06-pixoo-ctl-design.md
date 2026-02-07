# pixoo-ctl Design

A Rust CLI tool to push images to named Divoom Pixoo 64 devices, designed to be scriptable via launchd for scheduled album art display.

## Devices

| Name | IP |
|------|-----|
| stage-left | 192.168.88.43 |
| stage-right | 192.168.88.170 |

## CLI Commands

```
,pixoo-ctl push-image --device stage-left ./show-art.png
,pixoo-ctl push-image --device all ./on-air.png
,pixoo-ctl get-settings --device stage-left
```

### push-image

Core command. Loads a local image file, resizes to 64x64 if needed, and sends it to the target device.

1. Load file via `image` crate (PNG, JPEG, GIF, BMP supported)
2. Resize to 64x64 using Lanczos3 filter if dimensions don't match
3. Convert to RGB byte array
4. Send via `divoom` crate's animation frame API

### get-settings

Query current device state (brightness, channel, etc.). Useful for debugging and scripting.

## Configuration

File: `~/.config/pixoo-ctl/config.toml`

```toml
[devices.stage-left]
ip = "192.168.88.43"

[devices.stage-right]
ip = "192.168.88.170"
```

The `--device` flag accepts any key from `[devices.*]`, or `all` to target every device.

## Project Structure

```
src/
├── main.rs     # Entry point, clap CLI parsing
├── config.rs   # TOML config loading
├── device.rs   # Device communication (wraps divoom crate)
└── image.rs    # Image loading & 64x64 validation/resize
```

Target: under 400 lines total across all files.

## Dependencies

| Crate | Purpose |
|-------|---------|
| divoom | Pixoo device communication |
| clap (derive) | CLI argument parsing |
| serde / toml | Config file loading |
| image | Image loading and resize |
| anyhow | Error handling |

## Error Handling

- Device name not in config: list available devices
- Device unreachable: timeout with IP check suggestion
- Image file not found: standard file error
- Unsupported format: list supported formats

## Escape Hatch

If the `divoom` crate (v0.1.42, last updated 2022) is broken or insufficient, replace with direct `reqwest` POST calls to `http://<ip>/post` with JSON payloads. The Pixoo local API is simple enough that this is low risk.

## Future (not in v1)

- Gallery/channel management
- Text display
- URL-based image fetching
- Album art scheduling (handled by launchd, not this tool)
