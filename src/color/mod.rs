mod palette;
mod rgb;
mod wave;

use std::path::Path;

pub use palette::Palette;
pub use rgb::Rgb;
pub use wave::WaveExtractor;

pub trait ColorExtractor {
    fn extract(&self, image: &Path) -> Result<Palette, Box<dyn std::error::Error>>;
}
