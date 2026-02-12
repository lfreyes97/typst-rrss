use image::{GenericImageView, ImageBuffer, Rgba};
use palette::{Srgb, Mix};
use std::collections::HashMap;
use std::str::FromStr;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result, anyhow};

// Security limits
const MAX_IMAGE_SIZE_MB: u64 = 256;
const MAX_IMAGE_DIMENSION: u32 = 4096;

pub fn recolor_image(
    image_path: &str,
    theme_name: &str,
    theme: &HashMap<String, String>,
    output_path: Option<&str>,
    intensity: f32,
) -> Result<String> {
    // Resource limit check
    let metadata = std::fs::metadata(image_path).context("Failed to read image metadata")?;
    if metadata.len() > MAX_IMAGE_SIZE_MB * 1024 * 1024 {
        return Err(anyhow!(
            "Image file too large: {} MB (max: {} MB)",
            metadata.len() / (1024 * 1024),
            MAX_IMAGE_SIZE_MB
        ));
    }
    
    let img = image::open(image_path).context("Failed to open image")?;
    let (width, height) = img.dimensions();
    
    if width > MAX_IMAGE_DIMENSION || height > MAX_IMAGE_DIMENSION {
        return Err(anyhow!(
            "Image dimensions too large: {}x{} (max: {}x{})",
            width, height, MAX_IMAGE_DIMENSION, MAX_IMAGE_DIMENSION
        ));
    }

    let p = Path::new(image_path);
    let root_path = p.parent().unwrap_or_else(|| Path::new("."));
    let stem = p.file_stem().and_then(|s| s.to_str()).context("Invalid filename")?;
    let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("png");
    
    let default_output = root_path.join(format!("{}_{}.{}", stem, theme_name, ext));
    let out_path = output_path.map(PathBuf::from).unwrap_or(default_output);
    let is_jpeg = out_path.extension().map_or(false, |e| {
         let s = e.to_string_lossy().to_lowercase();
         s == "jpg" || s == "jpeg"
    });

    let bg_hex = theme.get("bg").ok_or_else(|| anyhow!("Theme missing 'bg'"))?;
    let primary_hex = theme.get("primary").ok_or_else(|| anyhow!("Theme missing 'primary'"))?;
    
    let bg_color: palette::LinSrgb = Srgb::from_str(bg_hex).map_err(|_| anyhow!("Invalid bg color"))?.into_linear();
    let primary_color: palette::LinSrgb = Srgb::from_str(primary_hex).map_err(|_| anyhow!("Invalid primary color"))?.into_linear();
    
    let mut buffer = ImageBuffer::new(width, height);
    
    for (x, y, pixel) in img.pixels() {
        let (r, g, b, a) = (pixel[0], pixel[1], pixel[2], pixel[3]);
        
        // Skip fully transparent pixels if not JPEG
        if a == 0 && !is_jpeg {
            buffer.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            continue;
        }

        // Convert to linear sRGB for correct mixing
        let original = Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0).into_linear();
        
        // Grayscale value (luminance)
        // Rec. 709 luminance coefficients
        let gray = original.red * 0.2126 + original.green * 0.7152 + original.blue * 0.0722;
        
        // Map gray to duotone gradient (mix bg and primary based on luminance)
        // gray 0.0 -> bg, gray 1.0 -> primary
        let duotone = bg_color.mix(primary_color, gray);
        
        // Blend with original based on intensity
        let blended = original.mix(duotone, intensity);
        
        let final_srgb: Srgb = Srgb::from_linear(blended);
        
        buffer.put_pixel(x, y, Rgba([
             (final_srgb.red * 255.0).round() as u8,
             (final_srgb.green * 255.0).round() as u8,
             (final_srgb.blue * 255.0).round() as u8,
             a
        ]));
    }
    
    if is_jpeg {
        // Convert to RGB8 (drop alpha)
        let dynamic = image::DynamicImage::ImageRgba8(buffer);
        let rgb_img = dynamic.to_rgb8();
        rgb_img.save(&out_path).context("Failed to save image")?;
    } else {
        buffer.save(&out_path).context("Failed to save image")?;
    }
    
    Ok(out_path.to_string_lossy().to_string())
}

pub fn generate_contours(image_path: &str) -> Result<String> {
    let img = image::open(image_path).context("Failed to open image for contour")?;
    let gray = img.grayscale();
    
    let p = Path::new(image_path);
    let root = p.parent().unwrap_or_else(|| Path::new("."));
    let stem = p.file_stem().and_then(|s| s.to_str()).context("Invalid file stem")?;
    
    let output_path = root.join(format!("{}_contour.png", stem));
    
    // Saving grayscale as "contour" for now. A real implementation would need edge detection kernel.
    gray.save(&output_path).context("Failed to save contour")?;
    
    Ok(output_path.to_string_lossy().to_string())
}

pub fn generate_noise_contours(_root_path: &Path) -> Result<String> {
     // TODO: Implement Perlin noise contour generation
     Err(anyhow!("Noise contour generation not yet implemented"))
}
