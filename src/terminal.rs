use std::io::{self, IsTerminal, Write};

use crate::color::{Palette, Rgb};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn apply_sequences(palette: &Palette) -> Result<bool> {
    let stdout = io::stdout();
    if !stdout.is_terminal() {
        return Ok(false);
    }

    let mut stdout = stdout.lock();
    write_special(&mut stdout, 10, palette.foreground)?;
    write_special(&mut stdout, 11, palette.background)?;
    write_special(&mut stdout, 12, palette.cursor)?;

    for (idx, color) in palette.colors.iter().enumerate() {
        write!(stdout, "\x1b]4;{idx};{}\x1b\\", osc_rgb(*color))?;
    }

    stdout.flush()?;
    Ok(true)
}

fn write_special(mut writer: impl Write, code: u8, color: Rgb) -> Result<()> {
    write!(writer, "\x1b]{code};{}\x1b\\", osc_rgb(color))?;
    Ok(())
}

fn osc_rgb(color: Rgb) -> String {
    format!("rgb:{:02x}/{:02x}/{:02x}", color.r, color.g, color.b)
}
