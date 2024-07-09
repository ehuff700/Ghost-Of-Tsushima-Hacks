#[macro_use]
extern crate tracing;

pub mod api;
pub mod errors;
pub mod helpers;

pub type GamecheatResult<T> = std::result::Result<T, errors::GamecheatError>;
