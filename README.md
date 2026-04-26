<div align="center">

# Wave

### Fast 16-color scheme generation for Linux ricing

Wave turns wallpapers into terminal-ready color schemes, renders user templates, and applies live terminal colors with a small Rust CLI.

<p>
  <img alt="Rust" src="https://img.shields.io/badge/Rust-1.70%2B-f74c00?style=for-the-badge&logo=rust&logoColor=white">
  <img alt="CLI" src="https://img.shields.io/badge/CLI-wave-2f81f7?style=for-the-badge&logo=gnubash&logoColor=white">
  <img alt="License" src="https://img.shields.io/badge/License-MIT-3fb950?style=for-the-badge">
  <img alt="Status" src="https://img.shields.io/badge/Status-Alpha-f2cc60?style=for-the-badge">
</p>

```sh
wave tide ~/Pictures/wallpaper.png
```

</div>

## Overview

Wave is a modular Rust CLI for generating classic 16-color schemes from images. It is built for speed, low memory usage, and ricing workflows where one wallpaper should drive terminal colors, app themes, visualizers, bars, launchers, and other dotfile-managed tools.

Wave uses this layout:

| Path | Purpose |
| --- | --- |
| `~/.config/wave/config.wave` | Main Wave config |
| `~/.config/wave/canvas/` | User templates |
| `~/.config/wave/paints/` | Generated palettes and rendered files |

## Features

- Generates `color0` through `color15` from an image.
- Applies live terminal colors with OSC sequences.
- Prints color-block previews by default.
- Renders templates for Kitty, Cava, Waybar, Hyprland, Rofi, and more.
- Supports post-apply commands per target.
- Exports `colors.wave`, `colors.sh`, `colors.css`, and `colors.json`.
- Uses ImageMagick for high-quality quantization when available.
- Falls back to a native Rust extractor when ImageMagick is unavailable.

## Installation

### Build From Source

```sh
git clone https://github.com/<your-name>/wave.git
cd wave
cargo build --release
```

Binary:

```sh
./target/release/wave --help
```

### Install Locally

```sh
cargo install --path .
```

This installs `wave` into Cargo's binary directory, usually:

```txt
~/.cargo/bin/wave
```

Make sure `~/.cargo/bin` is in your `PATH`.

### Optional Dependency

Wave works without ImageMagick, but ImageMagick improves palette accuracy.

Arch:

```sh
sudo pacman -S imagemagick
```

Debian/Ubuntu:

```sh
sudo apt install imagemagick
```

Fedora:

```sh
sudo dnf install ImageMagick
```

## Usage

Generate a scheme from an image:

```sh
wave tide ~/Pictures/wallpaper.png
```

Generate colors without rendering templates:

```sh
wave tide ~/Pictures/wallpaper.png --no-apply
```

Generate colors without applying terminal OSC sequences:

```sh
wave tide ~/Pictures/wallpaper.png --no-sequences
```

Preview the current generated colors:

```sh
wave preview
```

`wave tide` always prints color blocks after generating a palette.

## Generated Files

After running `wave tide`, Wave writes:

```txt
~/.config/wave/paints/colors.wave
~/.config/wave/paints/colors.sh
~/.config/wave/paints/colors.css
~/.config/wave/paints/colors.json
```

Use these files directly in shell scripts, CSS-based configs, or custom tooling.

## Configuration

Wave reads:

```txt
~/.config/wave/config.wave
```

If the file does not exist, Wave creates a default one.

Example:

```ini
[kitty]
canvas='~/.config/wave/canvas/kitty-colors.conf'
output='~/.config/wave/paints/kitty-colors.conf'
post-apply='killall -SIGUSR1 kitty'

[cava]
canvas='~/.config/wave/canvas/cava.colors'
output='~/.config/wave/paints/cava.colors'
post-apply='killall cava'
```

Config fields:

| Field | Required | Meaning |
| --- | --- | --- |
| `canvas` | Yes | Template source file |
| `output` | No | Rendered output file |
| `post-apply` | No | Shell command to run after rendering |

If `output` is omitted, Wave writes to:

```txt
~/.config/wave/paints/<canvas-file-name>
```

Point applications at files in `~/.config/wave/paints/`, not at the templates in `~/.config/wave/canvas/`.

## Templates

Templates are plain text files with placeholders. Put them in:

```txt
~/.config/wave/canvas/
```

Example Kitty template:

```conf
background {background}
foreground {foreground}
cursor {cursor}

color0 {color0}
color1 {color1}
color2 {color2}
color3 {color3}
color4 {color4}
color5 {color5}
color6 {color6}
color7 {color7}
color8 {color8}
color9 {color9}
color10 {color10}
color11 {color11}
color12 {color12}
color13 {color13}
color14 {color14}
color15 {color15}
```

Add this target to `config.wave`:

```ini
[kitty]
canvas='~/.config/wave/canvas/kitty-colors.conf'
output='~/.config/wave/paints/kitty-colors.conf'
```

Include the generated file from `kitty.conf`:

```conf
include ~/.config/wave/paints/kitty-colors.conf
```

Run:

```sh
wave tide ~/Pictures/wallpaper.png
```

## Placeholders

| Placeholder | Output |
| --- | --- |
| `{background}` | `#rrggbb` |
| `{foreground}` | `#rrggbb` |
| `{cursor}` | `#rrggbb` |
| `{color0}` through `{color15}` | `#rrggbb` |
| `{{color0}}` | `#rrggbb`, double braces also work |
| `{color0.strip}` | `rrggbb` |
| `{color0.rgb}` | `r,g,b` |
| `{color0.r}` | Red channel |
| `{color0.g}` | Green channel |
| `{color0.b}` | Blue channel |

All placeholder suffixes work for `background`, `foreground`, `cursor`, and every `colorN`.

## More Template Examples

### Cava

`~/.config/wave/canvas/cava.colors`:

```ini
gradient = 1
gradient_count = 8
gradient_color_1 = '{color1}'
gradient_color_2 = '{color2}'
gradient_color_3 = '{color3}'
gradient_color_4 = '{color4}'
gradient_color_5 = '{color5}'
gradient_color_6 = '{color6}'
gradient_color_7 = '{color7}'
gradient_color_8 = '{color15}'
```

`~/.config/wave/config.wave`:

```ini
[cava]
canvas='~/.config/wave/canvas/cava.colors'
output='~/.config/wave/paints/cava.colors'
post-apply='killall cava'
```

### CSS Variables

`~/.config/wave/canvas/theme.css`:

```css
:root {
  --bg: {background};
  --fg: {foreground};
  --accent: {color4};
  --accent-rgb: {color4.rgb};
}
```

## Terminal Colors

By default, Wave sends OSC sequences to the current terminal:

| Sequence | Purpose |
| --- | --- |
| OSC `4` | ANSI colors `0..15` |
| OSC `10` | Foreground |
| OSC `11` | Background |
| OSC `12` | Cursor |

Disable this with:

```sh
wave tide ~/Pictures/wallpaper.png --no-sequences
```

Some terminals do not allow live color changes, or only apply them to the current terminal session.

## Commands

```txt
wave tide <IMAGE> [--config <PATH>] [--no-apply] [--no-sequences]
wave preview
```

## Project Layout

```txt
src/main.rs         Entry point
src/cli.rs          CLI arguments
src/app.rs          Application orchestration
src/color/          Extraction, quantization, palette model
src/config/         Config parser
src/fs.rs           Wave paths and palette file output
src/log.rs          Terminal logging
src/preview.rs      Color block preview
src/template.rs     Canvas rendering
src/terminal.rs     OSC terminal color application
```

## Development

Format:

```sh
cargo fmt
```

Check:

```sh
cargo check
```

Lint:

```sh
cargo clippy -- -D warnings
```

Build release:

```sh
cargo build --release
```

## License

MIT
