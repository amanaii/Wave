use std::fs;
use std::path::{Path, PathBuf};

use crate::fs::expand_tilde;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
pub struct Config {
    pub targets: Vec<Target>,
    pub created: bool,
}

#[derive(Debug, Clone)]
pub struct Target {
    pub name: String,
    pub canvas: PathBuf,
    pub output: Option<PathBuf>,
    pub post_apply: Option<String>,
}

impl Config {
    pub fn load_or_create(path: &Path) -> Result<Self> {
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, default_config())?;
            return Ok(Self {
                targets: vec![],
                created: true,
            });
        }

        let mut config = Self::parse(&fs::read_to_string(path)?)?;
        config.created = false;
        Ok(config)
    }

    fn parse(input: &str) -> Result<Self> {
        let mut sections = Vec::new();
        let mut current: Option<PartialTarget> = None;

        for (line_idx, raw_line) in input.lines().enumerate() {
            let line = strip_comment(raw_line).trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                if let Some(section) = current.take() {
                    sections.push(section.finish()?);
                }

                let name = line
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .trim()
                    .to_string();

                if name.is_empty() {
                    return Err(format!("empty section at line {}", line_idx + 1).into());
                }

                current = Some(PartialTarget::new(name));
                continue;
            }

            let Some((key, value)) = line.split_once('=') else {
                return Err(format!("invalid config line {}: {raw_line}", line_idx + 1).into());
            };

            let section = current
                .as_mut()
                .ok_or_else(|| format!("key outside section at line {}", line_idx + 1))?;
            section.set(key.trim(), unquote(value.trim())?)?;
        }

        if let Some(section) = current.take() {
            sections.push(section.finish()?);
        }

        Ok(Self {
            targets: sections,
            created: false,
        })
    }
}

#[derive(Debug)]
struct PartialTarget {
    name: String,
    canvas: Option<PathBuf>,
    output: Option<PathBuf>,
    post_apply: Option<String>,
}

impl PartialTarget {
    fn new(name: String) -> Self {
        Self {
            name,
            canvas: None,
            output: None,
            post_apply: None,
        }
    }

    fn set(&mut self, key: &str, value: String) -> Result<()> {
        match key {
            "canvas" => self.canvas = Some(expand_tilde(&value)?),
            "output" => self.output = Some(expand_tilde(&value)?),
            "post-apply" | "post_apply" => self.post_apply = Some(value),
            unknown => return Err(format!("unknown key `{unknown}` in [{}]", self.name).into()),
        }

        Ok(())
    }

    fn finish(self) -> Result<Target> {
        let canvas = self
            .canvas
            .ok_or_else(|| format!("[{}] missing `canvas`", self.name))?;

        Ok(Target {
            name: self.name,
            canvas,
            output: self.output,
            post_apply: self.post_apply,
        })
    }
}

fn strip_comment(line: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;

    for (idx, byte) in line.bytes().enumerate() {
        match byte {
            b'\'' if !in_double => in_single = !in_single,
            b'"' if !in_single => in_double = !in_double,
            b'#' | b';' if !in_single && !in_double => return &line[..idx],
            _ => {}
        }
    }

    line
}

fn unquote(value: &str) -> Result<String> {
    let quoted = (value.starts_with('\'') && value.ends_with('\''))
        || (value.starts_with('"') && value.ends_with('"'));

    if quoted {
        return Ok(value[1..value.len() - 1].to_string());
    }

    Ok(value.to_string())
}

fn default_config() -> &'static str {
    "# Wave config.
# Sections render canvas templates into ~/.config/wave/paints/ by default.
#
# [kitty]
# canvas='~/.config/wave/canvas/kitty-colors.conf'
# output='~/.config/wave/paints/kitty-colors.conf'
# post-apply='killall -SIGUSR1 kitty'
"
}
