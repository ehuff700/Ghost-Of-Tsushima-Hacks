#[macro_use]
extern crate tracing;

pub mod api;
pub mod errors;
pub mod game_handle;

pub type GamecheatResult<T> = std::result::Result<T, errors::GamecheatError>;
