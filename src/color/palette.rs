use super::Rgb;

#[derive(Debug, Clone)]
pub struct Palette {
    pub background: Rgb,
    pub foreground: Rgb,
    pub cursor: Rgb,
    pub colors: [Rgb; 16],
}

impl Palette {
    pub fn get(&self, name: &str) -> Option<Rgb> {
        match name {
            "background" => Some(self.background),
            "foreground" => Some(self.foreground),
            "cursor" => Some(self.cursor),
            _ => name
                .strip_prefix("color")
                .and_then(|idx| idx.parse::<usize>().ok())
                .and_then(|idx| self.colors.get(idx).copied()),
        }
    }

    pub fn to_wave(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("background='{}'\n", self.background));
        output.push_str(&format!("foreground='{}'\n", self.foreground));
        output.push_str(&format!("cursor='{}'\n", self.cursor));
        for (idx, color) in self.colors.iter().enumerate() {
            output.push_str(&format!("color{idx}='{color}'\n"));
        }
        output
    }

    pub fn to_shell(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("export background='{}'\n", self.background));
        output.push_str(&format!("export foreground='{}'\n", self.foreground));
        output.push_str(&format!("export cursor='{}'\n", self.cursor));
        for (idx, color) in self.colors.iter().enumerate() {
            output.push_str(&format!("export color{idx}='{color}'\n"));
        }
        output
    }

    pub fn to_css(&self) -> String {
        let mut output = String::from(":root {\n");
        output.push_str(&format!("  --background: {};\n", self.background));
        output.push_str(&format!("  --foreground: {};\n", self.foreground));
        output.push_str(&format!("  --cursor: {};\n", self.cursor));
        for (idx, color) in self.colors.iter().enumerate() {
            output.push_str(&format!("  --color{idx}: {color};\n"));
        }
        output.push_str("}\n");
        output
    }
}
