use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::color::{Palette, Rgb};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
pub struct WaveDirs {
    pub paints: PathBuf,
}

pub fn ensure_wave_dirs() -> Result<WaveDirs> {
    let root = wave_root()?;
    let paints = root.join("paints");
    let canvas = root.join("canvas");

    fs::create_dir_all(&paints)?;
    fs::create_dir_all(&canvas)?;

    Ok(WaveDirs { paints })
}

pub fn default_config_path() -> PathBuf {
    wave_root()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("config.wave")
}

pub fn wave_root() -> Result<PathBuf> {
    if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(config_home).join("wave"));
    }

    let home = env::var("HOME").map_err(|_| "HOME is not set")?;
    Ok(PathBuf::from(home).join(".config").join("wave"))
}

pub fn expand_tilde(path: &str) -> Result<PathBuf> {
    if path == "~" {
        return Ok(PathBuf::from(
            env::var("HOME").map_err(|_| "HOME is not set")?,
        ));
    }

    if let Some(rest) = path.strip_prefix("~/") {
        return Ok(PathBuf::from(env::var("HOME").map_err(|_| "HOME is not set")?).join(rest));
    }

    Ok(PathBuf::from(path))
}

pub fn write_palette_files(paints: &Path, palette: &Palette) -> Result<()> {
    fs::create_dir_all(paints)?;
    fs::write(paints.join("colors.wave"), palette.to_wave())?;
    fs::write(paints.join("colors.sh"), palette.to_shell())?;
    fs::write(paints.join("colors.css"), palette.to_css())?;
    fs::write(paints.join("colors.json"), palette_json(palette))?;
    Ok(())
}

pub fn read_palette_file(path: &Path) -> Result<Palette> {
    let input = fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let mut background = None;
    let mut foreground = None;
    let mut cursor = None;
    let mut colors = [None; 16];

    for raw_line in input.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim().trim_matches(['\'', '"']);
        let value = value.trim_start_matches('#');
        let Some(color) = Rgb::from_hex(value) else {
            continue;
        };

        match key.trim() {
            "background" => background = Some(color),
            "foreground" => foreground = Some(color),
            "cursor" => cursor = Some(color),
            key if key.starts_with("color") => {
                if let Ok(idx) = key[5..].parse::<usize>() {
                    if let Some(slot) = colors.get_mut(idx) {
                        *slot = Some(color);
                    }
                }
            }
            _ => {}
        }
    }

    let mut resolved = [Rgb::new(0, 0, 0); 16];
    for (idx, color) in colors.into_iter().enumerate() {
        resolved[idx] = color.ok_or_else(|| format!("{} missing color{idx}", path.display()))?;
    }

    Ok(Palette {
        background: background.unwrap_or(resolved[0]),
        foreground: foreground.unwrap_or(resolved[15]),
        cursor: cursor.unwrap_or(resolved[15]),
        colors: resolved,
    })
}

fn palette_json(palette: &Palette) -> String {
    let mut colors = serde_json::Map::new();
    for (idx, color) in palette.colors.iter().enumerate() {
        colors.insert(format!("color{idx}"), json!(color.to_string()));
    }

    json!({
        "special": {
            "background": palette.background.to_string(),
            "foreground": palette.foreground.to_string(),
            "cursor": palette.cursor.to_string()
        },
        "colors": colors
    })
    .to_string()
}
