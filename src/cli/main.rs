#[macro_use]
extern crate tracing;

use clap::Parser;
use cli::{add_material, set_material, subtract_material, Cli, Subcommands};
use gamecheat::helpers::GetGameHandle;
use tracing::level_filters::LevelFilter;
pub mod cli;

fn entry() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(game_handle) = GetGameHandle("GhostOfTsushima.exe")? {
        let (material, new_value) = match cli.command {
            Subcommands::Set { material, value } => {
                (material, set_material(&game_handle, material, value)?)
            }
            Subcommands::Add { material, value } => {
                (material, add_material(&game_handle, material, value)?)
            }
            Subcommands::Subtract { material, value } => {
                (material, subtract_material(&game_handle, material, value)?)
            }
        };
        info!("successfully updated \"{material}\" to value {new_value}");
    } else {
        error!("GhostOfTsushima.exe not found in the process list.");
    }
    Ok(())
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    if let Err(why) = entry() {
        error!("FATAL ERROR: {why}");
    }
}
