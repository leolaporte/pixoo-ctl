# pixoo-ctl

A command-line tool to push images to [Divoom Pixoo 64](https://divoom.com/products/pixoo-64) LED displays. Supports multiple named devices, automatic scheduling by day and time, and manual overrides.

Built in Rust. Talks directly to the Pixoo local HTTP API — no cloud, no app, no dependencies on third-party SDKs.

## Install

```bash
cargo install --path .
```

Or build manually:

```bash
cargo build --release
cp target/release/pixoo-ctl ~/.local/bin/
```

## Configuration

Create `~/.config/pixoo-ctl/config.toml`:

```toml
[devices.stage-left]
ip = "192.168.88.43"

[devices.stage-right]
ip = "192.168.88.170"
```

Add as many `[devices.<name>]` entries as you have Pixoo devices.

## Usage

### Push an image

```bash
# Push to one device
pixoo-ctl -d stage-left push-image ./art.png

# Push to all devices
pixoo-ctl -d all push-image ./on-air.png
```

Images are automatically resized to 64x64 using Lanczos3 filtering. Supports PNG, JPEG, GIF, and BMP.

### Query device settings

```bash
pixoo-ctl -d stage-left get-settings
```

### Scheduled updates

Add schedule entries to `config.toml`:

```toml
[[schedule]]
days = ["Mon", "Tue", "Wed", "Thu", "Fri"]
time = "09:00"
stage-left = "/path/to/morning-show.png"
stage-right = "/path/to/logo.png"

[[schedule]]
days = ["Mon", "Tue", "Wed", "Thu", "Fri"]
time = "14:00"
stage-left = "/path/to/afternoon-show.png"
stage-right = "/path/to/logo.png"

[[schedule]]
days = ["Sat", "Sun"]
time = "08:00"
stage-left = "/path/to/weekend.png"
stage-right = "/path/to/logo.png"
```

Run the scheduler:

```bash
pixoo-ctl update-schedule
```

It matches the current day and time, finds the most recent applicable entry, and pushes the images. It tracks what's already displayed and skips re-sending — safe to run frequently.

Day abbreviations: `Mon`, `Tue`, `Wed`, `Thu`, `Fri`, `Sat`, `Sun`.

### Automate with launchd (macOS)

Create `~/Library/LaunchAgents/com.pixoo-ctl.schedule.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.pixoo-ctl.schedule</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Users/YOU/.local/bin/pixoo-ctl</string>
        <string>update-schedule</string>
    </array>
    <key>StartInterval</key>
    <integer>300</integer>
    <key>StandardOutPath</key>
    <string>/Users/YOU/.config/pixoo-ctl/schedule.log</string>
    <key>StandardErrorPath</key>
    <string>/Users/YOU/.config/pixoo-ctl/schedule.log</string>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
```

Load it:

```bash
launchctl load ~/Library/LaunchAgents/com.pixoo-ctl.schedule.plist
```

For Linux, use a systemd timer or cron instead.

### Override the schedule

Push a custom image and pause automatic updates:

```bash
pixoo-ctl -d stage-left push-image ./special.png --hold
```

The scheduler will skip updates while held. Resume when ready:

```bash
pixoo-ctl resume
```

## How it works

The Pixoo 64 exposes a local HTTP API at `http://<device-ip>/post`. This tool sends JSON POST requests to push images as single-frame animations (`Draw/SendHttpGif`). Before each push, it resets the device's GIF ID cache (`Draw/ResetHttpGifId`) to ensure the display always updates.

Images are loaded with the [image](https://crates.io/crates/image) crate, converted to raw RGB bytes (3 bytes per pixel, 64x64 = 12,288 bytes), base64-encoded, and sent in the request payload.

## Files

| Path | Purpose |
|------|---------|
| `~/.config/pixoo-ctl/config.toml` | Device IPs and schedule |
| `~/.config/pixoo-ctl/state.json` | Tracks last-pushed images (auto-managed) |
| `~/.config/pixoo-ctl/hold` | Hold file (present = schedule paused) |
| `~/.config/pixoo-ctl/schedule.log` | Log output from launchd |

## Credits

Written by [Leo Laporte](https://leolaporte.com) and [Claude](https://claude.ai) (Anthropic's Claude Opus 4.6).

This entire project — design, implementation, debugging, and documentation — was pair-programmed in a single Claude Code session. Claude wrote all the code; Leo provided the requirements, tested on real hardware, and caught the GIF ID caching bug that led to the `Draw/ResetHttpGifId` fix.

## License

MIT
