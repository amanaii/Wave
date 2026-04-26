mod app;
mod cli;
mod color;
mod config;
mod fs;
mod log;
mod preview;
mod template;
mod terminal;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();

    if let Err(error) = app::run(cli) {
        eprintln!("wave: {error}");
        std::process::exit(1);
    }
}
