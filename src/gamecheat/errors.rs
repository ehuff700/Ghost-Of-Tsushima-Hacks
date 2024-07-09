use thiserror::Error;

#[derive(Error, Debug)]
pub enum GamecheatError {
    #[error("The game process \"{0}\" couldn't be found")]
    GameProcessNotFound(&'static str),
    #[error("api error occurred when calling \"{0}\": {1}")]
    OperationError(&'static str, std::io::Error),
    #[error("failed to enumerate processes")]
    ProcessEnumError,
}
