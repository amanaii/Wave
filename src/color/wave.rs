use std::cmp::Ordering;
use std::path::Path;
use std::process::Command;

use image::imageops::FilterType;
use image::ImageReader;

use super::{ColorExtractor, Palette, Rgb};

#[derive(Debug, Clone)]
pub struct WaveExtractor {
    max_size: u32,
    boxes: usize,
}

impl Default for WaveExtractor {
    fn default() -> Self {
        Self {
            max_size: 256,
            boxes: 16,
        }
    }
}

impl ColorExtractor for WaveExtractor {
    fn extract(&self, image: &Path) -> Result<Palette, Box<dyn std::error::Error>> {
        if let Some(colors) = imagemagick_palette(image) {
            return Ok(classic_adjust(colors));
        }

        let pixels = self.load_pixels(image)?;
        if pixels.is_empty() {
            return Err("image has no visible pixels".into());
        }

        Ok(self.palette_from_pixels(pixels))
    }
}

fn imagemagick_palette(image: &Path) -> Option<Vec<Rgb>> {
    let commands: &[&[&str]] = &[&["magick"], &["magick", "convert"], &["convert"]];

    for command in commands {
        for color_count in 16..36 {
            let mut process = Command::new(command[0]);
            process.args(&command[1..]);
            process.arg(format!("{}[0]", image.display()));
            process.args(["-resize", "25%", "-colors", &color_count.to_string()]);
            process.args(["-unique-colors", "txt:-"]);

            let output = match process.output() {
                Ok(output) if output.status.success() => output.stdout,
                _ => continue,
            };

            let colors = parse_imagemagick_colors(&output);
            if colors.len() >= 16 {
                return Some(colors);
            }
        }
    }

    None
}

fn parse_imagemagick_colors(output: &[u8]) -> Vec<Rgb> {
    let Ok(text) = std::str::from_utf8(output) else {
        return Vec::new();
    };

    text.split(|char: char| !char.is_ascii_hexdigit() && char != '#')
        .filter_map(|token| {
            let hex = token.strip_prefix('#')?;
            if hex.len() >= 6 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                Rgb::from_hex(&hex[..6])
            } else {
                None
            }
        })
        .collect()
}

fn classic_adjust(colors: Vec<Rgb>) -> Palette {
    let mut adjusted = Vec::with_capacity(16);
    adjusted.push(colors[0]);
    adjusted.extend_from_slice(&colors[8..16]);
    adjusted.extend_from_slice(&colors[8..15]);

    adjusted[0] = classic_dark_background(adjusted[0]);
    adjusted[7] = classic_lighten(adjusted[0], 0.75);
    adjusted[8] = classic_saturate(classic_lighten(adjusted[0], 0.35), 0.10);
    adjusted[15] = adjusted[7];

    let colors: [Rgb; 16] = adjusted
        .try_into()
        .unwrap_or_else(|_| [Rgb::new(0, 0, 0); 16]);

    Palette {
        background: colors[0],
        foreground: colors[15],
        cursor: colors[15],
        colors,
    }
}

fn classic_dark_background(color: Rgb) -> Rgb {
    let mut color = if color.r >= 16 {
        classic_darken(color, 0.40)
    } else {
        color
    };

    if color.r < 16 || color.g < 16 || color.b < 16 {
        color = classic_saturate(classic_lighten(color, 0.03), 0.40);
    }

    color
}

fn classic_darken(color: Rgb, amount: f32) -> Rgb {
    let keep = 1.0 - amount;
    Rgb::new(
        (color.r as f32 * keep) as u8,
        (color.g as f32 * keep) as u8,
        (color.b as f32 * keep) as u8,
    )
}

fn classic_lighten(color: Rgb, amount: f32) -> Rgb {
    Rgb::new(
        (color.r as f32 + (255.0 - color.r as f32) * amount) as u8,
        (color.g as f32 + (255.0 - color.g as f32) * amount) as u8,
        (color.b as f32 + (255.0 - color.b as f32) * amount) as u8,
    )
}

fn classic_saturate(color: Rgb, amount: f32) -> Rgb {
    let (hue, lightness, _) = rgb_to_hls(color);
    hls_to_rgb(hue, lightness, amount)
}

fn rgb_to_hls(color: Rgb) -> (f32, f32, f32) {
    let r = color.r as f32 / 255.0;
    let g = color.g as f32 / 255.0;
    let b = color.b as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let lightness = (max + min) / 2.0;

    if max == min {
        return (0.0, lightness, 0.0);
    }

    let delta = max - min;
    let saturation = if lightness <= 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };
    let hue = if max == r {
        ((g - b) / delta).rem_euclid(6.0) / 6.0
    } else if max == g {
        ((b - r) / delta + 2.0) / 6.0
    } else {
        ((r - g) / delta + 4.0) / 6.0
    };

    (hue, lightness, saturation)
}

fn hls_to_rgb(hue: f32, lightness: f32, saturation: f32) -> Rgb {
    if saturation == 0.0 {
        let value = (lightness * 255.0) as u8;
        return Rgb::new(value, value, value);
    }

    let m2 = if lightness <= 0.5 {
        lightness * (1.0 + saturation)
    } else {
        lightness + saturation - lightness * saturation
    };
    let m1 = 2.0 * lightness - m2;

    Rgb::new(
        (hue_to_rgb(m1, m2, hue + 1.0 / 3.0) * 255.0) as u8,
        (hue_to_rgb(m1, m2, hue) * 255.0) as u8,
        (hue_to_rgb(m1, m2, hue - 1.0 / 3.0) * 255.0) as u8,
    )
}

fn hue_to_rgb(m1: f32, m2: f32, hue: f32) -> f32 {
    let hue = hue.rem_euclid(1.0);

    if hue < 1.0 / 6.0 {
        m1 + (m2 - m1) * hue * 6.0
    } else if hue < 0.5 {
        m2
    } else if hue < 2.0 / 3.0 {
        m1 + (m2 - m1) * (2.0 / 3.0 - hue) * 6.0
    } else {
        m1
    }
}

impl WaveExtractor {
    fn load_pixels(&self, image: &Path) -> Result<Vec<Rgb>, Box<dyn std::error::Error>> {
        let decoded = ImageReader::open(image)?.with_guessed_format()?.decode()?;
        let thumbnail = decoded.resize(self.max_size, self.max_size, FilterType::Triangle);
        let rgba = thumbnail.to_rgba8();
        let mut pixels = Vec::with_capacity((rgba.width() * rgba.height()) as usize);

        for pixel in rgba.pixels() {
            if pixel[3] > 8 {
                pixels.push(Rgb::new(pixel[0], pixel[1], pixel[2]));
            }
        }

        Ok(pixels)
    }

    fn palette_from_pixels(&self, pixels: Vec<Rgb>) -> Palette {
        let mut swatches = median_cut(pixels, self.boxes);
        let primary = primary_color(&swatches);
        swatches.sort_by(|left, right| compare_luminance(&left.color, &right.color));
        let quantized: Vec<Rgb> = swatches.iter().map(|swatch| swatch.color).collect();

        let lightest = quantized.last().copied().unwrap_or(Rgb::new(255, 255, 255));
        let background = wave_background(primary);
        let foreground = readable_foreground(lightest, background);
        let accents = accents(&quantized);

        let mut colors = [Rgb::new(0, 0, 0); 16];
        colors[0] = background;
        colors[7] = foreground.darken(0.08);
        colors[8] = background.lighten(0.22);
        colors[15] = foreground.lighten(0.08);

        for idx in 0..6 {
            let base = accents
                .get(idx)
                .copied()
                .unwrap_or_else(|| fallback_accent(&quantized, idx));
            colors[idx + 1] = normalize_accent(base, background);
            colors[idx + 9] = colors[idx + 1].lighten(0.24);
        }

        Palette {
            background,
            foreground,
            cursor: foreground,
            colors,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Swatch {
    color: Rgb,
    population: usize,
}

fn median_cut(mut pixels: Vec<Rgb>, target_boxes: usize) -> Vec<Swatch> {
    if pixels.len() <= target_boxes {
        return pixels
            .into_iter()
            .map(|color| Swatch {
                color,
                population: 1,
            })
            .collect();
    }

    let mut boxes = vec![ColorBox::new(std::mem::take(&mut pixels))];

    while boxes.len() < target_boxes {
        let Some((split_idx, _)) = boxes
            .iter()
            .enumerate()
            .filter(|(_, color_box)| color_box.pixels.len() > 1)
            .max_by_key(|(_, color_box)| color_box.score())
        else {
            break;
        };

        let color_box = boxes.swap_remove(split_idx);
        let (left, right) = color_box.split();
        boxes.push(left);
        boxes.push(right);
    }

    boxes
        .into_iter()
        .map(|color_box| color_box.swatch())
        .collect()
}

#[derive(Debug)]
struct ColorBox {
    pixels: Vec<Rgb>,
    r_min: u8,
    r_max: u8,
    g_min: u8,
    g_max: u8,
    b_min: u8,
    b_max: u8,
}

impl ColorBox {
    fn new(pixels: Vec<Rgb>) -> Self {
        let mut color_box = Self {
            pixels,
            r_min: u8::MAX,
            r_max: u8::MIN,
            g_min: u8::MAX,
            g_max: u8::MIN,
            b_min: u8::MAX,
            b_max: u8::MIN,
        };
        color_box.recompute_bounds();
        color_box
    }

    fn score(&self) -> usize {
        self.pixels.len() * self.range() as usize
    }

    fn range(&self) -> u8 {
        self.r_range().max(self.g_range()).max(self.b_range())
    }

    fn r_range(&self) -> u8 {
        self.r_max.saturating_sub(self.r_min)
    }

    fn g_range(&self) -> u8 {
        self.g_max.saturating_sub(self.g_min)
    }

    fn b_range(&self) -> u8 {
        self.b_max.saturating_sub(self.b_min)
    }

    fn split(mut self) -> (Self, Self) {
        if self.r_range() >= self.g_range() && self.r_range() >= self.b_range() {
            self.pixels.sort_unstable_by_key(|pixel| pixel.r);
        } else if self.g_range() >= self.b_range() {
            self.pixels.sort_unstable_by_key(|pixel| pixel.g);
        } else {
            self.pixels.sort_unstable_by_key(|pixel| pixel.b);
        }

        let midpoint = self.pixels.len() / 2;
        let right = self.pixels.split_off(midpoint);
        (Self::new(self.pixels), Self::new(right))
    }

    fn swatch(self) -> Swatch {
        let mut r = 0usize;
        let mut g = 0usize;
        let mut b = 0usize;
        let len = self.pixels.len().max(1);

        for pixel in self.pixels {
            r += pixel.r as usize;
            g += pixel.g as usize;
            b += pixel.b as usize;
        }

        Swatch {
            color: Rgb::new((r / len) as u8, (g / len) as u8, (b / len) as u8),
            population: len,
        }
    }

    fn recompute_bounds(&mut self) {
        for pixel in &self.pixels {
            self.r_min = self.r_min.min(pixel.r);
            self.r_max = self.r_max.max(pixel.r);
            self.g_min = self.g_min.min(pixel.g);
            self.g_max = self.g_max.max(pixel.g);
            self.b_min = self.b_min.min(pixel.b);
            self.b_max = self.b_max.max(pixel.b);
        }
    }
}

fn primary_color(swatches: &[Swatch]) -> Rgb {
    swatches
        .iter()
        .copied()
        .filter(|swatch| {
            swatch.color.saturation() >= 0.12 && (0.12..=0.86).contains(&swatch.color.luminance())
        })
        .max_by(|left, right| {
            primary_score(*left)
                .partial_cmp(&primary_score(*right))
                .unwrap_or(Ordering::Equal)
        })
        .map(|swatch| swatch.color)
        .or_else(|| {
            swatches
                .iter()
                .copied()
                .max_by_key(|swatch| swatch.population)
                .map(|swatch| swatch.color)
        })
        .unwrap_or(Rgb::new(32, 32, 32))
}

fn primary_score(swatch: Swatch) -> f32 {
    let color = swatch.color;
    let saturation = color.saturation();
    let luminance = color.luminance();
    let usable_luminance = 1.0 - (luminance - 0.42).abs().min(0.42);

    swatch.population as f32 * (0.65 + saturation) * usable_luminance.max(0.30)
}

fn wave_background(primary: Rgb) -> Rgb {
    let amount = if primary.luminance() > 0.55 {
        0.70
    } else if primary.luminance() > 0.32 {
        0.58
    } else {
        0.36
    };

    primary.darken(amount)
}

fn accents(colors: &[Rgb]) -> Vec<Rgb> {
    let mut accents: Vec<Rgb> = colors
        .iter()
        .copied()
        .filter(|color| color.saturation() >= 0.12 && (0.16..=0.88).contains(&color.luminance()))
        .collect();

    accents.sort_by(|left, right| {
        right
            .saturation()
            .partial_cmp(&left.saturation())
            .unwrap_or(Ordering::Equal)
    });
    accents.truncate(8);
    accents.sort_by(|left, right| {
        left.hue()
            .partial_cmp(&right.hue())
            .unwrap_or(Ordering::Equal)
    });
    accents
}

fn fallback_accent(colors: &[Rgb], idx: usize) -> Rgb {
    colors
        .get((idx + 1).min(colors.len().saturating_sub(1)))
        .copied()
        .unwrap_or(Rgb::new(128, 128, 128))
}

fn normalize_accent(color: Rgb, background: Rgb) -> Rgb {
    if contrast(color, background) < 1.55 {
        if background.luminance() < 0.5 {
            color.lighten(0.18)
        } else {
            color.darken(0.18)
        }
    } else {
        color
    }
}

fn readable_foreground(lightest: Rgb, background: Rgb) -> Rgb {
    if contrast(lightest, background) < 3.0 {
        if background.luminance() < 0.5 {
            Rgb::new(238, 238, 238)
        } else {
            Rgb::new(17, 17, 17)
        }
    } else {
        lightest
    }
}

fn contrast(left: Rgb, right: Rgb) -> f32 {
    let l1 = left.luminance() + 0.05;
    let l2 = right.luminance() + 0.05;
    l1.max(l2) / l1.min(l2)
}

fn compare_luminance(left: &Rgb, right: &Rgb) -> Ordering {
    left.luminance()
        .partial_cmp(&right.luminance())
        .unwrap_or(Ordering::Equal)
}
