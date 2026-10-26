//! Textures as `MyLoadTexture`/`LoadTexture` makes them.
//!
//! The files are 24-bit BMP and JPEG without alpha channels, so Blitz
//! builds the alpha from the flags: 2 (alpha) uses the brightness, so dark
//! is see-through, and 4 (masked) makes pure black transparent. Blitz's
//! default `TextureFilter` adds mipmapping to every texture, so a mip chain
//! is built here too. [`decode`] is plain Rust; [`to_image`] wraps the
//! result for Bevy.

use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// Blitz texture flag 2.
pub const ALPHA: u32 = 2;
/// Blitz texture flag 4.
pub const MASKED: u32 = 4;

/// An RGBA8 texture with its mip chain, largest first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decoded {
    pub width: u32,
    pub height: u32,
    pub levels: Vec<Vec<u8>>,
    /// Some pixel is not fully opaque.
    pub has_alpha: bool,
}

/// Decodes a BMP or JPEG file and applies Blitz's texture `flags`.
pub fn decode(data: &[u8], flags: u32) -> Result<Decoded, image::ImageError> {
    let img = image::load_from_memory(data)?;
    let file_alpha = img.color().has_alpha();
    let mut rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    for px in rgba.pixels_mut() {
        let [r, g, b, _] = px.0;
        if flags & ALPHA != 0 && !file_alpha {
            px.0[3] = ((u32::from(r) + u32::from(g) + u32::from(b)) / 3) as u8;
        }
        if flags & MASKED != 0 && r == 0 && g == 0 && b == 0 {
            px.0[3] = 0;
        } else if flags & (ALPHA | MASKED) == 0 {
            px.0[3] = 255;
        }
    }
    let has_alpha = rgba.pixels().any(|p| p.0[3] < 255);
    let mut levels = vec![rgba.into_raw()];
    let (mut w, mut h) = (width, height);
    while w > 1 || h > 1 {
        let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
        levels.push(halve(levels.last().unwrap(), w, h, nw, nh));
        (w, h) = (nw, nh);
    }
    Ok(Decoded {
        width,
        height,
        levels,
        has_alpha,
    })
}

/// A 2x2 box filter.
fn halve(src: &[u8], w: u32, h: u32, nw: u32, nh: u32) -> Vec<u8> {
    let mut out = Vec::with_capacity((nw * nh * 4) as usize);
    for y in 0..nh {
        for x in 0..nw {
            for c in 0..4 {
                let mut sum = 0u32;
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let sx = (x * 2 + dx).min(w - 1);
                    let sy = (y * 2 + dy).min(h - 1);
                    sum += u32::from(src[((sy * w + sx) * 4 + c) as usize]);
                }
                out.push((sum / 4) as u8);
            }
        }
    }
    out
}

/// A Bevy image that repeats in both directions, as Blitz textures do.
pub fn to_image(d: Decoded) -> Image {
    let mips = d.levels.len() as u32;
    let mut image = Image::new(
        Extent3d {
            width: d.width,
            height: d.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        d.levels[0].clone(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    // `Image::new` takes one level; the chain follows it in the data.
    image.data = Some(d.levels.concat());
    image.texture_descriptor.mip_level_count = mips;
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    });
    image
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bmp(pixels: &[[u8; 3]], w: u32, h: u32) -> Vec<u8> {
        let mut img = image::RgbImage::new(w, h);
        for (i, p) in pixels.iter().enumerate() {
            img.put_pixel(i as u32 % w, i as u32 / w, image::Rgb(*p));
        }
        let mut out = std::io::Cursor::new(Vec::new());
        img.write_to(&mut out, image::ImageFormat::Bmp).unwrap();
        out.into_inner()
    }

    #[test]
    fn flags_build_the_alpha() {
        let data = bmp(&[[0, 0, 0], [255, 255, 255], [30, 60, 90], [0, 0, 1]], 2, 2);
        let plain = decode(&data, 1).unwrap();
        assert!(!plain.has_alpha);
        let alpha = decode(&data, 2).unwrap();
        let a: Vec<u8> = alpha.levels[0].chunks(4).map(|p| p[3]).collect();
        assert_eq!(a, [0, 255, 60, 0]);
        let masked = decode(&data, 4).unwrap();
        let a: Vec<u8> = masked.levels[0].chunks(4).map(|p| p[3]).collect();
        assert_eq!(a, [0, 255, 255, 255]);
        // 2x2 then 1x1.
        assert_eq!(masked.levels.len(), 2);
        assert_eq!(masked.levels[1].len(), 4);
    }
}
