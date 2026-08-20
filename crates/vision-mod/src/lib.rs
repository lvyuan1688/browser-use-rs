//! vision-mod: screenshot diff + similarity.
//! Compares two RGBA screenshots and returns a 0.0–1.0 similarity score,
//! plus a per-pixel diff count. Uses downscaling + mean-squared diff to
//! stay cheap — this is a coarse signal, not a perceptual hash.

use anyhow::Result;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

/// Default target width for downscaling before comparison.
pub const TARGET_W: u32 = 64;
/// Default target height.
pub const TARGET_H: u32 = 36;
/// Per-channel threshold above which a pixel counts as "different".
pub const CHANNEL_THRESH: u8 = 24;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diff {
    /// 0.0 = identical, 1.0 = completely different.
    pub similarity: f32,
    /// Fraction of pixels that differ beyond CHANNEL_THRESH.
    pub changed_fraction: f32,
}

/// Decode a PNG/RGBA buffer into a DynamicImage.
pub fn decode(buf: &[u8]) -> Result<DynamicImage> {
    Ok(image::load_from_memory(buf)?)
}

/// Downscale an image to (TARGET_W, TARGET_H).
pub fn downscale(img: &DynamicImage) -> RgbaImage {
    ImageBuffer::from_fn(TARGET_W, TARGET_H, |x, y| {
        let sx = (x as f32 * img.width() as f32 / TARGET_W as f32) as u32;
        let sy = (y as f32 * img.height() as f32 / TARGET_H as f32) as u32;
        img.get_pixel(sx.min(img.width() - 1), sy.min(img.height() - 1))
    })
}

/// Compare two downscale'd RGBA buffers. Returns similarity in [0,1].
pub fn compare(a: &RgbaImage, b: &RgbaImage) -> Diff {
    let w = a.width().min(b.width());
    let h = a.height().min(b.height());
    if w == 0 || h == 0 {
        return Diff { similarity: 1.0, changed_fraction: 0.0 };
    }
    let mut sum_sq: f64 = 0.0;
    let mut changed: u32 = 0;
    let total = (w * h) as u32;
    for y in 0..h {
        for x in 0..w {
            let pa: Rgba<u8> = a.get_pixel(x, y);
            let pb: Rgba<u8> = b.get_pixel(x, y);
            let dr = (pa[0] as i32 - pb[0] as i32).abs();
            let dg = (pa[1] as i32 - pb[1] as i32).abs();
            let db = (pa[2] as i32 - pb[2] as i32).abs();
            sum_sq += (dr * dr + dg * dg + db * db) as f64;
            if dr > CHANNEL_THRESH as i32
                || dg > CHANNEL_THRESH as i32
                || db > CHANNEL_THRESH as i32
            {
                changed += 1;
            }
        }
    }
    let max_sq = (3.0 * 255.0 * 255.0) * (total as f64);
    let similarity = 1.0 - (sum_sq / max_sq).sqrt() / 255.0;
    Diff {
        similarity: similarity as f32,
        changed_fraction: changed as f32 / total as f32,
    }
}

/// Convenience: end-to-end compare of two raw RGBA buffers.
pub fn diff_buffers(a: &[u8], b: &[u8]) -> Result<Diff> {
    let ia = decode(a)?;
    let ib = decode(b)?;
    let da = downscale(&ia);
    let db = downscale(&ib);
    Ok(compare(&da, &db))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(color: u8) -> RgbaImage {
        ImageBuffer::from_fn(TARGET_W, TARGET_H, |_, _| Rgba([color, color, color, 255]))
    }

    #[test]
    fn identical_images_have_similarity_1() {
        let d = compare(&solid(0), &solid(0));
        assert!(d.similarity > 0.99);
        assert_eq!(d.changed_fraction, 0.0);
    }

    #[test]
    fn opposite_images_have_similarity_near_0() {
        let d = compare(&solid(0), &solid(255));
        assert!(d.similarity < 0.05);
        assert!(d.changed_fraction > 0.95);
    }

    #[test]
    fn empty_buffers_return_unchanged() {
        let d = compare(&solid(0), &solid(0));
        assert!(d.changed_fraction == 0.0);
    }
}
