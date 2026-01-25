//! Image comparison utilities

use anyhow::{anyhow, Result};
use image::{GenericImageView, Rgba, RgbaImage};
use std::io::Cursor;

/// Result of comparing two images
pub struct DiffResult {
    /// Similarity score 0.0-1.0 (1.0 = identical)
    pub similarity: f32,

    /// Mean absolute error per channel (0.0-1.0)
    pub mae: f32,

    /// Peak error (max difference in any pixel)
    pub peak_error: f32,

    /// Descriptions of regions with high difference
    pub diff_regions: Vec<String>,

    /// Visual diff image (optional, PNG bytes)
    pub diff_image: Option<Vec<u8>>,
}

/// Compare two images and return difference metrics
pub fn compare_images(output: &[u8], reference: &[u8]) -> Result<DiffResult> {
    // Decode images
    let out_img = image::load_from_memory(output)
        .map_err(|e| anyhow!("Failed to decode output: {}", e))?;
    let ref_img = image::load_from_memory(reference)
        .map_err(|e| anyhow!("Failed to decode reference: {}", e))?;

    // Resize if needed to match dimensions
    let (width, height) = ref_img.dimensions();
    let out_img = if out_img.dimensions() != (width, height) {
        out_img.resize_exact(width, height, image::imageops::FilterType::Lanczos3)
    } else {
        out_img
    };

    let out_rgba = out_img.to_rgba8();
    let ref_rgba = ref_img.to_rgba8();

    // Calculate metrics
    let mut total_error: f64 = 0.0;
    let mut peak_error: f32 = 0.0;
    let pixel_count = (width * height) as f64;

    // Create diff visualization
    let mut diff_img = RgbaImage::new(width, height);

    // Track error by region (divide into 3x3 grid)
    let mut region_errors = [[0.0f64; 3]; 3];
    let region_w = width / 3;
    let region_h = height / 3;

    for y in 0..height {
        for x in 0..width {
            let out_pixel = out_rgba.get_pixel(x, y);
            let ref_pixel = ref_rgba.get_pixel(x, y);

            // Calculate per-pixel error (normalized 0-1)
            let r_diff = (out_pixel[0] as f32 - ref_pixel[0] as f32).abs() / 255.0;
            let g_diff = (out_pixel[1] as f32 - ref_pixel[1] as f32).abs() / 255.0;
            let b_diff = (out_pixel[2] as f32 - ref_pixel[2] as f32).abs() / 255.0;

            let pixel_error = (r_diff + g_diff + b_diff) / 3.0;
            total_error += pixel_error as f64;
            peak_error = peak_error.max(pixel_error);

            // Track region error
            let rx = (x / region_w).min(2) as usize;
            let ry = (y / region_h).min(2) as usize;
            region_errors[ry][rx] += pixel_error as f64;

            // Create diff visualization (red = difference)
            let diff_intensity = (pixel_error * 255.0 * 3.0).min(255.0) as u8;
            diff_img.put_pixel(
                x,
                y,
                Rgba([diff_intensity, 0, (255 - diff_intensity), 255]),
            );
        }
    }

    let mae = (total_error / pixel_count) as f32;
    let similarity = 1.0 - mae;

    // Analyze regions
    let region_names = [
        ["top-left", "top-center", "top-right"],
        ["middle-left", "center", "middle-right"],
        ["bottom-left", "bottom-center", "bottom-right"],
    ];

    let region_pixel_count = (region_w * region_h) as f64;
    let threshold = 0.05; // 5% average error threshold

    let mut diff_regions = Vec::new();
    for ry in 0..3 {
        for rx in 0..3 {
            let region_mae = region_errors[ry][rx] / region_pixel_count;
            if region_mae > threshold {
                let severity = if region_mae > 0.2 {
                    "high"
                } else if region_mae > 0.1 {
                    "medium"
                } else {
                    "slight"
                };
                diff_regions.push(format!(
                    "{}: {} difference ({:.1}%)",
                    region_names[ry][rx],
                    severity,
                    region_mae * 100.0
                ));
            }
        }
    }

    // Encode diff image
    let mut diff_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut diff_bytes);
    diff_img
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| anyhow!("Failed to encode diff image: {}", e))?;

    Ok(DiffResult {
        similarity,
        mae,
        peak_error,
        diff_regions,
        diff_image: Some(diff_bytes),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_images() {
        // Create a simple test image
        let mut img = RgbaImage::new(10, 10);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([128, 128, 128, 255]);
        }

        let mut bytes = Vec::new();
        let mut cursor = Cursor::new(&mut bytes);
        img.write_to(&mut cursor, image::ImageFormat::Png).unwrap();

        let result = compare_images(&bytes, &bytes).unwrap();

        assert!((result.similarity - 1.0).abs() < 0.001);
        assert!(result.mae < 0.001);
        assert!(result.peak_error < 0.001);
    }
}
