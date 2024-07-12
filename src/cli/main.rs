#[macro_use]
extern crate tracing;

use clap::Parser;
use cli::{add_material, infinite_ammo, set_material, subtract_material, Cli, Subcommands};
use gamecheat::game_handle::GameHandle;
use tracing::level_filters::LevelFilter;
pub mod cli;

fn entry() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let game_handle = GameHandle::new("GhostOfTsushima.exe")?;
    match cli.command {
        Subcommands::Set { material, value } => {
            let new_value = set_material(&game_handle, material, value)?;
            info!("successfully updated \"{material}\" to value {new_value}");
        }
        Subcommands::Add { material, value } => {
            let new_value = add_material(&game_handle, material, value)?;
            info!("successfully updated \"{material}\" to value {new_value}");
        }
        Subcommands::Subtract { material, value } => {
            let new_value = subtract_material(&game_handle, material, value)?;
            info!("successfully updated \"{material}\" to value {new_value}");
        }
        Subcommands::Infinite { ammo_type } => {
            infinite_ammo(&game_handle, ammo_type)?;
            info!("successfully set \"{ammo_type}\" to infinite");
        }
    };

    Ok(())
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();
    let _ = entry().inspect_err(|e| error!("{}", e));
}
