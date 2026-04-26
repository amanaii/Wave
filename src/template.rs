use std::fs;
use std::path::{Path, PathBuf};

use crate::color::{Palette, Rgb};
use crate::config::Target;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
pub struct TemplateRenderer {
    palette: Palette,
    paints_dir: PathBuf,
}

impl TemplateRenderer {
    pub fn new(palette: Palette, paints_dir: PathBuf) -> Self {
        Self {
            palette,
            paints_dir,
        }
    }

    pub fn render_target(&self, target: &Target) -> Result<PathBuf> {
        let output = target
            .output
            .clone()
            .unwrap_or_else(|| self.default_output_path(&target.canvas));
        let template = fs::read_to_string(&target.canvas)?;
        let rendered = self.render(&template);

        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&output, rendered)?;

        Ok(output)
    }

    fn default_output_path(&self, canvas: &Path) -> PathBuf {
        canvas
            .file_name()
            .map(|name| self.paints_dir.join(name))
            .unwrap_or_else(|| self.paints_dir.join("canvas.out"))
    }

    fn render(&self, template: &str) -> String {
        let mut rendered = template.to_string();

        for name in ["background", "foreground", "cursor"] {
            if let Some(color) = self.palette.get(name) {
                replace_color(&mut rendered, name, color);
            }
        }

        for idx in 0..16 {
            let name = format!("color{idx}");
            replace_color(&mut rendered, &name, self.palette.colors[idx]);
        }

        rendered
    }
}

fn replace_color(rendered: &mut String, name: &str, color: Rgb) {
    let values = [
        ("", color.to_string()),
        (".strip", color.strip_hex()),
        (".rgb", color.rgb_csv()),
        (".r", color.r.to_string()),
        (".g", color.g.to_string()),
        (".b", color.b.to_string()),
    ];

    for (suffix, value) in values {
        let token = format!("{name}{suffix}");
        *rendered = rendered.replace(&format!("{{{{{token}}}}}"), &value);
        *rendered = rendered.replace(&format!("{{{token}}}"), &value);
    }
}
