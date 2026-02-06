use anyhow::{Context, Result};
use std::path::Path;

const CANVAS_SIZE: u32 = 64;

/// Load an image file and return raw RGB bytes (64x64 = 12,288 bytes).
pub fn load_and_prepare(path: &Path) -> Result<Vec<u8>> {
    let img = image::open(path)
        .with_context(|| format!("Failed to open image: {}", path.display()))?;

    let img = if img.width() != CANVAS_SIZE || img.height() != CANVAS_SIZE {
        eprintln!(
            "Resizing {}x{} -> {}x{}",
            img.width(),
            img.height(),
            CANVAS_SIZE,
            CANVAS_SIZE
        );
        img.resize_exact(
            CANVAS_SIZE,
            CANVAS_SIZE,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        img
    };

    let rgb = img.to_rgb8();
    Ok(rgb.into_raw())
}
