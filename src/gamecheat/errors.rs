use thiserror::Error;

#[derive(Error, Debug)]
pub enum GamecheatError {
    #[error("The game process \"{0}\" couldn't be found")]
    GameProcessNotFound(&'static str),
    #[error("failed to read memory from the game process: {0}")]
    MemoryReadError(std::io::Error),
    #[error("failed to write memory to the game process: {0}")]
    MemoryWriteError(std::io::Error),
}
