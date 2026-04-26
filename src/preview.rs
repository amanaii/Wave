use std::io::{self, IsTerminal};

use crate::color::{Palette, Rgb};

pub fn print_blocks(palette: &Palette) {
    if !io::stdout().is_terminal() {
        print_plain_blocks(palette);
        return;
    }

    println!();
    println!("          0    1    2    3    4    5    6    7");
    print_row("normal ", &palette.colors[0..8]);
    print_row("bright ", &palette.colors[8..16]);
    print_row(
        "special",
        &[palette.background, palette.foreground, palette.cursor],
    );
    println!("          bg   fg   cur");
    println!();
}

fn print_row(label: &str, colors: &[Rgb]) {
    print!("{label} ");
    for color in colors {
        print!("\x1b[48;2;{};{};{}m    \x1b[0m ", color.r, color.g, color.b);
    }
    println!();
}

fn print_plain_blocks(palette: &Palette) {
    println!();
    println!("normal  {}", hex_row(&palette.colors[0..8]));
    println!("bright  {}", hex_row(&palette.colors[8..16]));
    println!(
        "special {} {} {}",
        palette.background, palette.foreground, palette.cursor
    );
    println!();
}

fn hex_row(colors: &[Rgb]) -> String {
    colors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}
