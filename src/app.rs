use std::process::Command as ProcessCommand;

use crate::cli::{Cli, Command};
use crate::color::{ColorExtractor, WaveExtractor};
use crate::config::Config;
use crate::fs::{default_config_path, ensure_wave_dirs, read_palette_file, write_palette_files};
use crate::log::Logger;
use crate::preview::print_blocks;
use crate::template::TemplateRenderer;
use crate::terminal::apply_sequences;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Tide {
            image,
            config,
            no_apply,
            no_sequences,
        } => {
            let logger = Logger::new();
            let dirs = ensure_wave_dirs()?;
            let config_path = config.unwrap_or_else(default_config_path);
            let config = Config::load_or_create(&config_path)?;

            logger.info("image", &format!("Using image {}.", image.display()));
            logger.info(
                "config",
                &format!("Reading config from {}.", config_path.display()),
            );
            if config.created {
                logger.info("config", "Created default config.");
            }

            let extractor = WaveExtractor::default();
            let palette = extractor.extract(&image)?;
            write_palette_files(&dirs.paints, &palette)?;
            logger.info("colors", "Generated 16-color scheme.");
            logger.info(
                "export",
                &format!("Exported palette files to {}.", dirs.paints.display()),
            );

            if !no_sequences {
                match apply_sequences(&palette) {
                    Ok(true) => logger.info("terminal", "Applied live terminal colors."),
                    Ok(false) => logger.warn("terminal", "Skipped sequences: stdout is not a TTY."),
                    Err(error) => {
                        logger.warn("terminal", &format!("Could not apply sequences: {error}."))
                    }
                }
            }

            print_blocks(&palette);

            if !no_apply {
                if config.targets.is_empty() {
                    logger.info("export", "No canvas targets configured.");
                    logger.info(
                        "hint",
                        "Add sections to ~/.config/wave/config.wave, or run `wave preview`.",
                    );
                }

                let renderer = TemplateRenderer::new(palette.clone(), dirs.paints.clone());
                for target in config.targets {
                    let rendered = renderer.render_target(&target)?;
                    if let Some(command) = target.post_apply {
                        run_post_apply(&target.name, &command)?;
                    }
                    logger.info(
                        "export",
                        &format!("Rendered [{}] to {}.", target.name, rendered.display()),
                    );
                }
            }
        }
        Command::Preview => {
            let dirs = ensure_wave_dirs()?;
            let palette = read_palette_file(&dirs.paints.join("colors.wave"))?;
            print_blocks(&palette);
        }
    }

    Ok(())
}

fn run_post_apply(name: &str, command: &str) -> Result<()> {
    let status = ProcessCommand::new("sh").arg("-c").arg(command).status()?;

    if !status.success() {
        return Err(format!("post-apply for [{name}] failed with {status}").into());
    }

    Ok(())
}
