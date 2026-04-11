use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Reqwest error: {0}")]
    Reqwest(String),

    #[error("IO error: {0}")]
    Io(String),
}

impl std::convert::From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Reqwest(err.to_string())
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
}
