mod config;
mod device;
mod image;
mod schedule;

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "pixoo-ctl", about = "Control Divoom Pixoo 64 devices")]
struct Cli {
    /// Device name from config, or "all" for every device
    #[arg(short, long)]
    device: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Push a local image to the device (resizes to 64x64 if needed)
    PushImage {
        /// Path to image file (PNG, JPEG, GIF, BMP)
        path: PathBuf,
        /// Hold the schedule (prevent automatic updates until `resume`)
        #[arg(long)]
        hold: bool,
    },
    /// Show current device settings
    GetSettings,
    /// Push images based on the current day/time schedule
    UpdateSchedule,
    /// Resume the schedule after a hold
    Resume,
}

fn require_device(device: &Option<String>, command: &str) -> Result<String> {
    device
        .clone()
        .ok_or_else(|| anyhow!("--device is required for {}", command))
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = config::load()?;

    match cli.command {
        Command::PushImage { path, hold } => {
            let device = require_device(&cli.device, "push-image")?;
            let devices = config::resolve_devices(&cfg, &device)?;
            let rgb_data = image::load_and_prepare(&path)?;
            for (name, ip) in &devices {
                eprintln!("Pushing image to {} ({})", name, ip);
                device::push_image(ip, &rgb_data).await?;
                eprintln!("Done: {}", name);
            }
            if hold {
                schedule::set_hold()?;
            }
        }
        Command::GetSettings => {
            let device = require_device(&cli.device, "get-settings")?;
            let devices = config::resolve_devices(&cfg, &device)?;
            for (name, ip) in &devices {
                eprintln!("--- {} ({}) ---", name, ip);
                device::get_settings(ip).await?;
            }
        }
        Command::UpdateSchedule => {
            schedule::update(&cfg).await?;
        }
        Command::Resume => {
            schedule::clear_hold()?;
        }
    }

    Ok(())
}
