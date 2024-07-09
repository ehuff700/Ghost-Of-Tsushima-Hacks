use thiserror::Error;

#[derive(Error, Debug)]
pub enum GamecheatError {
    #[error("the game process \"{0}\" couldn't be found")]
    GameProcessNotFound(&'static str),
    #[error("game module was not found!")]
    GameModuleNotFound,
    #[error("api error occurred when calling \"{0}\": {1}")]
    OperationError(&'static str, std::io::Error),
    #[error("failed to enumerate processes")]
    ProcessEnumError,
}
