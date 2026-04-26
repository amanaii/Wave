use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "wave",
    version,
    about = "Generate classic 16-color schemes from images"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Extract a 16-color scheme from an image and apply configured canvases.
    Tide {
        /// Image path.
        image: PathBuf,

        /// Config path. Defaults to ~/.config/wave/config.wave.
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Skip rendering canvases and post-apply commands.
        #[arg(long)]
        no_apply: bool,

        /// Skip terminal OSC color sequences.
        #[arg(long)]
        no_sequences: bool,
    },

    /// Preview the current colorscheme from ~/.config/wave/paints/colors.wave.
    Preview,
}
