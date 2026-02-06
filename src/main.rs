mod config;
mod device;
mod image;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "pixoo-ctl", about = "Control Divoom Pixoo 64 devices")]
struct Cli {
    /// Device name from config, or "all" for every device
    #[arg(short, long)]
    device: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Push a local image to the device (resizes to 64x64 if needed)
    PushImage {
        /// Path to image file (PNG, JPEG, GIF, BMP)
        path: PathBuf,
    },
    /// Show current device settings
    GetSettings,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = config::load()?;
    let devices = config::resolve_devices(&cfg, &cli.device)?;

    match cli.command {
        Command::PushImage { path } => {
            let pixmap = image::load_and_prepare(&path)?;
            for (name, ip) in &devices {
                eprintln!("Pushing image to {} ({})", name, ip);
                device::push_image(ip, &pixmap).await?;
                eprintln!("Done: {}", name);
            }
        }
        Command::GetSettings => {
            for (name, ip) in &devices {
                eprintln!("--- {} ({}) ---", name, ip);
                device::get_settings(ip).await?;
            }
        }
    }

    Ok(())
}
