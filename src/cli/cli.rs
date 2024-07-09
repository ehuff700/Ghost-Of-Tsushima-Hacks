use anyhow::anyhow;
use clap::{Parser, Subcommand, ValueEnum};
use gamecheat::game_handle::GameHandle;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Subcommands,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Material {
    /// blessing
    b,
    /// honor
    h,
    /// essence
    e,
    /// sword tokens
    st,
    // bow tokens
    bt,
    // charm tokens
    ct,
    // gw1 token
    gw1,
    // gw2 token
    gw2,
}

impl Material {
    /* Static Offsets for common memory locations */
    pub const ESSENCE_OFFSET: u64 = 0x1cd8b34;
    pub const HONOR_OFFSET: u64 = 0x1cd8b38;
    pub const BLESSING_OFFSET: u64 = 0x1cd8b3c;
    pub const SWORD_TOKEN_OFFSET: u64 = 0x1cd8b40;
    pub const BOW_TOKEN_OFFSET: u64 = 0x1cd8b44;
    pub const CHARM_TOKEN_OFFSET: u64 = 0x1cd8b48;
    pub const GW1_TOKEN_OFFSET: u64 = 0x1cd8b4c;
    pub const GW2_TOKEN_OFFSET: u64 = 0x1cd8b50;

    /// Returns the appropriate offset for the given material.
    pub fn offset(&self) -> u64 {
        match self {
            Material::b => Self::BLESSING_OFFSET,
            Material::h => Self::HONOR_OFFSET,
            Material::e => Self::ESSENCE_OFFSET,
            Material::st => Self::SWORD_TOKEN_OFFSET,
            Material::bt => Self::BOW_TOKEN_OFFSET,
            Material::ct => Self::CHARM_TOKEN_OFFSET,
            Material::gw1 => Self::GW1_TOKEN_OFFSET,
            Material::gw2 => Self::GW2_TOKEN_OFFSET,
        }
    }
}

impl std::fmt::Display for Material {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Material::b => write!(f, "Blessing"),
            Material::h => write!(f, "Honor"),
            Material::e => write!(f, "Essence"),
            Material::st => write!(f, "Sword Tokens"),
            Material::bt => write!(f, "Bow Tokens"),
            Material::ct => write!(f, "Charm Tokens"),
            Material::gw1 => write!(f, "GW1 Token"),
            Material::gw2 => write!(f, "GW2 Token"),
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Subcommands {
    /// Sets a material to a specific value
    Set {
        /// The material to set (blessing is house, honor is hands, essence is fireball)
        #[arg(short, long)]
        material: Material,
        /// The value to set the material to
        #[arg(required = true)]
        value: u32,
    },
    /// Adds a specific value to a material
    Add {
        /// The material to add to (blessing is house, honor is hands, essence is fireball)
        #[arg(short, long)]
        material: Material,
        /// The value to add to the material
        #[arg(required = true)]
        value: u32,
    },
    /// Subtracts a specific value from a material
    Subtract {
        /// The material to subtract from (blessing is house, honor is hands, essence is fireball)
        #[arg(short, long)]
        material: Material,
        /// The value to subtract from the material
        #[arg(required = true)]
        value: u32,
    },
}

/// Sets a given material to the provided value.
///
/// Returns the new value on success
pub fn set_material(
    game_handle: &GameHandle,
    material: Material,
    value: u32,
) -> Result<u32, Box<dyn std::error::Error>> {
    game_handle.write_u32(material.offset(), value)?;
    Ok(value)
}

/// Adds a given material amount to the provided value.
///
/// Returns the new value on success
pub fn add_material(
    game_handle: &GameHandle,
    material: Material,
    value: u32,
) -> Result<u32, Box<dyn std::error::Error>> {
    // Get the current material amount and increment it by the given value.
    let material_amount = game_handle.read_u32(material.offset())?;
    if let Some(value) = material_amount.checked_add(value) {
        // Write the modified value back to the game process.
        game_handle.write_u32(material.offset(), value)?;
        return Ok(value);
    }
    Err(anyhow!("overflow occurred while adding value {value} to {material_amount}").into())
}

/// Subtracts a given material amount from the provided value.
///
/// Returns the new value on success
pub fn subtract_material(
    game_handle: &GameHandle,
    material: Material,
    value: u32,
) -> Result<u32, Box<dyn std::error::Error>> {
    // Get the current material amount and decrement it by the given value.
    let material_amount = game_handle.read_u32(material.offset())?;
    if let Some(value) = material_amount.checked_sub(value) {
        game_handle.write_u32(material.offset(), value)?;
        return Ok(value);
    };
    Err(anyhow!("overflow occurred while subtracting value {value} from {material_amount}").into())
}
